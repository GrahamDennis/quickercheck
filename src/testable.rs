use generate::GenerateCtx;
use arbitrary::Arbitrary;
use property::{Property, ForAllProperty};

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

macro_rules! fn_impls {
    ($($name:ident),*) => {
        impl <Output: Into<TestResult>, $($name: Arbitrary),*> IntoTestable for fn($($name),*) -> Output
        {
            type Testable = ForAllProperty<($($name,)*), <($($name,)*) as Arbitrary>::ArbitraryGenerator, Self>;

            #[inline]
            fn into_testable(self) -> Self::Testable {
                Property::<($($name,)*)>::new(self)
            }
        }
    }
}

macro_tuples_impl! {fn_impls}

#[derive(Copy, Clone)]
pub struct CheckResult;

pub fn qckchk<T: IntoTestable>(t: T) -> CheckResult
{
    let testable = t.into_testable();
    let mut ctx = GenerateCtx { rng: rand::thread_rng(), size: 5 };
    let _test_result = testable.test(&mut ctx);
    CheckResult
}

#[cfg(test)]
mod tests {
    use super::*;
    use generate::Constant;
    use property::Property;

    #[test]
    fn test_result_is_testable() {
        qckchk(TestResult { status: Status::Pass });
    }

    #[test]
    fn into_test_result_is_testable() {
        qckchk(6u8);
    }

    #[test]
    fn property_is_testable() {
        let simple_prop = Property::<()>::new(|| TestResult { status: Status::Pass });
        qckchk(simple_prop);
    }

    #[test]
    fn property_is_testable_by_reference() {
        let simple_prop = Property::<()>::new(|| TestResult { status: Status::Pass });
        qckchk(&simple_prop);
    }

    #[test]
    fn property_with_args_is_testable() {
        let my_prop = Property::<(usize,)>::new(|_| TestResult { status: Status::Pass });
        qckchk(&my_prop);
    }

    #[test]
    fn property_returning_non_test_result_is_testable() {
        let my_prop2 = Property::<(usize,)>::new(|_| 3u8 );
        qckchk(&my_prop2);
    }

    #[test]
    fn for_all_is_testable() {
        let for_all_prop = Property::<(usize,)>
            ::for_all((Constant(32usize),))
            .property(|_| 3u8);
        qckchk(&for_all_prop);
    }

    #[test]
    fn for_all_with_two_arguments_is_testable() {
        let my_prop3 = Property::<(usize, usize)>
            ::for_all((Constant(32usize), Constant(42usize)))
            .property(|_, _| 3u8);
        qckchk(&my_prop3);
    }

    #[test]
    fn when_property_is_testable() {
        let predicate_prop = Property::<(usize,)>
            ::when(|s| s > 5)
            .then(|_| 3u8);
        qckchk(&predicate_prop);
    }

    #[test]
    fn cast_fn_is_testable() {
        fn my_fn() -> bool { true }
        qckchk(my_fn as fn() -> bool);

        fn my_fn2(_:usize) -> bool { true }
        qckchk(my_fn2 as fn(usize) -> bool);
    }

    #[test]
    fn cast_fn_is_testable2() {
        fn simple_prop() -> TestResult { TestResult { status: Status::Pass }};
        qckchk(simple_prop as fn() -> TestResult);
    }
}
