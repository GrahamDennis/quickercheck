use std::marker::PhantomData;
use std::iter;

pub trait Shrink {
    type Item;
    type Iterator: Iterator<Item=Self::Item>;

    fn shrink(value: &Self::Item) -> Self::Iterator;
}

pub struct Empty<T>(PhantomData<T>);

impl <T> Empty<T> {
    pub fn empty() -> Self { Empty(PhantomData) }
}

impl <T> Shrink for Empty<T> {
    type Item = T;
    type Iterator = iter::Empty<T>;

    fn shrink(_: &Self::Item) -> Self::Iterator {
        iter::empty()
    }
}

macro_rules! tuple_impls {
    ($($name:ident),*) => {
        impl <$($name: 'static + Shrink),*> Shrink for ($($name,)*) {
            type Item = ($($name::Item,)*);
            type Iterator = Box<Iterator<Item=Self::Item>>;

            #[inline]
            #[allow(unused_variables, non_snake_case)]
            fn shrink(value: &Self::Item) -> Self::Iterator {
                let ( $(ref $name,)* ) = *value;
                Box::new(
                    iter::empty()
                )
//                ($($name.generate(ctx),)*)
            }
        }
    }
}

macro_tuples_impl!{tuple_impls}


pub struct DefaultShrinker<T>(PhantomData<T>);

impl <T> DefaultShrinker<T>
    where DefaultShrinker<T>: Shrink
{
    pub fn new() -> Self { DefaultShrinker(PhantomData) }
}

impl Shrink for DefaultShrinker<bool> {
    type Item = bool;
    type Iterator = Box<Iterator<Item=bool>>;

    fn shrink(value: &bool) -> Self::Iterator {
        match *value {
            true => Box::new(iter::once(false)),
            false => Box::new(iter::empty())
        }
    }
}
