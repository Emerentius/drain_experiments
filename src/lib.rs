#![feature(drain_filter)]
use std::{ptr, slice};
struct _TryDrain<'a, T: 'a> {
    vec: &'a mut Vec<T>,
    finished: bool,
    idx: usize,
    del: usize,
    old_len: usize,
}

pub struct TryDrain<'a, T: 'a, F>
    where F: FnMut(&mut T) -> Flag,
{
    drain: _TryDrain<'a, T>,
    pred: F,
}

impl<'a, T: 'a> _TryDrain<'a, T> {
    fn tail_len(&self) -> usize {
        self.old_len - self.idx
    }
}

impl<'a, T: 'a, F> std::ops::Drop for TryDrain<'a, T, F>
    where F: FnMut(&mut T) -> Flag,
{
    fn drop(&mut self) {
        //for _ in self.by_ref() {}

        let drain = &mut self.drain;
        let del = drain.del;
        /*
        if del > 0 {
            let tail_len = drain.tail_len();
            if tail_len == 0 { return }
            unsafe {
                let v = slice::from_raw_parts_mut(drain.vec.as_mut_ptr(), drain.old_len);
                let i = drain.idx;
                let src: *const T = &v[i];
                let dst: *mut T = &mut v[i - del];
                // This is safe because self.vec has length 0
                // thus its elements will not have Drop::drop
                // called on them in the event of a panic.
                ptr::copy(src, dst, tail_len);
                // RANGE: add parameter here
            }
        }
        */
        unsafe {
            drain.vec.set_len(drain.old_len-del);
        }
    }
}
/*

struct Element<'a: 'b, 'b, T: 'a> {
    drain: &'b mut _TryDrain<'a, T>,
}

impl<'a: 'b, 'b, T: 'a> std::ops::Deref for Element<'a, 'b, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<'a: 'b, 'b, T: 'a> std::ops::DerefMut for Element<'a, 'b, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

impl<'a: 'b, 'b, T: 'a> Element<'a, 'b, T> {
    #[inline]
    fn get_mut(&mut self) -> &mut T {
        let i = self.drain.idx - 1;
        //let v = slice::from_raw_parts_mut(self.drain.vec.as_mut_ptr(), self.drain.old_len);
        unsafe { self.drain.vec.get_unchecked_mut(i) }
    }

    #[inline]
    fn get(&self) -> &T {
        let i = self.drain.idx - 1;
        //let v = slice::from_raw_parts_mut(self.drain.vec.as_mut_ptr(), self.drain.old_len);
        unsafe { self.drain.vec.get_unchecked(i) }
    }

    #[inline]
    fn take(self) -> Flag {
        Flag::Yield
    }

    #[inline]
    fn take_inner(self) -> T {
        // take element out, but don't shift anything over
        self.drain.del += 1;
        let element = unsafe {
            let v = slice::from_raw_parts_mut(self.drain.vec.as_mut_ptr(), self.drain.old_len);
            ptr::read(&v[self.drain.idx - 1])
        };
        std::mem::forget(self);
        element
    }

    #[inline]
    fn delete(self) -> Flag {
        Flag::Continue
    }

    #[inline]
    fn take_and_stop(self) -> Flag {
        Flag::Return
    }

    /*
    fn take(self) -> Flag {
        Flag::Take
    }
    */

    #[inline]
    #[allow(unused)]
    fn keep(self) -> Flag {
        Flag::Continue
    }

    #[inline]
    fn stop(self) -> Flag {
        Flag::Break
    }

    /*
    // TODO: this is wrong
    #[inline]
    fn skip(self, n: usize) -> Flag<T> {
        let del = self.drain.del;
        if n > 0 && del > 0 {
            let n = std::cmp::min(n, self.drain.tail_len());
            self.drain.idx += n;
            unsafe {
                let v = slice::from_raw_parts_mut(self.drain.vec.as_mut_ptr(), self.drain.old_len);
                let i = self.drain.idx - 1;
                let src: *const T = &v[i];
                let dst: *mut T = &mut v[i - del];
                // This is safe because self.vec has length 0
                // thus its elements will not have Drop::drop
                // called on them in the event of a panic.
                ptr::copy(src, dst, n);
            }
        }
        Flag::Continue
    }
    */
}

impl<'a: 'b, 'b, T: 'a> std::ops::Drop for Element<'a, 'b, T> {
    fn drop(&mut self) {
        let del = self.drain.del;
        if del > 0 {
            unsafe {
                let v = slice::from_raw_parts_mut(self.drain.vec.as_mut_ptr(), self.drain.old_len);
                let i = self.drain.idx - 1;
                let src: *const T = &v[i];
                let dst: *mut T = &mut v[i - del];
                // This is safe because self.vec has length 0
                // thus its elements will not have Drop::drop
                // called on them in the event of a panic.
                ptr::copy_nonoverlapping(src, dst, 1);
            }
        }
    }
}
*/
pub enum Flag {
    Yield, // remove and yield
    Return, // remove, yield and stop iteration
    Continue,    // keep
    Break,   // keep and stop iteration
}

impl<'a: 'b, 'b, T: 'a, F> Iterator for TryDrain<'a, T, F>
    //where F: for<'c, 'd> FnMut(&mut T) -> Flag,
    where F: FnMut(&mut T) -> Flag,
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while (self.drain.idx != self.drain.old_len) & !self.drain.finished {
            let i = self.drain.idx;
            self.drain.idx += 1;

            unsafe {
                let v = slice::from_raw_parts_mut(self.drain.vec.as_mut_ptr(), self.drain.old_len);
                match (self.pred)(&mut v[i]) {
                    Flag::Yield => {
                        self.drain.del += 1;
                        return Some(ptr::read(&v[i]));
                    },
                    Flag::Continue => {
                        let del = self.drain.del;
                        if del > 0 {
                            let src: *const T = &v[i];
                            let dst: *mut T = &mut v[i - del];
                            // This is safe because self.vec has length 0
                            // thus its elements will not have Drop::drop
                            // called on them in the event of a panic.
                            ptr::copy_nonoverlapping(src, dst, 1);
                        }
                    },
                    Flag::Break => {
                        self.drain.finished = true;
                        self.drain.idx -= 1;
                        break
                    },
                    Flag::Return => {
                        self.drain.finished = true;
                        self.drain.del += 1;
                        return Some(ptr::read(&v[i]));
                    }
                }
            }
        }
        None


        /*
        unsafe {
            while self.drain.idx != self.drain.old_len {
                let i = self.drain.idx;
                self.drain.idx += 1;
                let v = slice::from_raw_parts_mut(self.drain.vec.as_mut_ptr(), self.drain.old_len);
                if (self.pred)(&mut v[i]) {
                    self.drain.del += 1;
                    return Some(ptr::read(&v[i]));
                } else if self.drain.del > 0 {
                    let del = self.drain.del;
                    let src: *const T = &v[i];
                    let dst: *mut T = &mut v[i - del];
                    // This is safe because self.vec has length 0
                    // thus its elements will not have Drop::drop
                    // called on them in the event of a panic.
                    ptr::copy_nonoverlapping(src, dst, 1);
                }
            }
            None
        */
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.drain.old_len - self.drain.idx))
    }
}

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
    fn range(mut self, rg: std::ops::Range<usize>) -> Self {
        self.start = rg.start;
        self.end = rg.end;
        self
    }

    fn filter<F: FnMut(&mut T) -> bool>(self, f: F) -> DrainFilterBuilder<'a, T, F> {
        DrainFilterBuilder(self.vec.drain_filter(f))
    }

    fn try_drain<F: FnMut(&mut T) -> Flag>(self, pred: F) -> TryDrainBuilder<'a, T, F>
    {
        let old_len = self.vec.len();

        // Guard against us getting leaked (leak amplification)
        unsafe { self.vec.set_len(0); }

        TryDrainBuilder(TryDrain {
            drain: _TryDrain {
                vec: self.vec,
                finished: false,
                idx: 0,
                del: 0,
                old_len,
                //last_element_taken: false,
            },
            pred,
        })
    }
}

pub trait VecExtend {
    type T;
    fn try_drain<'a, F: FnMut(&mut Self::T) -> Flag>(&'a mut self, pred: F) -> TryDrain<'a, Self::T, F>;
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


    fn try_drain<'a, F: FnMut(&mut T) -> Flag>(&'a mut self, pred: F) -> TryDrain<'a, T, F>
    {
        self.drain_builder().try_drain(pred).into_iter()
    }
}
