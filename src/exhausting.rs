/*
trait IteratorExt {
    fn exhausting(self) -> Exhausting<Self> where Self: Sized + Iterator;
}

impl<T: Iterator> IteratorExt for T {
    fn exhausting(self) -> Exhausting<Self>
    where
        Self: Sized
    {
        Exhausting {
            iter: self
        }
    }
}

struct Exhausting<T: Iterator> {
    iter: T
}

impl<T: Iterator> std::ops::Drop for Exhausting<T> {
    fn drop(&mut self) {
        for _ in self {}
    }
}

impl<T: Iterator> Iterator for Exhausting<T> {
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
*/
