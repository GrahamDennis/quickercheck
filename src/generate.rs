use rand;

#[derive(Copy, Clone)]
pub struct GenerateCtx<R> {
    pub rng: R,
    pub size: usize
}

pub trait Generator {
    type Output;

    fn generate<R: rand::Rng>(&self, &mut GenerateCtx<R>) -> <Self as Generator>::Output;
}

impl <'a, G: Generator> Generator for &'a G {
    type Output = G::Output;

    #[inline]
    fn generate<R: rand::Rng>(&self, ctx: &mut GenerateCtx<R>) -> Self::Output {
        (*self).generate(ctx)
    }
}

#[derive(Copy, Clone)]
pub struct Constant<T>(pub T);

impl <T: Clone> Generator for Constant<T> {
    type Output = T;

    #[inline]
    fn generate<R: rand::Rng>(&self, _: &mut GenerateCtx<R>) -> T {
        self.0.clone()
    }
}

macro_rules! fn_impls {
    ($($name:ident),*) => {
        impl <$($name: Generator),*> Generator for ($($name,)*) {
            type Output = ($($name::Output,)*);

            #[inline]
            #[allow(unused_variables, non_snake_case)]
            fn generate<R: rand::Rng>(&self, ctx: &mut GenerateCtx<R>) -> Self::Output {
                let ( $(ref $name,)* ) = *self;
                ($($name.generate(ctx),)*)
            }
        }
    }
}

macro_tuples_impl!{fn_impls}
