pub trait QuickFn<Args> {
    type Output;

    fn call(&self, args: Args) -> Self::Output;
}

#[cfg(not(feature = "no_function_casts"))]
mod stable {
    use super::QuickFn;

    macro_rules! fn_impls {
        ($($ident: ident),*) => {
            impl <Output, Function, $($ident),*> QuickFn<($($ident,)*)> for Function
                where Function: Fn($($ident),*) -> Output
            {
                type Output = Output;

                #[inline]
                #[allow(unused_variables, non_snake_case)]
                fn call(&self, args: ($($ident,)*)) -> Self::Output {
                    let ( $($ident,)* ) = args;
                    self($($ident),*)
                }
            }
        }
    }

    macro_tuples_impl!{fn_impls}
}

#[cfg(feature = "no_function_casts")]
mod unstable {
    use super::QuickFn;

    impl <F: Fn<Args>, Args> QuickFn<Args> for F {
        type Output = F::Output;

        #[inline]
        fn call(&self, args: Args) -> Self::Output {
            self.call(args)
        }
    }
}
