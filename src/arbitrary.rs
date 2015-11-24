use generate::{
    Generator,
    IntegerGenerator,
    UnsignedIntegerGenerator,
    FromIteratorGenerator,
    OptionGenerator,
    ResultGenerator,
    RandGenerator
};

use std::collections::{
    BTreeMap,
    BTreeSet,
    BinaryHeap,
    HashSet,
    HashMap,
    LinkedList,
    VecDeque
};
use std::iter::{self, FromIterator};
use num::bigint::{BigInt, BigUint};

pub trait Arbitrary: Sized + 'static {
    type Generator: Generator<Output=Self>;

    fn arbitrary() -> Self::Generator;
    fn shrink(self) -> Box<Iterator<Item=Self>>
    {
        Box::new(iter::empty())
    }
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

int_impls!  {i8, i16, i32, i64, isize, BigInt}
uint_impls! {u8, u16, u32, u64, usize, BigUint}

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

impl <T: Arbitrary> Arbitrary for Option<T> {
    type Generator = OptionGenerator<T::Generator>;

    fn arbitrary() -> Self::Generator {
        OptionGenerator::new(T::arbitrary())
    }
}

impl <TOk: Arbitrary, TErr: Arbitrary> Arbitrary for Result<TOk, TErr> {
    type Generator = ResultGenerator<TOk::Generator, TErr::Generator>;

    fn arbitrary() -> Self::Generator {
        ResultGenerator::new(TOk::arbitrary(), TErr::arbitrary())
    }
}

impl Arbitrary for bool {
    type Generator = RandGenerator<bool>;

    fn arbitrary() -> Self::Generator {
        RandGenerator::new()
    }
}

impl Arbitrary for char {
    type Generator = RandGenerator<char>;

    fn arbitrary() -> Self::Generator {
        RandGenerator::new()
    }
}

impl Arbitrary for String {
    type Generator = FromIteratorGenerator<String, <char as Arbitrary>::Generator>;

    fn arbitrary() -> Self::Generator {
        FromIteratorGenerator::new(char::arbitrary())
    }
}
