use generate::{Generator, GenerateCtx, Constant};
use arbitrary::Arbitrary;
use call::{Call};

use std::convert::{Into, From};

use rand;

#[derive(Copy, Clone)]
pub struct TestResult {
    status: Status,
}

#[derive(Copy, Clone)]
pub enum Status { Pass, Fail, Discard }

trait Testable {
    fn test<R: rand::Rng>(&self, ctx: &mut GenerateCtx<R>) -> TestResult;
}

trait IntoTestable {
    type Testable: Testable;

    fn into_testable(self) -> Self::Testable;
}

macro_rules! impl_into_testable_for_testable {
    ($ty: ty) => {
        impl IntoTestable for $ty {
            type Testable = Self;

            fn into_testable(self) -> Self {
                self
            }
        }
    };
    ($ident: ident, ($($arg:ident),*)) => {
        impl <$($arg),*> IntoTestable for $ident<$($arg),*>
            where $ident<$($arg),*>: Testable
        {
            type Testable = Self;

            fn into_testable(self) -> Self {
                self
            }
        }
    }
}

impl_into_testable_for_testable! { TestResult }
impl_into_testable_for_testable! { u8 }

impl <T: Clone + Into<TestResult>> Testable for T {
    #[inline]
    fn test<R: rand::Rng>(&self, _: &mut GenerateCtx<R>) -> TestResult {
        self.clone().into()
    }
}

impl From<u8> for TestResult {
    #[inline]
    fn from(_: u8) -> TestResult {
        TestResult { status: Status::Pass }
    }
}

#[derive(Copy, Clone)]
pub struct CheckResult;

pub struct ForAll<G>(G);

fn for_all<G: Generator>(g: G) -> ForAll<G> {
    ForAll(g)
}

pub struct FnTestable<G, F> {
    generator: G,
    f: F
}

impl_into_testable_for_testable! { FnTestable, (G, F) }

impl <G: Generator> ForAll<G> {
    fn property<F, T>(self, f: F) -> FnTestable<G, F>
        where F: Call<Input=G::Output, Output=T>,
              T: Into<TestResult>
    {
        FnTestable { generator: self.0, f: f }
    }
}

impl <G, F, T> Testable for FnTestable<G, F>
    where G: Generator,
          F: Call<Input=G::Output, Output=T>,
          T: Into<TestResult>
{
    #[inline]
    fn test<R: rand::Rng>(&self, ctx: &mut GenerateCtx<R>) -> TestResult {
        let args = self.generator.generate(ctx);
        self.f.call(args).into()
    }
}

pub struct When<P>(P);

fn when<P: Call<Output=bool>>(predicate: P) -> When<P> {
    When(predicate)
}

pub struct WhenProperty<P, F> {
    predicate: P,
    f: F
}

impl <P, F, T> Call for WhenProperty<P, F>
    where P: Call<Output=bool>,
          F: Call<Input=P::Input, Output=T>,
          T: Into<TestResult>,
          P::Input: Clone
{
    type Input = P::Input;
    type Output = TestResult;

    #[inline]
    fn call(&self, args: P::Input) -> TestResult {
        let fn_args = args.clone();
        match self.predicate.call(args) {
            false => TestResult { status: Status::Discard },
            true => self.f.call(fn_args).into()
        }
    }
}

impl <T: Into<TestResult>, Args: Arbitrary, F: Call<Input=Args, Output=T>> IntoTestable for F {
    type Testable = FnTestable<Args::ArbitraryGenerator, Self>;

    #[inline]
    #[allow(unused_variables, non_snake_case)]
    fn into_testable(self) -> Self::Testable {
        let generator = Args::arbitrary();
        for_all(generator).property(self)
    }
}


impl <P> When<P> {
    fn property<T, F>(self, f: F) -> WhenProperty<P, F>
        where P: Call<Output=bool>,
              F: Call<Input=P::Input, Output=T>
    {
        WhenProperty {
            predicate: self.0,
            f: f
        }
    }
}

fn qckchk<T: IntoTestable>(t: T) -> CheckResult
{
    let testable = t.into_testable();
    let mut ctx = GenerateCtx { rng: rand::thread_rng(), size: 5 };
    let test_result = testable.test(&mut ctx);
    CheckResult
}

macro_rules! fn_impls {
    ($($name:ident),*) => {
        impl <$($name: Generator),*> Generator for ($($name,)*) {
            type Output = ($($name::Output,)*);

            #[inline]
            #[allow(unused_variables, non_snake_case)]
            fn generate<R: rand::Rng>(&self, ctx: &mut GenerateCtx<R>) -> Self::Output {
                let ( $(ref $name,)* ) = *self;
                ($($name.generate(ctx),)*)
            }
        }

        impl <$($name: Arbitrary),*> Arbitrary for ($($name,)*) {
            type ArbitraryGenerator = ($($name::ArbitraryGenerator,)*);

            #[inline]
            fn arbitrary() -> Self::ArbitraryGenerator {
                ($($name::arbitrary(),)*)
            }
        }

        // impl <T: Into<TestResult>, $($name: Arbitrary),*> IntoTestable for fn($($name),*) -> T {
        //     type Testable = FnTestable<($($name::ArbitraryGenerator,)*), fn($($name),*) -> T>;
        //
        //     #[inline]
        //     #[allow(unused_variables, non_snake_case)]
        //     fn into_testable(self) -> Self::Testable {
        //         let generator = ($($name::arbitrary(),)*);
        //         for_all(generator).property(self)
        //     }
        // }
    }
}

fn_impls!{}
fn_impls!{A}
fn_impls!{A, B}
fn_impls!{A, B, C}
fn_impls!{A, B, C, D}
fn_impls!{A, B, C, D, E}
fn_impls!{A, B, C, D, E, F}
fn_impls!{A, B, C, D, E, F, G}
fn_impls!{A, B, C, D, E, F, G, H}
fn_impls!{A, B, C, D, E, F, G, H, I}
fn_impls!{A, B, C, D, E, F, G, H, I, J}
fn_impls!{A, B, C, D, E, F, G, H, I, J, K}
fn_impls!{A, B, C, D, E, F, G, H, I, J, K, L}

macro_rules! property {
    (($($arg_name:ident : $arg_type:ty),*) -> $output:ty {$($stmt:stmt)*}) => {{
        #[allow(dead_code)]
        fn property($($arg_name: $arg_type),*) -> $output { $($stmt)* }
        property as fn($($arg_type),*) -> $output
    }}
}

pub fn main() {
    qckchk(TestResult { status: Status::Pass });

    let simple_prop = property!{ () -> TestResult {
        TestResult { status: Status::Pass }
    }};

    qckchk(simple_prop);

    qckchk(property!{ (_x: usize) -> TestResult {
        TestResult { status: Status::Pass }
    }});

    let my_prop2 = property! { (_x: usize) -> u8 { 3u8 } };
    qckchk(my_prop2);

    let _ = qckchk(6u8);

    let generator = (Constant(32usize),);
    qckchk(for_all(generator).property(my_prop2 as fn(usize) -> u8));

    fn my_prop3(_: usize, _: usize) -> u8 {
        3u8
    }
    qckchk(for_all((Constant(32usize), Constant(42usize))).property(my_prop3 as fn(usize,usize) -> u8));

    fn my_predicate(s: usize) -> bool {
        s > 5
    }

    qckchk(when(my_predicate as fn(usize) -> bool).property(my_prop2 as fn(usize) -> u8));
}
