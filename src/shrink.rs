use std::marker::PhantomData;
use std::iter::{self, FromIterator, IntoIterator};

pub trait Shrink {
    type Item;
    type Iterator: Iterator<Item=Self::Item>;

    fn shrink(&self, value: &Self::Item) -> Self::Iterator;
}

pub struct Empty<T>(PhantomData<T>);

impl <T> Empty<T> {
    pub fn empty() -> Self { Empty(PhantomData) }
}

impl <T> Shrink for Empty<T> {
    type Item = T;
    type Iterator = iter::Empty<T>;

    fn shrink(&self, _: &Self::Item) -> Self::Iterator {
        iter::empty()
    }
}

impl <'a, S: Shrink> Shrink for &'a S {
    type Item = S::Item;
    type Iterator = S::Iterator;

    fn shrink(&self, value: &Self::Item) -> Self::Iterator {
        (*self).shrink(value)
    }
}

macro_rules! tuple_shrink_iterator {
    ( $shrinkers:ident, ($($value_before:ident,)*), ($value:ident, $($value_after:ident,)*)) => {
        {
            let values = ($($value_before.clone(),)* $($value_after.clone(),)*);
            let ($(ref $value_before,)* ref shrinker, $(ref $value_after,)*) = *$shrinkers;
            shrinker.shrink(&$value)
                .scan(
                    values,
                    |values, value_shrunk| {
                        let ($($value_before,)* $($value_after,)*) = values.clone();
                        Some(($($value_before,)* value_shrunk, $($value_after,)*))
                    }
                )
        }.chain(tuple_shrink_iterator!($shrinkers, ($($value_before,)* $value,), ($($value_after,)*)))
    };
    ( $shrinkers:ident, ($($value_before:ident,)*), ()) => {
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
            fn shrink(&self, value: &Self::Item) -> Self::Iterator {
                let ($first, $($rest,)*) = value.clone();
                Box::new(
                    tuple_shrink_iterator!(self, (), ($first, $($rest,)*))
                )
            }
        }
    };
    () => {
        impl Shrink for () {
            type Item = ();
            type Iterator = iter::Empty<()>;

            #[inline]
            fn shrink(&self, _: &()) -> Self::Iterator {
                iter::empty()
            }
        }
    }
}

macro_tuples_impl!{tuple_impls}

pub struct IntegerShrinker<T>(PhantomData<T>);

impl <T> IntegerShrinker<T> where IntegerShrinker<T>: Shrink
{
    pub fn new() -> Self { IntegerShrinker(PhantomData) }
}

macro_rules! int_impls {
    ($($ty:ty),*) => {
        $(
            impl Shrink for IntegerShrinker<$ty>
            {
                type Item = $ty;
                type Iterator = Box<Iterator<Item=Self::Item>>;

                #[inline]
                fn shrink(&self, v: &$ty) -> Self::Iterator {
                    let v = *v;
                    let mut initials = vec![0];
                    if v < 0 { initials.push(-v); }
                    initials.push(v/2);
                    let range = 0..(v.abs());
                    if v < 0 {
                        Box::new(
                            initials.into_iter()
                                .chain(range.scan(v, |&mut v, r| Some(v + r)))
                        )
                    } else {
                        Box::new(
                            initials.into_iter()
                                .chain(range.scan(v, |&mut v, r| Some(v - r)))
                        )
                    }
                }
            }
        )*
    }
}

int_impls! { i8, i16, i32, i64, isize }

pub struct UnsignedIntegerShrinker<T>(PhantomData<T>);

impl <T> UnsignedIntegerShrinker<T> where UnsignedIntegerShrinker<T>: Shrink
{
    pub fn new() -> Self { UnsignedIntegerShrinker(PhantomData) }
}

macro_rules! uint_impls {
    ($($ty:ty),*) => {
        $(
            impl Shrink for UnsignedIntegerShrinker<$ty>
            {
                type Item = $ty;
                type Iterator = Box<Iterator<Item=Self::Item>>;

                #[inline]
                fn shrink(&self, v: &$ty) -> Self::Iterator {
                    let v = *v;
                    Box::new(
                        vec![0, v/2].into_iter()
                            .chain((0..v).scan(v, |&mut v, r| Some(v - r)))
                    )
                }
            }
        )*
    }
}

uint_impls! { i8, i16, i32, i64, isize }

pub struct FromIteratorShrinker<C, S> {
    shrinker: S,
    _marker: PhantomData<C>
}

impl <C, S> FromIteratorShrinker<C, S>
    where FromIteratorShrinker<C, S>: Shrink
{
    pub fn new(shrinker: S) -> Self {
        FromIteratorShrinker { shrinker: shrinker, _marker: PhantomData }
    }
}

impl <C, S> Shrink for FromIteratorShrinker<C, S>
    where S: Shrink + Clone + 'static,
          C: FromIterator<S::Item> + IntoIterator<Item=S::Item> + Clone + 'static,
          S::Item: Clone + 'static
{
    type Item = C;
    type Iterator = Box<Iterator<Item=Self::Item>>;

    fn shrink(&self, v: &C) -> Self::Iterator {
        let elements = v.clone().into_iter().collect::<Vec<_>>();
        let elements_len = elements.len();
        let shrinker = self.shrinker.clone();

        Box::new(
            (0..(elements.len()))
                .map(|shift| elements_len >> shift)
                .take_while(|&k| k > 0)
                .scan(elements.clone(), |elements, k| Some(Removes::new(elements.clone(), k)))
                .fold(Box::new(iter::empty()) as Box<Iterator<Item=C>>, |it, next| Box::new(it.chain(next)) )
                .chain(
                    (0..(elements.len()))
                        .scan((elements, shrinker), |&mut (ref elements, ref shrinker), idx| {
                            let ref v = elements[idx];
                            Some(
                                shrinker.shrink(&v)
                                    .map(|v_shrunk| {
                                        let mut elements = elements.clone();
                                        elements[idx] = v_shrunk;
                                        elements.into_iter().collect::<C>()
                                    })
                                    .nth(0)
                            )
                        })
                        .filter_map(|x: Option<C>| x)
                )
        )
    }
}

struct Removes<T, C> {
    elements: Vec<T>,
    k: usize,
    offset: usize,
    _marker: PhantomData<C>
}

impl <T, C> Removes<T, C>
    where Removes<T, C>: Iterator
{
    fn new(elements: Vec<T>, k: usize) -> Self {
        Removes {
            elements: elements,
            k: k,
            offset: 0,
            _marker: PhantomData
        }
    }
}

impl <T, C> Iterator for Removes<T, C>
    where C: FromIterator<T>,
          T: Clone
{
    type Item = C;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.elements.len() { None } else {
            let result = Some(
                self.elements.iter()
                    .take(self.offset)
                    .chain(
                        self.elements.iter()
                            .skip(self.offset + self.k)
                    )
                    .cloned()
                    .collect::<C>()
            );
            self.offset += self.k;
            result
        }
    }
}

pub struct DefaultShrinker<T>(PhantomData<T>);

impl <T> DefaultShrinker<T>
    where DefaultShrinker<T>: Shrink
{
    pub fn new() -> Self { DefaultShrinker(PhantomData) }
}

impl Shrink for DefaultShrinker<bool> {
    type Item = bool;
    type Iterator = Box<Iterator<Item=bool>>;

    fn shrink(&self, value: &bool) -> Self::Iterator {
        match *value {
            true => Box::new(iter::once(false)),
            false => Box::new(iter::empty())
        }
    }
}
