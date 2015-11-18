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

macro_rules! fn_impls {
    ($($ident:ident),*) => {
        impl <G, T, F, $($ident),*> Testable for ForAllProperty<($($ident,)*), G, F>
            where G: Generator<Output=($($ident,)*)>,
                  F: Fn($($ident),*) -> T,
                  T: Into<TestResult>
        {
            #[inline]
            #[allow(non_snake_case)]
            fn test<R: Rng>(&self, ctx: &mut GenerateCtx<R>) -> TestResult {
                let args = self.generator.generate(ctx);
                let ($($ident,)*) = args;
                (self.f)($($ident),*).into()
            }
        }

        impl <G, $($ident),*> ForAll<($($ident,)*), G> {
            #[inline]
            pub fn property<F, T>(self, f: F) -> ForAllProperty<($($ident,)*), G, F>
                where F: Fn($($ident),*) -> T,
                      T: Into<TestResult>,
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
                      T: Into<TestResult>
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

        // impl <P, $($ident: Arbitrary),*> When<($($ident,)*), P> {
        //     #[inline]
        //     pub fn property<F, T>(self, f: F)
        //         -> ForAllProperty<($($ident,)*), <($($ident,)*) as Arbitrary>::Generator, WhenFn<($($ident,)*), P, F>>
        //         where
        // }
    }
}

macro_tuples_impl! { fn_impls }

// impl <Args, P> When<Args, P> {
//     #[inline]
//     pub fn property<F: QuickFn<Args>>(self, f: F)
//         -> ForAllProperty<Args, Args::Generator, WhenFn<Args, P, F>>
//         where Args: Arbitrary,
//               P: QuickFn<Args, Output=bool>,
//               WhenFn<Args, P, F>: QuickFn<Args>,
//               <WhenFn<Args, P, F> as QuickFn<Args>>::Output: Into<TestResult>
//     {
//         Property::<Args>::new(WhenFn {
//             predicate: self.predicate,
//             f: f,
//             _marker: PhantomData
//         })
//     }
// }

// impl <Args, P, F> IntoTestable for WhenFn<Args, P, F>
//     where Args: Arbitrary,
//           WhenFn<Args, P, F>: QuickFn<Args>,
//           <WhenFn<Args, P, F> as QuickFn<Args>>::Output: Into<TestResult>
// {
//     type Testable = ForAllProperty<Args, Args::Generator, Self>;
//
//     #[inline]
//     fn into_testable(self) -> Self::Testable {
//         Property::<Args>::new(self)
//     }
// }

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

    // #[test]
    // fn test_when_property() {
    //     Property::<()>::when(|| false).property(|| true);
    // }

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

#[cfg(all(test, feature = "no_function_casts"))]
mod unstable_tests {
    use super::*;
    use arbitrary::Arbitrary;

    #[test]
    fn test_simple_property() {
        Property::new(|| false);
    }

    #[test]
    fn test_for_all_property() {
        Property::for_all(<()>::arbitrary()).property(|| true);
    }

    #[test]
    fn test_when_property() {
        Property::when(|| false).then(|| true);
    }

    #[test]
    fn test_nested_when_property() {
        Property::for_all(<()>::arbitrary()).property(when(|| true).property(|| false));
    }

    #[test]
    fn test_generic_property() {
        fn property<A>(_: A) -> bool {
            ::std::mem::size_of::<A>() == 0
        }

        Property::<(usize,)>::new(property);
        Property::new(property as fn(usize) -> bool);
    }
}
