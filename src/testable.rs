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

fn qckchk<T: IntoTestable>(t: T) -> CheckResult
{
    let testable = t.into_testable();
    let mut ctx = GenerateCtx { rng: rand::thread_rng(), size: 5 };
    let test_result: TestResult = testable.test(&mut ctx);
    CheckResult
}

macro_rules! fn_impls {
    ($($name:ident),*) => {
        impl <T: Into<TestResult>, $($name: Arbitrary),*> Testable for fn($($name),*) -> T {
            #[inline]
            #[allow(unused_variables, non_snake_case)]
            fn test<R: rand::Rng>(&self, ctx: &mut GenerateCtx<R>) -> TestResult {
                let ( $($name,)* ) = ($($name::arbitrary().generate(ctx),)*);
                self($($name),*).into()
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

    fn my_prop(_: usize) -> TestResult {
        TestResult { status: Status::Pass }
    }
    qckchk(my_prop as fn(usize) -> TestResult);

    fn my_prop2(_: usize) -> u8 {
        3u8
    }
    qckchk(my_prop2 as fn(usize) -> u8);

    let _ = qckchk(6u8);
}
