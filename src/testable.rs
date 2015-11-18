use generate::GenerateCtx;
use arbitrary::Arbitrary;
use property::{Property, ForAllProperty};

use std::convert::{Into, From};

use rand::Rng;

#[derive(Copy, Clone)]
pub struct TestResult {
    pub status: Status,
}

impl TestResult {
    pub fn passed() ->  TestResult { TestResult { status: Status::Pass } }
    pub fn failed() ->  TestResult { TestResult { status: Status::Fail } }
    pub fn discard() -> TestResult { TestResult { status: Status::Discard } }
}

#[derive(Copy, Clone)]
pub enum Status { Pass, Fail, Discard }

pub trait Testable {
    fn test<R: Rng>(&self, ctx: &mut GenerateCtx<R>) -> TestResult;
}

impl <'a, T: Testable> Testable for &'a T {
    #[inline]
    fn test<R: Rng>(&self, ctx: &mut GenerateCtx<R>) -> TestResult {
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
}

pub struct ResizedTestable<T, F> {
    testable: T,
    resize: F
}

impl <T, F> Testable for ResizedTestable<T, F>
    where T: Testable,
          F: Fn(usize) -> usize
{
    fn test<R: Rng>(&self, ctx: &mut GenerateCtx<R>) -> TestResult {
        let new_size = (self.resize)(ctx.size);
        let mut new_ctx = GenerateCtx::new(ctx.rng, new_size);
        self.testable.test(&mut new_ctx)
    }
}

impl <T: Testable> IntoTestable for T {
    type Testable = Self;

    #[inline]
    fn into_testable(self) -> Self { self }
}

impl Testable for TestResult {
    fn test<R: Rng>(&self, _: &mut GenerateCtx<R>) -> TestResult {
        *self
    }
}

impl Testable for bool {
    fn test<R: Rng>(&self, _: &mut GenerateCtx<R>) -> TestResult {
        (*self).into()
    }
}

impl From<bool> for TestResult {
    #[inline]
    fn from(success: bool) -> TestResult {
        if success { TestResult::passed() } else { TestResult::failed() }
    }
}

macro_rules! fn_impls {
    ($($name:ident),*) => {
        impl <Output: Into<TestResult>, $($name: Arbitrary),*> IntoTestable for fn($($name),*) -> Output
        {
            type Testable = ForAllProperty<($($name,)*), <($($name,)*) as Arbitrary>::Generator, Self>;

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
    fn test_result_is_testable() {
        quickcheck(TestResult { status: Status::Pass });
    }

    #[test]
    fn into_test_result_is_testable() {
        quickcheck(true);
    }

    #[test]
    fn property_is_testable() {
        let simple_prop = Property::<()>::new(|| TestResult { status: Status::Pass });
        quickcheck(simple_prop);
    }

    #[test]
    fn property_is_testable_by_reference() {
        let simple_prop = Property::<()>::new(|| TestResult { status: Status::Pass });
        quickcheck(&simple_prop);
    }

    #[test]
    fn property_with_args_is_testable() {
        let my_prop = Property::<(usize,)>::new(|_| TestResult { status: Status::Pass });
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
        fn simple_prop() -> TestResult { TestResult { status: Status::Pass }};
        quickcheck(simple_prop as fn() -> TestResult);
    }
}
