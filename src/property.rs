use arbitrary::Arbitrary;
use generate::{Generator, GenerateCtx};
use testable::{Testable, TestResult, TestStatus};
use quick_fn::QuickFn;

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

#[derive(Copy, Clone)]
pub struct ForAll<Args, G> {
    generator: G,
    _marker: PhantomData<Args>
}

#[derive(Copy, Clone)]
pub struct When<Args, P> {
    predicate: P,
    _marker: PhantomData<Args>
}

#[derive(Copy, Clone)]
pub struct WhenFn<Args, P, F> {
    predicate: P,
    f: F,
    _marker: PhantomData<Args>
}

#[derive(Copy, Clone)]
pub struct QuickFnArgs<A>(A);

impl <A: Generator> Generator for QuickFnArgs<A> {
    type Output = QuickFnArgs<A::Output>;

    #[inline]
    fn generate<R: Rng>(&self, ctx: &mut GenerateCtx<R>) -> <Self as Generator>::Output {
        QuickFnArgs(self.0.generate(ctx))
    }
}

impl <A: Arbitrary> Arbitrary for QuickFnArgs<A> {
    type Generator = QuickFnArgs<A::Generator>;

    fn arbitrary() -> Self::Generator {
        QuickFnArgs(A::arbitrary())
    }
}

impl <G, T, F, Args> Testable for ForAllProperty<QuickFnArgs<Args>, G, F>
    where G: Generator<Output=Args>,
          F: QuickFn<Args, Output=T>,
          T: Into<TestStatus>
{
    #[inline]
    fn test<R: Rng>(&self, ctx: &mut GenerateCtx<R>) -> TestResult {
        let args = self.generator.generate(ctx);
        TestResult {
            input: format!("{:?}", ()),
            status: self.f.call(args).into()
        }
    }
}

impl <G, Args> ForAll<QuickFnArgs<Args>, G> {
    #[inline]
    pub fn property<F, T>(self, f: F) -> ForAllProperty<QuickFnArgs<Args>, G, F>
        where F: QuickFn<Args, Output=T>,
              T: Into<TestStatus>,
              G: Generator<Output=Args>
    {
        ForAllProperty {
            generator: self.generator,
            f: f,
            _marker: PhantomData
        }
    }
}

impl <Args: Arbitrary> Property<QuickFnArgs<Args>> {
    pub fn new<F, T>(f: F) -> ForAllProperty<QuickFnArgs<Args>, Args::Generator, F>
        where F: QuickFn<Args, Output=T>,
              T: Into<TestStatus>
    {
        Property::<QuickFnArgs<Args>>::for_all(<Args>::arbitrary()).property(f)
    }

    pub fn for_all<G: Generator<Output=Args>>(g: G) -> ForAll<QuickFnArgs<Args>, G> {
        ForAll {
            generator: g,
            _marker: PhantomData
        }
    }

    pub fn when<P: QuickFn<Args, Output=bool>>(p: P) -> When<QuickFnArgs<Args>, P> {
        When { predicate: p, _marker: PhantomData }
    }
}

macro_rules! fn_impls {
    ($($ident:ident),*) => {
        impl <G, T, F, $($ident),*> Testable for ForAllProperty<($($ident,)*), G, F>
            where G: Generator<Output=($($ident,)*)>,
                  F: Fn($($ident),*) -> T,
                  T: Into<TestStatus>
        {
            #[inline]
            #[allow(non_snake_case)]
            fn test<R: Rng>(&self, ctx: &mut GenerateCtx<R>) -> TestResult {
                let args = self.generator.generate(ctx);
                let ($($ident,)*) = args;
                TestResult {
                    input: format!("{:?}", ()),
                    status: (self.f)($($ident),*).into()
                }
            }
        }

        impl <G, $($ident),*> ForAll<($($ident,)*), G> {
            #[inline]
            pub fn property<F, T>(self, f: F) -> ForAllProperty<($($ident,)*), G, F>
                where F: Fn($($ident),*) -> T,
                      T: Into<TestStatus>,
                      G: Generator<Output=($($ident,)*)>
            {
                ForAllProperty {
                    generator: self.generator,
                    f: f,
                    _marker: PhantomData
                }
            }
        }

        impl <$($ident: Arbitrary),*> Property<($($ident,)*)> {
            pub fn new<F, T>(f: F) -> ForAllProperty<($($ident,)*), <($($ident,)*) as Arbitrary>::Generator, F>
                where F: Fn($($ident),*) -> T,
                      T: Into<TestStatus>
            {
                Property::<($($ident,)*)>::for_all(<($($ident,)*)>::arbitrary()).property(f)
            }

            pub fn for_all<G: Generator<Output=($($ident,)*)>>(g: G) -> ForAll<($($ident,)*), G> {
                ForAll {
                    generator: g,
                    _marker: PhantomData
                }
            }

            pub fn when<P: Fn($($ident),*) -> bool>(p: P) -> When<($($ident,)*), P> {
                When { predicate: p, _marker: PhantomData }
            }
        }

        impl <P, F, T, $($ident: Clone),*> QuickFn<($($ident,)*)> for WhenFn<($($ident,)*), P, F>
            where P: Fn($($ident),*) -> bool,
                  F: Fn($($ident),*) -> T,
                  T: Into<TestStatus>
        {
            type Output = TestStatus;

            #[inline]
            #[allow(non_snake_case)]
            fn call(&self, args: ($($ident,)*)) -> Self::Output {
                let ($($ident,)*) = args;
                match (self.predicate)($($ident.clone()),*) {
                    false => TestStatus::Discard,
                    true  => (self.f)($($ident),*).into()
                }
            }
        }


        impl <P, $($ident: Arbitrary + Clone),*> When<($($ident,)*), P> {
            #[inline]
            pub fn property<F, T>(self, f: F)
                -> ForAllProperty<QuickFnArgs<($($ident,)*)>,
                                  <($($ident,)*) as Arbitrary>::Generator,
                                  WhenFn<($($ident,)*), P, F>>
                where P: Fn($($ident),*) -> bool,
                      F: Fn($($ident),*) -> T,
                      T: Into<TestStatus>
            {
                Property::<QuickFnArgs<($($ident,)*)>>::new(WhenFn {
                    predicate: self.predicate,
                    f: f,
                    _marker: PhantomData
                })
            }
        }
    }
}

macro_tuples_impl! { fn_impls }

#[cfg(test)]
mod tests {
    use super::*;
    use arbitrary::Arbitrary;

    #[test]
    fn test_simple_property() {
        Property::<()>::new(|| false);
    }

    #[test]
    fn test_for_all_property() {
        Property::<()>::for_all(<()>::arbitrary()).property(|| true);
    }

    #[test]
    fn test_when_property() {
        Property::<()>::when(|| false).property(|| true);
    }

    #[test]
    fn test_generic_property() {
        fn property<A>(_: A) -> bool {
            ::std::mem::size_of::<A>() == 0
        }

        Property::<(usize,)>::new(property);
        // Doesn't type check
        // Property::new(property as fn(usize) -> bool);
    }
}
