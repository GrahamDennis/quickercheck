use arbitrary::Arbitrary;
use generate::{Generator, GenerateCtx};
use shrink::{self, Shrink};
use testable::{Testable, TestResult, TestStatus};
use quick_fn::QuickFn;
use rose::{Rose, GenerateWithRose, RoseTraitMap};

use std::marker::PhantomData;
use std::fmt::Debug;
use std::rc::Rc;
use rand::Rng;

#[derive(Copy, Clone)]
pub struct Property<Args> {
    _marker: PhantomData<Args>
}

#[derive(Clone)]
pub struct ForAllProperty<Args, G, S, F> {
    generator: G,
    shrinker: S,
    f: Rc<F>,
    _marker: PhantomData<Args>
}

#[derive(Copy, Clone)]
pub struct ForAll<Args, G, S> {
    generator: G,
    shrinker: S,
    _marker: PhantomData<Args>
}

#[derive(Copy, Clone)]
pub struct When<Args, P> {
    predicate: P,
    _marker: PhantomData<Args>
}

#[derive(Clone)]
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
    type Shrink = shrink::Empty<Self>;

    fn arbitrary() -> Self::Generator {
        QuickFnArgs(A::arbitrary())
    }

    #[inline] fn shrink() -> Self::Shrink { shrink::Empty::empty() }
}

impl <G, S, T, F, Args: Debug + 'static> Testable for ForAllProperty<QuickFnArgs<Args>, G, S, F>
    where G: Generator<Output=Args>,
          S: Shrink<Item=Args> + Clone + 'static,
          <S as Shrink>::Iterator: 'static,
          F: QuickFn<Args, Output=T> + 'static,
          T: Into<TestStatus>
{
    #[inline]
    fn test<R: Rng>(&self, ctx: &mut GenerateCtx<R>) -> Rose<TestResult> {
        let args = self.generator.generate(ctx);
        let shrinker = self.shrinker.clone();

        GenerateWithRose::new(
            args,
            shrinker,
            |shrinker, args| Box::new(shrinker.shrink(&args))
        ).scan(
            self.f.clone(),
            |f, args|
                TestResult {
                    input: format!("{:?}", &args),
                    status: f.call(args).into()
                }
        )
     }
}

impl <G, S, Args> ForAll<QuickFnArgs<Args>, G, S> {
    #[inline]
    pub fn property<F, T>(self, f: F) -> ForAllProperty<QuickFnArgs<Args>, G, S, F>
        where F: QuickFn<Args, Output=T>,
              S: Shrink<Item=Args>,
              T: Into<TestStatus>,
              G: Generator<Output=Args>
    {
        ForAllProperty {
            generator: self.generator,
            shrinker: self.shrinker,
            f: Rc::new(f),
            _marker: PhantomData
        }
    }
}

impl <Args: Arbitrary> Property<QuickFnArgs<Args>> {
    pub fn new<F, T>(f: F) -> ForAllProperty<QuickFnArgs<Args>, Args::Generator, Args::Shrink, F>
        where F: QuickFn<Args, Output=T>,
              T: Into<TestStatus>
    {
        Property::<QuickFnArgs<Args>>::for_all_shrink(<Args>::arbitrary(), <Args>::shrink()).property(f)
    }

    pub fn for_all<G: Generator<Output=Args>>(g: G) -> ForAll<QuickFnArgs<Args>, G, shrink::Empty<Args>> {
        Property::<QuickFnArgs<Args>>::for_all_shrink(g, shrink::Empty::empty())
    }

    pub fn for_all_shrink<G, S>(g: G, s: S) -> ForAll<QuickFnArgs<Args>, G, S>
        where G: Generator<Output=Args>,
              S: Shrink<Item=Args>
    {
        ForAll {
            generator: g,
            shrinker: s,
            _marker: PhantomData
        }
    }

    pub fn when<P: QuickFn<Args, Output=bool>>(p: P) -> When<QuickFnArgs<Args>, P> {
        When { predicate: p, _marker: PhantomData }
    }
}

macro_rules! fn_impls {
    ($($ident:ident),*) => {
        impl <G, S, T, F, $($ident: Debug),*> Testable for ForAllProperty<($($ident,)*), G, S, F>
            where G: Generator<Output=($($ident,)*)>,
                  S: Shrink<Item=($($ident,)*)>,
                  F: Fn($($ident),*) -> T,
                  T: Into<TestStatus>
        {
            #[inline]
            #[allow(non_snake_case)]
            fn test<R: Rng>(&self, ctx: &mut GenerateCtx<R>) -> Rose<TestResult> {
                let args = self.generator.generate(ctx);
                self.rose_from_args(args)
            }
        }

        impl <G, S, T, F, $($ident: Debug),*> ForAllProperty<($($ident,)*), G, S, F>
            where G: Generator<Output=($($ident,)*)>,
                  S: Shrink<Item=($($ident,)*)>,
                  F: Fn($($ident),*) -> T,
                  T: Into<TestStatus>
        {
            #[inline]
            #[allow(non_snake_case)]
            fn rose_from_args(&self, args: ($($ident,)*)) -> Rose<TestResult> {
                let ($($ident,)*) = args;

                Rose::single(
                    TestResult {
                        input: format!("{:?}", ($(&$ident,)*)),
                        status: (self.f)($($ident),*).into()
                    }
                )
                //     ,
                //     self.shrinker.shrink(&args)
                //         .map(|shrunk_args| {
                //             self.rose_from_args(shrunk_args)
                //         })
                // )
            }
        }

        impl <G, S, $($ident),*> ForAll<($($ident,)*), G, S> {
            #[inline]
            pub fn property<F, T>(self, f: F) -> ForAllProperty<($($ident,)*), G, S, F>
                where F: Fn($($ident),*) -> T,
                      T: Into<TestStatus>,
                      G: Generator<Output=($($ident,)*)>
            {
                ForAllProperty {
                    generator: self.generator,
                    shrinker: self.shrinker,
                    f: Rc::new(f),
                    _marker: PhantomData
                }
            }
        }

        impl <$($ident: Arbitrary),*> Property<($($ident,)*)> {
            pub fn new<F, T>(f: F) -> ForAllProperty<
                                            ($($ident,)*),
                                            <($($ident,)*) as Arbitrary>::Generator,
                                            <($($ident,)*) as Arbitrary>::Shrink,
                                            F>
                where F: Fn($($ident),*) -> T,
                      T: Into<TestStatus>
            {
                Property::<($($ident,)*)>::for_all_shrink(
                    <($($ident,)*) as Arbitrary>::arbitrary(),
                    <($($ident,)*) as Arbitrary>::shrink()
                ).property(f)
            }

            pub fn for_all<G>(g: G) -> ForAll<($($ident,)*), G, shrink::Empty<($($ident,)*)>>
                where G: Generator<Output=($($ident,)*)>
            {
                Property::<($($ident,)*)>::for_all_shrink(g, shrink::Empty::empty())
            }

            pub fn for_all_shrink<G, S>(g: G, s: S) -> ForAll<($($ident,)*), G, S>
                where G: Generator<Output=($($ident,)*)>,
                      S: Shrink<Item=($($ident,)*)>
            {
                ForAll {
                    generator: g,
                    shrinker: s,
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
                                  shrink::Empty<($($ident,)*)>,
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
