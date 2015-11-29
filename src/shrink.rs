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
    ($first:ident $(, $rest:ident)*) => {
        impl <$first: 'static + Shrink $(, $rest: 'static + Shrink)*> Shrink for ($first, $($rest),*)
            where $first::Item: Clone $(, $rest::Item: Clone)*
        {
            type Item = ($first::Item, $($rest::Item),*);
            type Iterator = Box<Iterator<Item=Self::Item>>;

            #[inline]
            #[allow(unused_variables, non_snake_case)]
            fn shrink(value: &Self::Item) -> Self::Iterator {
                let ( ref $first, $(ref $rest),*) = *value;
                let t_rest: ($($rest::Item,)*) = ($($rest.clone(),)*);

                Box::new(
                    <$first as Shrink>::shrink($first)
                        .scan(t_rest.clone(), |rest, $first| {
                            let ($($rest,)*) = rest.clone();
                            Some(($first, $($rest),*))
                        })
                        .chain(
                            <($($rest,)*) as Shrink>::shrink(&t_rest)
                                .scan($first.clone(), |$first, ($($rest,)*)| {
                                    Some(($first.clone(), $($rest),*))
                                })
                        )
                )
            }
        }
    };
    () => {
        impl Shrink for () {
            type Item = ();
            type Iterator = iter::Empty<()>;

            #[inline]
            fn shrink(_: &()) -> Self::Iterator {
                iter::empty()
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
