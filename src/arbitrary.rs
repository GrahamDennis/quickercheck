use generate::{Generator, Constant};

pub trait Arbitrary {
    type ArbitraryGenerator: Generator<Output=Self>;

    fn arbitrary() -> Self::ArbitraryGenerator;
}

impl Arbitrary for usize {
    type ArbitraryGenerator = Constant<usize>;

    fn arbitrary() -> Constant<usize> {
        Constant(42)
    }
}
