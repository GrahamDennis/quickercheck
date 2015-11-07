use generate::{Generator, GenerateCtx, Constant, Map};
use arbitrary::Arbitrary;
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

impl <T: Testable> IntoTestable for T {
    type Testable = Self;

    fn into_testable(self) -> Self {
        self
    }
}

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

trait MyFn<Args> {
    type Output;

    fn call(&self, Args) -> Self::Output;
}

pub struct FnTestable<G, F> {
    generator: G,
    f: F
}

impl <G: Generator> ForAll<G> {
    fn property<T, F>(self, f: F) -> FnTestable<G, F>
        where F: MyFn<G::Output, Output=T>,
              T: Into<TestResult>
    {
        FnTestable { generator: self.0, f: f }
    }
}

impl <G, F, T> Testable for FnTestable<G, F>
    where G: Generator,
          F: MyFn<G::Output, Output=T>,
          T: Into<TestResult>
{
    #[inline]
    fn test<R: rand::Rng>(&self, ctx: &mut GenerateCtx<R>) -> TestResult {
        let args = self.generator.generate(ctx);
        self.f.call(args).into()
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

        impl <T, $($name),*> MyFn<($($name,)*)> for fn($($name),*) -> T {
            type Output = T;

            #[inline]
            #[allow(unused_variables, non_snake_case)]
            fn call(&self, args: ($($name,)*)) -> T {
                let ( $($name,)* ) = args;
                self($($name),*)
            }
        }

        impl <T: Into<TestResult>, $($name: Arbitrary),*> IntoTestable for fn($($name),*) -> T {
            type Testable = FnTestable<($($name::ArbitraryGenerator,)*), fn($($name),*) -> T>;

            #[inline]
            #[allow(unused_variables, non_snake_case)]
            fn into_testable(self) -> Self::Testable {
                let generator = ($($name::arbitrary(),)*);
                for_all(generator).property(self)
            }
        }
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

pub fn main() {
    qckchk(TestResult { status: Status::Pass });

    fn simple_prop() -> TestResult {
        TestResult { status: Status::Pass }
    }
    qckchk(simple_prop as fn() -> TestResult);

    fn my_prop(_: usize) -> TestResult {
        TestResult { status: Status::Pass }
    }
    qckchk(my_prop as fn(usize) -> TestResult);

    fn my_prop2(_: usize) -> u8 {
        3u8
    }
    qckchk(my_prop2 as fn(usize) -> u8);

    let _ = qckchk(6u8);

    qckchk(for_all((Constant(32usize),)).property(my_prop2 as fn(usize) -> u8));
}
