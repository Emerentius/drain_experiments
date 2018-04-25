#![feature(drain_filter)]
mod flag;
mod _try_drain;
mod drain_element_flag;
mod drain_element_streaming;
mod drain_refmut_flag;
mod drain_element_move_flag;

pub use flag::{Flag, MoveFlag};
pub use drain_refmut_flag::TryDrain;
pub use drain_element_streaming::{StreamDrain, StreamElement};
pub use drain_element_flag::{ElementTryDrain, Element};
pub use drain_element_move_flag::{ElementMoveDrain, MoveElement};


pub struct DrainBuilder<'a, T: 'a> {
    vec: &'a mut Vec<T>,
    start: usize,
    end: usize,
}

pub struct DrainFilterBuilder<'a, T: 'a, F: FnMut(&mut T) -> bool>(std::vec::DrainFilter<'a, T, F>);
pub struct TryDrainBuilder<'a, T: 'a, F: FnMut(&mut T) -> Flag>(TryDrain<'a, T, F>);

impl<'a, T: 'a, F: FnMut(&mut T) -> bool> IntoIterator for DrainFilterBuilder<'a, T, F> {
    type IntoIter = std::vec::DrainFilter<'a, T, F>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        self.0
    }
}

impl<'a, T: 'a, F: FnMut(&mut T) -> Flag> IntoIterator for TryDrainBuilder<'a, T, F> {
    type IntoIter = TryDrain<'a, T, F>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        self.0
    }
}

impl<'a, T: 'a> IntoIterator for DrainBuilder<'a, T> {
    type IntoIter = std::vec::Drain<'a, T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        let rg = self.start..self.end;
        self.vec.drain(rg)
    }
}

impl<'a, T: 'a> DrainBuilder<'a, T> {
    pub fn range(self, rg: std::ops::Range<usize>) -> Self {
        /*
        self.start = rg.start;
        self.end = rg.end;
        self
        */
        // none of the adapters use this right now
        unimplemented!()
    }

    pub fn filter<F: FnMut(&mut T) -> bool>(self, f: F) -> DrainFilterBuilder<'a, T, F> {
        DrainFilterBuilder(self.vec.drain_filter(f))
    }

    pub fn try_drain<F: FnMut(&mut T) -> Flag>(self, pred: F) -> TryDrainBuilder<'a, T, F>
    {
        TryDrainBuilder(TryDrain::new(self.vec, pred))
    }

    pub fn elem_try_drain<F: FnMut(Element<T>) -> Flag>(self, pred: F) -> ElementTryDrain<'a, T, F> {
        ElementTryDrain::new(self.vec, pred)
    }

    pub fn elem_move_drain<F: FnMut(MoveElement<T>) -> MoveFlag<T>>(self, pred: F) -> ElementMoveDrain<'a, T, F> {
        ElementMoveDrain::new(self.vec, pred)
    }

    pub fn stream_drain(self) -> StreamDrain<'a, T> {
        StreamDrain::new(self.vec)
    }
}

pub trait VecExtend {
    type T;
    fn drain_builder(&mut self) -> DrainBuilder<Self::T>;
}

impl<T> VecExtend for Vec<T> {
    type T = T;
    fn drain_builder(&mut self) -> DrainBuilder<Self::T> {
        DrainBuilder {
            start: 0,
            end: self.len(),
            vec: self,
        }
    }
}
