use rand;

pub struct GenerateCtx<'a, R: ?Sized + 'a> {
    pub rng: &'a mut R,
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

macro_rules! tuple_impls {
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

macro_tuples_impl!{tuple_impls}

pub struct FnGenerator<T> {
    f: Box<Fn(&mut GenerateCtx<rand::Rng>) -> T>
}

impl <T> Generator for FnGenerator<T> {
    type Output = T;

    fn generate<R: rand::Rng>(&self, ctx: &mut GenerateCtx<R>) -> T {
        let mut boxed_ctx = GenerateCtx { rng: ctx.rng as &mut rand::Rng, size: ctx.size };
        (self.f)(&mut boxed_ctx)
    }
}

impl <T> FnGenerator<T> {
    fn new<F: Fn(&mut GenerateCtx<rand::Rng>) -> T + 'static>(f: F) -> FnGenerator<T> {
        FnGenerator { f: Box::new(f) }
    }
}

fn testy() -> FnGenerator<u32> {
    FnGenerator::new(|ctx| ctx.rng.next_u32())
}
