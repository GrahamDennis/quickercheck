use std::iter;
use std::rc::Rc;

pub struct Rose<T: 'static>
{
    pub value: T,
    pub iterator: Box<Iterator<Item=Rose<T>>>
}

impl <T: 'static> Rose<T>
{
    pub fn new<I>(value: T, iterator: I) -> Rose<T>
        where I: Iterator<Item=Rose<T>> + 'static
    {
        Rose {
            value: value,
            iterator: Box::new(iterator)
        }
    }

    pub fn single(value: T) -> Rose<T>
    {
        info!("Creating a Rose::single");
        Rose {
            value: value,
            iterator: Box::new(iter::empty())
        }
    }
}

pub trait RoseTrait<T: 'static> {
    fn into_value(self) -> T;
    fn value(&self) -> &T;
    fn iterator(&self) -> Box<Iterator<Item=Self>>;
}

pub trait RoseTraitMap<T: 'static> {
    fn map<F: Fn(T) -> R + 'static, R>(self, f: F) -> Rose<R>;
    fn map_rc<F: Fn(T) -> R + 'static, R>(self, f: Rc<F>) -> Rose<R>;
    fn scan<S: 'static, F: Fn(&S, T) -> R + 'static, R>(self, state: S, f: F) -> Rose<R>;
    fn scan_rc<S: 'static, F: Fn(&S, T) -> R + 'static, R>(self, state: Rc<S>, f: Rc<F>) -> Rose<R>;
}

impl <T: 'static, ROSE: RoseTrait<T> + 'static> RoseTraitMap<T> for ROSE {
    fn map<F: Fn(T) -> R + 'static, R>(self, f: F) -> Rose<R> {
        self.map_rc(Rc::new(f))
    }

    fn map_rc<F: Fn(T) -> R + 'static, R>(self, f: Rc<F>) -> Rose<R> {
        let iterator = self.iterator();
        Rose {
            value: (&f)(self.into_value()),
            iterator: Box::new(iterator.scan(f, |f, r| Some(r.map_rc(f.clone()))))
        }
    }

    fn scan<S: 'static, F: Fn(&S, T) -> R + 'static, R>(self, state: S, f: F) -> Rose<R> {
        self.scan_rc(Rc::new(state), Rc::new(f))
    }

    fn scan_rc<S: 'static, F: Fn(&S, T) -> R + 'static, R>(self, state: Rc<S>, f: Rc<F>) -> Rose<R> {
        let iterator = self.iterator();
        Rose {
            value: (&f)(&state, self.into_value()),
            iterator: Box::new(iterator.scan((state, f), |&mut (ref mut state, ref mut f), r|
                Some(r.scan_rc(state.clone(), f.clone())
            )))
        }
    }
}

pub struct GeneratedRose<T: 'static, F>
{
    value: T,
    generator: Rc<F>
}

impl <T: 'static, F> GeneratedRose<T, F>
    where F: Fn(&T) -> Box<Iterator<Item=T>> + 'static
{
    pub fn new(value: T, generator: F) -> GeneratedRose<T, F> {
        GeneratedRose {
            value: value,
            generator: Rc::new(generator)
        }
    }
}

impl <T: 'static, F> RoseTrait<T> for GeneratedRose<T, F>
    where F: Fn(&T) -> Box<Iterator<Item=T>> + 'static
{
    fn value(&self) -> &T { &self.value }
    fn into_value(self) -> T { self.value }
    fn iterator(&self) -> Box<Iterator<Item=Self>> {
        Box::new(
            (&self.generator)(self.value()).scan(self.generator.clone(), |f, t|
                Some(GeneratedRose { value: t, generator: f.clone() })
        ))
    }
}

pub struct GenerateWithRose<T: 'static, S, F>
{
    value: T,
    state: Rc<S>,
    generator: Rc<F>
}

impl <T: 'static, S: 'static, F> GenerateWithRose<T, S, F>
    where F: Fn(&S, &T) -> Box<Iterator<Item=T>> + 'static
{
    pub fn new(value: T, state: S, generator: F) -> GenerateWithRose<T, S, F> {
        GenerateWithRose {
            value: value,
            state: Rc::new(state),
            generator: Rc::new(generator)
        }
    }
}

impl <T: 'static, S: 'static, F> RoseTrait<T> for GenerateWithRose<T, S, F>
    where F: Fn(&S, &T) -> Box<Iterator<Item=T>> + 'static
{
    fn value(&self) -> &T { &self.value }
    fn into_value(self) -> T { self.value }
    fn iterator(&self) -> Box<Iterator<Item=Self>> {
        Box::new(
            (&self.generator)(&self.state, self.value())
                .scan((self.state.clone(), self.generator.clone()), |&mut (ref mut s, ref mut f), t|
                    Some(GenerateWithRose { value: t, state: s.clone(), generator: f.clone()})
            )
        )
    }
}
