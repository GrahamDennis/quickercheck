use quick_fn::QuickFn;
use arbitrary::Arbitrary;
use generate::{Generator, GenerateCtx};
use testable::{IntoTestable, Testable, TestResult, Status};

use std::marker::PhantomData;
use rand::Rng;

#[derive(Copy, Clone)]
pub struct Property<Args> {
    _marker: PhantomData<Args>
}

#[derive(Copy, Clone)]
pub struct ForAllProperty<Args, G, F> {
    generator: G,
    f: F,
    _marker: PhantomData<Args>
}

impl <G, F, T> Testable for ForAllProperty<G::Output, G, F>
    where G: Generator,
          F: QuickFn<G::Output, Output=T>,
          T: Into<TestResult>
{
    #[inline]
    fn test<R: Rng>(&self, ctx: &mut GenerateCtx<R>) -> TestResult {
        let args = self.generator.generate(ctx);
        self.f.call(args).into()
    }
}

impl <Args, G, F> QuickFn<Args> for ForAllProperty<Args, G, F>
    where F: QuickFn<Args>
{
    type Output = F::Output;

    #[inline]
    fn call(&self, args: Args) -> Self::Output {
        self.f.call(args)
    }
}

#[derive(Copy, Clone)]
pub struct ForAll<Args, G> {
    generator: G,
    _marker: PhantomData<Args>
}

impl <Args, G> ForAll<Args, G> {
    #[inline]
    pub fn property<F: QuickFn<Args>>(self, f: F) -> ForAllProperty<Args, G, F>
        where F::Output: Into<TestResult>,
              G: Generator<Output=Args>
    {
        ForAllProperty {
            generator: self.generator,
            f: f,
            _marker: PhantomData
        }
    }
}

#[derive(Copy, Clone)]
pub struct When<Args, P> {
    predicate: P,
    _marker: PhantomData<Args>
}

pub fn when<Args, P>(p: P) -> When<Args, P>
    where P: QuickFn<Args, Output=bool>
{
    When { predicate: p, _marker: PhantomData }
}

#[derive(Copy, Clone)]
struct WhenFn<Args, P, F> {
    predicate: P,
    f: F,
    _marker: PhantomData<Args>
}

impl <Args, P> When<Args, P> {
    #[inline]
    pub fn then<F: QuickFn<Args>>(self, f: F)
        -> ForAllProperty<Args, Args::ArbitraryGenerator, WhenFn<Args, P, F>>
        where Args: Arbitrary,
              P: QuickFn<Args, Output=bool>,
              WhenFn<Args, P, F>: QuickFn<Args>,
              <WhenFn<Args, P, F> as QuickFn<Args>>::Output: Into<TestResult>
    {
        Property::<Args>::new(WhenFn {
            predicate: self.predicate,
            f: f,
            _marker: PhantomData
        })
    }
}

impl <Args, P, F> IntoTestable for WhenFn<Args, P, F>
    where Args: Arbitrary,
          WhenFn<Args, P, F>: QuickFn<Args>,
          <WhenFn<Args, P, F> as QuickFn<Args>>::Output: Into<TestResult>
{
    type Testable = ForAllProperty<Args, Args::ArbitraryGenerator, Self>;

    #[inline]
    fn into_testable(self) -> Self::Testable {
        Property::<Args>::new(self)
    }
}

impl <Args, P, F> QuickFn<Args> for WhenFn<Args, P, F>
    where Args: Clone,
          P: QuickFn<Args, Output=bool>,
          F: QuickFn<Args>,
          F::Output: Into<TestResult>
{
    type Output = TestResult;

    #[inline]
    fn call(&self, args: Args) -> Self::Output {
        let fn_args = args.clone();
        match self.predicate.call(args) {
            false => TestResult { status: Status::Discard },
            true  => self.f.call(fn_args).into()
        }
    }
}

impl <Args> Property<Args> {
    pub fn new<F: QuickFn<Args>>(f: F) -> ForAllProperty<Args, Args::ArbitraryGenerator, F>
        where Args: Arbitrary,
              F::Output: Into<TestResult>
    {
        Property::<Args>::for_all(Args::arbitrary()).property(f)
    }

    pub fn for_all<G: Generator<Output=Args>>(g: G) -> ForAll<Args, G> {
        ForAll {
            generator: g,
            _marker: PhantomData
        }
    }

    pub fn when<P: QuickFn<Args, Output=bool>>(p: P) -> When<Args, P>
        where Args: Arbitrary
    {
        when(p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arbitrary::Arbitrary;

    #[test]
    fn test_simple_property() {
        Property::<()>::property(|| false);
    }

    #[test]
    #[cfg(feature = "no_function_casts")]
    fn test_simple_property_unstable() {
        Property::property(|| false);
    }

    #[test]
    fn test_for_all_property() {
        Property::<()>::for_all(<()>::arbitrary()).property(|| true);
    }

    #[test]
    fn test_when_property() {
        Property::<()>::when(|| false).then(|| true);
    }

    #[test]
    fn test_nested_when_property() {
        Property::<()>::for_all(<()>::arbitrary()).property(when(|| true).then(|| false));
    }
}
