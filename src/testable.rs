use generate::GenerateCtx;
use shrink;
use arbitrary::Arbitrary;
use property::{Property, ForAllProperty};
use rose::Rose;

use std::convert::{Into, From};
use std::fmt::Debug;

use rand::Rng;

#[derive(Clone)]
pub struct TestResult {
    pub input: String,
    pub status: TestStatus,
}

#[derive(Copy, Clone, Debug)]
pub enum TestStatus { Pass, Fail, Discard }

pub trait Testable {
    fn test<R: Rng>(&self, ctx: &mut GenerateCtx<R>) -> Box<Rose<TestResult>>;
    fn is_expected_to_fail(&self) -> bool {
        false
    }
}

impl <'a, T: Testable> Testable for &'a T {
    #[inline]
    fn test<R: Rng>(&self, ctx: &mut GenerateCtx<R>) -> Box<Rose<TestResult>> {
        (*self).test(ctx)
    }
}

pub trait IntoTestable {
    type Testable: Testable;

    fn into_testable(self) -> Self::Testable;

    fn resize<F: Fn(usize) -> usize>(self, resize: F) -> ResizedTestable<Self::Testable, F>
        where Self: Sized
    {
        ResizedTestable { testable: self.into_testable(), resize: resize }
    }

    fn expect_failure(self) -> FailureExpectedTestable<Self::Testable>
        where Self: Sized
    {
        FailureExpectedTestable(self.into_testable())
    }
}

pub struct ResizedTestable<T, F> {
    testable: T,
    resize: F
}

impl <T, F> Testable for ResizedTestable<T, F>
    where T: Testable,
          F: Fn(usize) -> usize
{
    fn test<R: Rng>(&self, ctx: &mut GenerateCtx<R>) -> Box<Rose<TestResult>> {
        let new_size = (self.resize)(ctx.size);
        let mut new_ctx = GenerateCtx::new(ctx.rng, new_size);
        self.testable.test(&mut new_ctx)
    }
}

pub struct FailureExpectedTestable<T>(T);

impl <T: Testable> Testable for FailureExpectedTestable<T> {
    fn test<R: Rng>(&self, ctx: &mut GenerateCtx<R>) -> Box<Rose<TestResult>> { self.0.test(ctx) }
    fn is_expected_to_fail(&self) -> bool { true }
}

impl <T: Testable> IntoTestable for T {
    type Testable = Self;

    #[inline]
    fn into_testable(self) -> Self { self }
}

impl From<bool> for TestStatus {
    #[inline]
    fn from(success: bool) -> TestStatus {
        if success { TestStatus::Pass } else { TestStatus::Fail }
    }
}

impl <T: Into<TestStatus>, Err> From<Result<T, Err>> for TestStatus {
    #[inline]
    fn from(result: Result<T, Err>) -> TestStatus {
        match result {
            Ok(t) => t.into(),
            Err(_) => TestStatus::Fail
        }
    }
}

impl <'a, T: Into<TestStatus> + Clone, Err> From<&'a Result<T, Err>> for TestStatus {
    #[inline]
    fn from(result: &'a Result<T, Err>) -> TestStatus {
        match *result {
            Ok(ref t) => t.clone().into(),
            Err(_) => TestStatus::Fail
        }
    }
}

impl From<()> for TestStatus {
    #[inline]
    fn from(_: ()) -> TestStatus {
        TestStatus::Pass
    }
}

macro_rules! fn_impls {
    ($($name:ident),*) => {
        impl <Output: Into<TestStatus>, $($name: Arbitrary + Debug),*> IntoTestable for fn($($name),*) -> Output
        {
            type Testable = ForAllProperty<($($name,)*), <($($name,)*) as Arbitrary>::Generator, shrink::Empty<($($name,)*)>, Self>;

            #[inline]
            fn into_testable(self) -> Self::Testable {
                Property::<($($name,)*)>::new(self)
            }
        }
    }
}

macro_tuples_impl! {fn_impls}

#[cfg(test)]
mod tests {
    use super::*;
    use generate::Constant;
    use property::Property;
    use quick_check::quickcheck;

    #[test]
    fn property_is_testable() {
        let simple_prop = Property::<()>::new(|| TestStatus::Pass );
        quickcheck(simple_prop);
    }

    #[test]
    fn property_is_testable_by_reference() {
        let simple_prop = Property::<()>::new(|| TestStatus::Pass );
        quickcheck(&simple_prop);
    }

    #[test]
    fn property_with_args_is_testable() {
        let my_prop = Property::<(usize,)>::new(|_| TestStatus::Pass );
        quickcheck(&my_prop);
    }

    #[test]
    fn property_returning_non_test_result_is_testable() {
        let my_prop2 = Property::<(usize,)>::new(|_| true );
        quickcheck(&my_prop2);
    }

    #[test]
    fn for_all_is_testable() {
        let for_all_prop = Property::<(usize,)>
            ::for_all((Constant(32usize),))
            .property(|_| true);
        quickcheck(&for_all_prop);
    }

    #[test]
    fn for_all_with_two_arguments_is_testable() {
        let my_prop3 = Property::<(usize, usize)>
            ::for_all((Constant(32usize), Constant(42usize)))
            .property(|_, _| true);
        quickcheck(&my_prop3);
    }

    #[test]
    fn when_property_is_testable() {
        let predicate_prop = Property::<(usize,)>
            ::when(|s| s > 5)
            .property(|_| true);
        quickcheck(&predicate_prop);
    }

    #[test]
    fn cast_fn_is_testable() {
        fn my_fn() -> bool { true }
        quickcheck(my_fn as fn() -> bool);

        fn my_fn2(_:usize) -> bool { true }
        quickcheck(my_fn2 as fn(usize) -> bool);
    }

    #[test]
    fn cast_fn_is_testable2() {
        fn simple_prop() -> TestStatus { TestStatus::Pass };
        quickcheck(simple_prop as fn() -> TestStatus);
    }
}
