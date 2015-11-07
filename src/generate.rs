use rand;

pub struct GenerateCtx<R> {
    pub rng: R,
    pub size: usize
}

pub trait Generator {
    type Output;

    fn generate<R: rand::Rng>(&self, &mut GenerateCtx<R>) -> <Self as Generator>::Output;

    #[inline]
    fn map<'a, F, T>(&'a self, f: F) -> Map<&'a Self, F>
        where F: Fn(Self::Output) -> T
    {
        Map { generator: self, func: f }
    }
}

impl <'a, G: Generator> Generator for &'a G {
    type Output = G::Output;

    #[inline]
    fn generate<R: rand::Rng>(&self, ctx: &mut GenerateCtx<R>) -> Self::Output {
        (*self).generate(ctx)
    }
}

pub struct Constant<T>(pub T);

impl <T: Clone> Generator for Constant<T> {
    type Output = T;

    #[inline]
    fn generate<R: rand::Rng>(&self, _: &mut GenerateCtx<R>) -> T {
        self.0.clone()
    }
}

pub struct Map<G, F> {
    generator: G,
    func: F,
}

impl <G: Generator, F, T> Generator for Map<G, F>
    where F: Fn(G::Output) -> T
{
    type Output = T;

    fn generate<R: rand::Rng>(&self, ctx: &mut GenerateCtx<R>) -> T {
        (self.func)(self.generator.generate(ctx))
    }
}

impl <G: Generator> Generator for (G,) {
    type Output = (G::Output,);

    #[inline]
    fn generate<R: rand::Rng>(&self, ctx: &mut GenerateCtx<R>) -> Self::Output {
        (self.0.generate(ctx),)
    }
}
