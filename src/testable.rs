use generate::{Generator, GenerateCtx, Constant};
use arbitrary::Arbitrary;
use property::{Property};

use std::convert::{Into, From};

use rand;

#[derive(Copy, Clone)]
pub struct TestResult {
    pub status: Status,
}

#[derive(Copy, Clone)]
pub enum Status { Pass, Fail, Discard }

pub trait Testable {
    fn test<R: rand::Rng>(&self, ctx: &mut GenerateCtx<R>) -> TestResult;
}

impl <'a, T: Testable> Testable for &'a T {
    #[inline]
    fn test<R: rand::Rng>(&self, ctx: &mut GenerateCtx<R>) -> TestResult {
        (*self).test(ctx)
    }
}

pub trait IntoTestable {
    type Testable: Testable;

    fn into_testable(self) -> Self::Testable;
}

impl <T: Testable> IntoTestable for T {
    type Testable = Self;

    #[inline]
    fn into_testable(self) -> Self { self }
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

impl From<bool> for TestResult {
    #[inline]
    fn from(success: bool) -> TestResult {
        TestResult { status: if success {Status::Pass} else {Status::Fail} }
    }
}

#[derive(Copy, Clone)]
pub struct CheckResult;

fn qckchk<T: IntoTestable>(t: T) -> CheckResult
{
    let testable = t.into_testable();
    let mut ctx = GenerateCtx { rng: rand::thread_rng(), size: 5 };
    let test_result = testable.test(&mut ctx);
    CheckResult
}

macro_rules! static_fn {
    (($($arg_name:ident : $arg_type:ty),*) -> $output:ty {$($stmt:stmt)*}) => {{
        #[allow(dead_code)]
        fn property($($arg_name: $arg_type),*) -> $output { $($stmt)* }
        property as fn($($arg_type),*) -> $output
    }}
}

pub fn main() {
    qckchk(TestResult { status: Status::Pass });

    let simple_prop = Property::<()>::new(|| TestResult { status: Status::Pass });
    qckchk(&simple_prop);

    let my_prop = Property::<(usize,)>::new(|_| TestResult { status: Status::Pass });
    qckchk(&my_prop);

    let my_prop2 = Property::<(usize,)>::new(|_| 3u8 );
    qckchk(&my_prop2);

    let _ = qckchk(6u8);

    let for_all_prop = Property::<(usize,)>
        ::for_all((Constant(32usize),))
        .property(|_| 3u8);
    qckchk(&for_all_prop);

    let my_prop3 = Property::<(usize, usize)>
        ::for_all((Constant(32usize), Constant(42usize)))
        .property(|_, _| 3u8);
    qckchk(&my_prop3);

    let predicate_prop = Property::<(usize,)>
        ::when(|s| s > 5)
        .then(|_| 3u8);
    qckchk(&predicate_prop);
}
