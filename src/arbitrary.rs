use generate::{
    Generator,
    IntegerGenerator,
    UnsignedIntegerGenerator,
    FromIteratorGenerator,
    OptionGenerator,
    ResultGenerator,
    RandGenerator
};
use shrink::{
    self,
    Shrink,
    IntegerShrinker,
    UnsignedIntegerShrinker,
    FromIteratorShrinker,
    DefaultShrinker
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
use std::iter::{FromIterator};

pub trait Arbitrary: Sized + Clone + 'static {
    type Generator: Generator<Output=Self>;
    type Shrink: Shrink<Item=Self>;

    fn arbitrary() -> Self::Generator;
    fn shrink() -> Self::Shrink;
}

macro_rules! tuple_impls {
    ($($name:ident),*) => {
        impl <$($name: Arbitrary),*> Arbitrary for ($($name,)*) {
            type Generator = ($($name::Generator,)*);
            type Shrink = ($($name::Shrink,)*);

            fn arbitrary() -> Self::Generator {
                ($($name::arbitrary(),)*)
            }

            fn shrink() -> Self::Shrink {
                ($($name::shrink(),)*)
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
                type Shrink = IntegerShrinker<$ty>;

                fn arbitrary() -> Self::Generator {
                    IntegerGenerator::new()
                }

                fn shrink() -> Self::Shrink {
                    IntegerShrinker::new()
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
                type Shrink = UnsignedIntegerShrinker<$ty>;

                fn arbitrary() -> Self::Generator {
                    UnsignedIntegerGenerator::new()
                }

                fn shrink() -> Self::Shrink {
                    UnsignedIntegerShrinker::new()
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
            impl <$($placeholder: Arbitrary + Clone),*> Arbitrary for $container<$($placeholder),*>
                where $container<$($placeholder),*>: FromIterator<($($placeholder),*)> + IntoIterator<Item=($($placeholder),*)>
            {
                type Generator = FromIteratorGenerator<$container<$($placeholder),*>, <($($placeholder),*) as Arbitrary>::Generator>;
                type Shrink = FromIteratorShrinker<$container<$($placeholder),*>, <($($placeholder),*) as Arbitrary>::Shrink>;

                fn arbitrary() -> Self::Generator {
                    FromIteratorGenerator::new(($($placeholder::arbitrary()),*))
                }

                fn shrink() -> Self::Shrink {
                    FromIteratorShrinker::new(($($placeholder::shrink()),*))
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
    type Shrink = shrink::Empty<Self>;

    fn arbitrary() -> Self::Generator {
        OptionGenerator::new(T::arbitrary())
    }

    #[inline] fn shrink() -> Self::Shrink { shrink::Empty::empty() }
}

impl <TOk: Arbitrary, TErr: Arbitrary> Arbitrary for Result<TOk, TErr> {
    type Generator = ResultGenerator<TOk::Generator, TErr::Generator>;
    type Shrink = shrink::Empty<Self>;

    fn arbitrary() -> Self::Generator {
        ResultGenerator::new(TOk::arbitrary(), TErr::arbitrary())
    }

    #[inline] fn shrink() -> Self::Shrink { shrink::Empty::empty() }
}

impl Arbitrary for bool {
    type Generator = RandGenerator<bool>;
    type Shrink = DefaultShrinker<bool>;

    fn arbitrary() -> Self::Generator {
        RandGenerator::new()
    }

    fn shrink() -> Self::Shrink {
        DefaultShrinker::new()
    }
}

impl Arbitrary for char {
    type Generator = RandGenerator<char>;
    type Shrink = shrink::Empty<Self>;

    fn arbitrary() -> Self::Generator {
        RandGenerator::new()
    }

    #[inline] fn shrink() -> Self::Shrink { shrink::Empty::empty() }
}

impl Arbitrary for String {
    type Generator = FromIteratorGenerator<String, <char as Arbitrary>::Generator>;
    type Shrink = shrink::Empty<Self>;

    fn arbitrary() -> Self::Generator {
        FromIteratorGenerator::new(char::arbitrary())
    }

    #[inline] fn shrink() -> Self::Shrink { shrink::Empty::empty() }
}
