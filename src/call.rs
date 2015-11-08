use std::marker::PhantomData;

pub trait Call {
    type Input;
    type Output;

    fn call(&self, Self::Input) -> Self::Output;
}

#[cfg(not(feature = "no_function_casts"))]
mod stable {
    use super::Call;

    macro_rules! fn_impls {
        ($($ident: ident),*) => {
            impl <T, $($ident),*> Call for fn($($ident),*) -> T {
                type Input = ($($ident,)*);
                type Output = T;

                #[inline]
                #[allow(unused_variables, non_snake_case)]
                fn call(&self, args: ($($ident,)*)) -> T {
                    let ( $($ident,)* ) = args;
                    self($($ident),*)
                }
            }
        }
    }

    macro_rules! fn_impls_rec {
        ($first: ident $(, $ident: ident)* ) => {
            fn_impls! { $first $(, $ident)* }
            fn_impls_rec! { $($ident),* }
        };
        () => {
            fn_impls! {}
        }
    }

    fn_impls_rec!{A, B, C, D, E, F, G, H, I, J, K, L }
}

#[cfg(feature = "no_function_casts")]
mod unstable {
    // When this is available, we need to change most usages of 'Call' to 'IntoCall'
    use super::Call;
    use std::marker::PhantomData;

    struct MyCall<F, A> {
        f: F,
        _marker: PhantomData<A>
    }


    trait IntoCall<Args> {
        type Call: Call<Input=Args>;

        fn into_call(self) -> Self::Call;
    }

    impl <F, Args> Call for MyCall<F, Args>
        where F: Fn<Args>
    {
        type Input = Args;
        type Output = F::Output;

        #[inline]
        fn call(&self, args: Args) -> Self::Output {
            self.f.call(args)
        }
    }

    impl <F, Args> IntoCall<Args> for F
        where F: Fn<Args>
    {
        type Call = MyCall<Self, Args>;

        #[inline]
        fn into_call(self) -> Self::Call {
            MyCall { f: self, _marker: PhantomData }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn takes_into_call<Args, C: IntoCall<Args>>(_: C) {
        ;
    }

    #[test]
    fn test_into_call() {
        fn my_fn() -> u8 {
            8u8
        }

        takes_into_call::<(), _>(my_fn as fn() -> u8);
    }

    #[test]
    #[cfg(feature = "no_function_casts")]
    fn test_into_call_without_casts() {
        fn my_fn() -> u8 {
            8u8
        }
        fn my_fn2(_: usize) -> u8 {
            8u8
        }
        takes_into_call(my_fn);
        takes_into_call(my_fn2);
    }
}
