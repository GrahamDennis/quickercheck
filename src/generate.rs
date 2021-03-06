use std::marker::PhantomData;
use std::iter::FromIterator;

use rand;
use num::traits::FromPrimitive;
use num::bigint::{BigInt, BigUint};

pub struct GenerateCtx<'a, R: ?Sized + 'a> {
    pub rng: &'a mut R,
    pub size: usize
}

impl <'a, R: ?Sized + 'a> GenerateCtx<'a, R> {
    pub fn new(rng: &'a mut R, size: usize) -> Self {
        GenerateCtx { rng: rng, size: size }
    }

    #[inline]
    pub fn chop<'b>(&'b mut self) -> GenerateCtx<'b, R>
        where 'a: 'b
    {
        Self::new(self.rng, self.size/2)
    }

    pub fn gen_size(&mut self) -> usize
        where R: rand::Rng + Sized
    {
        match self.size {
            0 => 0,
            size @ _ if size == <usize>::max_value() => self.rng.gen(),
            size @ _ => self.rng.gen_range(0, size + 1)
        }
    }
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

pub struct IntegerGenerator<X>(PhantomData<fn() -> X>);

impl <X> IntegerGenerator<X> where IntegerGenerator<X>: Generator
{
    pub fn new() -> Self { IntegerGenerator(PhantomData) }
}

macro_rules! int_impls {
    ($($ty:ty),*) => {
        $(
            impl Generator for IntegerGenerator<$ty>
            {
                type Output = $ty;

                fn generate<R: rand::Rng>(&self, ctx: &mut GenerateCtx<R>) -> $ty {
                    if ctx.size == 0 { return 0; }
                    let cast_size = <$ty>::from_usize(ctx.size);
                    let upper_bound = cast_size.and_then(|s| s.checked_add(1));
                    let lower_bound = cast_size.and_then(|s| s.checked_mul(-1));
                    match (lower_bound, upper_bound) {
                        (Some(lower), Some(upper)) => ctx.rng.gen_range(lower, upper),
                        _ => ctx.rng.gen()
                    }
                }
            }
        )*
    }
}

int_impls! { i8, i16, i32, i64, isize }

impl Generator for IntegerGenerator<BigInt>
{
    type Output = BigInt;

    #[inline]
    fn generate<R: rand::Rng>(&self, ctx: &mut GenerateCtx<R>) -> BigInt {
        BigInt::from_i64(IntegerGenerator::<i64>::new().generate(ctx)).unwrap()
    }
}

pub struct UnsignedIntegerGenerator<X>(PhantomData<fn() -> X>);

impl <X> UnsignedIntegerGenerator<X> where UnsignedIntegerGenerator<X>: Generator
{
    pub fn new() -> Self { UnsignedIntegerGenerator(PhantomData) }
}

macro_rules! uint_impls {
    ($($ty:ty),*) => {
        $(
            impl Generator for UnsignedIntegerGenerator<$ty>
            {
                type Output = $ty;

                fn generate<R: rand::Rng>(&self, ctx: &mut GenerateCtx<R>) -> $ty {
                    if ctx.size == 0 { return 0; }
                    let upper_bound = <$ty>::from_usize(ctx.size).and_then(|s| s.checked_add(1));
                    match upper_bound {
                        Some(upper) => ctx.rng.gen_range(0, upper),
                        _ => ctx.rng.gen()
                    }
                }
            }
        )*
    }
}

uint_impls! { u8, u16, u32, u64, usize, i8, i16, i32, i64, isize }

impl Generator for UnsignedIntegerGenerator<BigUint>
{
    type Output = BigUint;

    #[inline]
    fn generate<R: rand::Rng>(&self, ctx: &mut GenerateCtx<R>) -> BigUint {
        BigUint::from_u64(UnsignedIntegerGenerator::<u64>::new().generate(ctx)).unwrap()
    }
}

pub struct FromIteratorGenerator<C, G> {
    generator: G,
    _marker: PhantomData<fn() -> C>
}

impl <C, G> FromIteratorGenerator<C, G>
    where FromIteratorGenerator<C, G>: Generator
{
    pub fn new(generator: G) -> Self {
        FromIteratorGenerator { generator: generator, _marker: PhantomData }
    }
}

impl <C, G> Generator for FromIteratorGenerator<C, G>
    where G: Generator,
          C: FromIterator<G::Output>
{
    type Output = C;

    fn generate<R: rand::Rng>(&self, ctx: &mut GenerateCtx<R>) -> Self::Output {
        let size = ctx.gen_size();
        let mut chopped_ctx = ctx.chop();
        (0..size).map(|_| self.generator.generate(&mut chopped_ctx)).collect()
    }
}

pub struct OptionGenerator<G> {
    generator: G
}

impl <G> OptionGenerator<G>
    where OptionGenerator<G>: Generator
{
    pub fn new(generator: G) -> Self { OptionGenerator { generator: generator } }
}

impl <G: Generator> Generator for OptionGenerator<G> {
    type Output = Option<G::Output>;

    fn generate<R: rand::Rng>(&self, ctx: &mut GenerateCtx<R>) -> Self::Output {
        match ctx.rng.gen() {
            true => Some(self.generator.generate(ctx)),
            false => None
        }
    }
}

pub struct ResultGenerator<GOk, GErr> {
    g_ok: GOk,
    g_err: GErr
}

impl <GOk, GErr> ResultGenerator<GOk, GErr>
    where ResultGenerator<GOk, GErr>: Generator
{
    pub fn new(g_ok: GOk, g_err: GErr) -> Self { ResultGenerator { g_ok: g_ok, g_err: g_err } }
}

impl <GOk: Generator, GErr: Generator> Generator for ResultGenerator<GOk, GErr> {
    type Output = Result<GOk::Output, GErr::Output>;

    fn generate<R: rand::Rng>(&self, ctx: &mut GenerateCtx<R>) -> Self::Output {
        match ctx.rng.gen() {
            true => Ok(self.g_ok.generate(ctx)),
            false => Err(self.g_err.generate(ctx))
        }
    }
}

pub struct RandGenerator<T>(PhantomData<fn() -> T>);

impl <T> RandGenerator<T>
    where RandGenerator<T>: Generator
{
    pub fn new() -> Self {
        RandGenerator(PhantomData)
    }
}

impl <T: rand::Rand> Generator for RandGenerator<T> {
    type Output = T;

    fn generate<R: rand::Rng>(&self, ctx: &mut GenerateCtx<R>) -> Self::Output {
        ctx.rng.gen()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand;

    #[test]
    fn gen_unit() {
        let mut rng = rand::thread_rng(); let mut ctx = GenerateCtx::new(&mut rng, 5);
        assert_eq!(().generate(&mut ctx), ());
    }

    #[test]
    fn gen_u8() {
        let mut rng = rand::thread_rng(); let mut ctx = GenerateCtx::new(&mut rng, 5);
        let gen = UnsignedIntegerGenerator::<u8>::new();
        rep(&mut || { let n = gen.generate(&mut ctx); assert!(n <= 5); });
    }

    #[test]
    fn gen_i8() {
        let mut rng = rand::thread_rng(); let mut ctx = GenerateCtx::new(&mut rng, 5);
        let gen = UnsignedIntegerGenerator::<i8>::new();
        rep(&mut || { let n = gen.generate(&mut ctx); assert!((n >= -5) && (n <= 5)); });
    }

    fn rep<F>(f: &mut F) where F: FnMut() -> () {
        for _ in 0..100 {
            f()
        }
    }
}
