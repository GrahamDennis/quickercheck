use std::iter;

pub trait Rose<T: 'static> {
    fn value(&self) -> &T;
    fn iterator(&mut self) -> &mut Iterator<Item=Box<Rose<T>>>;
}

pub struct SimpleRose<T: 'static>
{
    value: T,
    iterator: Box<Iterator<Item=Box<Rose<T>>>>
}

impl <T: 'static> Rose<T> for SimpleRose<T> {
    fn value(&self) -> &T {
        &self.value
    }

    fn iterator(&mut self) -> &mut Iterator<Item=Box<Rose<T>>> {
        &mut self.iterator
    }
}

impl <T: 'static> SimpleRose<T>
{
    pub fn new<I>(value: T, iterator: I) -> SimpleRose<T>
        where I: Iterator<Item=SimpleRose<T>> + 'static
    {
        SimpleRose {
            value: value,
            iterator: Box::new(iterator.map(|r| Box::new(r) as Box<Rose<T>>))
        }
    }

    pub fn single(value: T) -> SimpleRose<T>
    {
        SimpleRose {
            value: value,
            iterator: Box::new(iter::empty())
        }
    }
}
