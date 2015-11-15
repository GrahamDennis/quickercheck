use generate::{Generator, IntegerGenerator, UnsignedIntegerGenerator, FromIteratorGenerator};

use std::collections::{
    BTreeMap,
    BTreeSet,
    BinaryHeap,
    HashSet,
    HashMap,
    LinkedList,
    VecDeque
};
use std::iter::FromIterator;


pub trait Arbitrary: Sized {
    type Generator: Generator<Output=Self>;

    fn arbitrary() -> Self::Generator;
}

macro_rules! tuple_impls {
    ($($name:ident),*) => {
        impl <$($name: Arbitrary),*> Arbitrary for ($($name,)*) {
            type Generator = ($($name::Generator,)*);

            #[inline]
            fn arbitrary() -> Self::Generator {
                ($($name::arbitrary(),)*)
            }
        }
    }
}

macro_tuples_impl! {tuple_impls}

macro_rules! int_impls {
    ($($ty:ty),*) => {
        $(
            impl Arbitrary for $ty {
                type Generator = IntegerGenerator<$ty>;

                fn arbitrary() -> Self::Generator {
                    IntegerGenerator::new()
                }
            }
        )*
    }
}

macro_rules! uint_impls {
    ($($ty:ty),*) => {
        $(
            impl Arbitrary for $ty {
                type Generator = UnsignedIntegerGenerator<$ty>;

                fn arbitrary() -> Self::Generator {
                    UnsignedIntegerGenerator::new()
                }
            }
        )*
    }
}

int_impls!  {i8, i16, i32, i64, isize}
uint_impls! {u8, u16, u32, u64, usize}

macro_rules! generic_impls {
    ($($container:ident < $($placeholder:ident),* >),*) => {
        $(
            impl <$($placeholder: Arbitrary),*> Arbitrary for $container<$($placeholder),*>
                where $container<$($placeholder),*>: FromIterator<($($placeholder),*)>
            {
                type Generator = FromIteratorGenerator<$container<$($placeholder),*>, <($($placeholder),*) as Arbitrary>::Generator>;

                fn arbitrary() -> Self::Generator {
                    FromIteratorGenerator::new(($($placeholder::arbitrary()),*))
                }
            }
        )*
    }
}

generic_impls! {
    Vec<T>,
    BTreeMap<K, V>,
    BTreeSet<T>,
    HashMap<K, V>,
    HashSet<K, V>,
    BinaryHeap<T>,
    LinkedList<T>,
    VecDeque<T>
}
