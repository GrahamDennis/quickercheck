use generate::{Generator, Constant};

pub trait Arbitrary: Sized {
    type ArbitraryGenerator: Generator<Output=Self>;

    fn arbitrary() -> Self::ArbitraryGenerator;
}

impl Arbitrary for usize {
    type ArbitraryGenerator = Constant<usize>;

    fn arbitrary() -> Constant<usize> {
        Constant(42)
    }
}

macro_rules! fn_impls {
    ($($name:ident),*) => {
        impl <$($name: Arbitrary),*> Arbitrary for ($($name,)*) {
            type ArbitraryGenerator = ($($name::ArbitraryGenerator,)*);

            #[inline]
            fn arbitrary() -> Self::ArbitraryGenerator {
                ($($name::arbitrary(),)*)
            }
        }
    }
}

macro_tuples_impl! {fn_impls}
