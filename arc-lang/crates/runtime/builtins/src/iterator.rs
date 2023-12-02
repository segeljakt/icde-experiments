use crate::cow::Cow;
use crate::vec::Vec;

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Iter<I>(Cow<I>);

impl<T, I: std::iter::Iterator<Item = T>> Iter<I> {
    pub fn new(i: I) -> Iter<I> {
        Iter(Cow::new(i))
    }

    pub fn enumerate(self) -> Iter<std::iter::Enumerate<I>>
    where
        T: Clone,
        I: Clone,
    {
        Iter(Cow::new(self.0.take().enumerate()))
    }

    pub fn map<U, F: FnMut(T) -> U>(self, f: F) -> Iter<std::iter::Map<I, F>>
    where
        T: Clone,
        I: Clone,
    {
        Iter(Cow::new(self.0.take().map(f)))
    }

    pub fn filter<F: FnMut(T) -> bool>(
        self,
        mut f: F,
    ) -> Iter<std::iter::Filter<I, impl FnMut(&T) -> bool>>
    where
        T: Clone,
        I: Clone,
    {
        Iter(Cow::new(self.0.take().filter(move |s: &T| f(s.clone()))))
    }

    pub fn for_each<S, F: FnMut(S, T) -> std::ops::ControlFlow<S, S>>(
        mut self,
        mut s0: S,
        mut f: F,
    ) -> S
    where
        T: Clone,
        I: Clone,
    {
        self.0.update(|this| loop {
            match this.next() {
                Some(v) => match f(s0, v) {
                    std::ops::ControlFlow::Continue(s1) => s0 = s1,
                    std::ops::ControlFlow::Break(s1) => break s1,
                },
                None => break s0,
            };
        })
    }

    pub fn test(self)
    where
        T: Clone,
        I: Clone,
    {
        // var x = 0;
        // x = 1;
        // x = x + 1;
        let mut x = 0;
        x = x.clone() + 1;
        println!("{}", x);
        
        let mut iter = self.0.take();
        let mut count = 0;
        loop {
            if let Some(_) = iter.next() {
                count += 1;
            } else {
                break;
            }
        }
        println!("{}", count);
    }

    pub fn collect_vec(self) -> Vec<T>
    where
        T: Clone,
        I: Clone,
    {
        self.0
            .map(|this| Vec::from(this.collect::<std::vec::Vec<_>>()))
    }
}

#[macro_export]
macro_rules! for_each {
    ($state:expr, $var:pat, $iter:expr, $body:expr) => {
        for $var in $iter {
            $body
        }
    };
}

impl<T, I: std::iter::Iterator<Item = T> + Clone> Iterator for Iter<I> {
    type Item = T;

    fn next(&mut self) -> std::option::Option<T> {
        self.0.update(|this| match this.next() {
            Some(x) => std::option::Option::Some(x),
            None => std::option::Option::None,
        })
    }
}
