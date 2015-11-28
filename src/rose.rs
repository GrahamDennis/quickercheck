
pub struct Rose<T: 'static> {
    pub value: T,
    pub iterator: Box<Iterator<Item=Rose<T>>>
}

impl <T> Rose<T> {
    pub fn new<I>(value: T, iterator: I) -> Rose<T>
        where I: Iterator<Item=Self> + 'static
    {
        Rose {
            value: value,
            iterator: Box::new(iterator)
        }
    }

    pub fn single(value: T) -> Rose<T> {
        Rose {
            value: value,
            iterator: Box::new(::std::iter::empty())
        }
    }
}
