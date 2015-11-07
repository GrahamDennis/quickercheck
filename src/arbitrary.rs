use generate::{Generator, Constant};

pub trait Arbitrary {
    type ArbitraryGenerator: Generator<Output=Self>;

    fn arbitrary() -> Self::ArbitraryGenerator;
}

impl Arbitrary for () {
    type ArbitraryGenerator = Constant<()>;

    fn arbitrary() -> Constant<()> {
        Constant(())
    }
}

impl Arbitrary for usize {
    type ArbitraryGenerator = Constant<usize>;

    fn arbitrary() -> Constant<usize> {
        Constant(42)
    }
}

impl <T: Arbitrary + Clone> Arbitrary for (T,) {
    type ArbitraryGenerator = (T::ArbitraryGenerator,);

    fn arbitrary() -> Self::ArbitraryGenerator {
        (T::arbitrary(),)
    }
}
