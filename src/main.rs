#![feature(drain_filter)]
#![feature(test)]
extern crate test;
use std::{ptr, slice};

struct _TryDrain<'a, T: 'a> {
    vec: &'a mut Vec<T>,
    finished: bool,
    idx: usize,
    del: usize,
    old_len: usize,
}

struct TryDrain<'a, T: 'a, F>
    where F: FnMut(&mut T) -> bool,
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
    where F: FnMut(&mut T) -> bool,
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
enum Flag {
    Yield, // remove and yield
    Return, // remove, yield and stop iteration
    Continue,    // keep
    Break,   // keep and stop iteration
}

impl<'a: 'b, 'b, T: 'a, F> Iterator for TryDrain<'a, T, F>
    //where F: for<'c, 'd> FnMut(&mut T) -> Flag,
    where F: FnMut(&mut T) -> bool,
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        /*
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
        */


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
        }

    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.drain.old_len - self.drain.idx))
    }
}

trait VecTryDrain {
    type T;
    fn try_drain<F>(&mut self, filter: F) -> TryDrain<Self::T, F>
        //where F: FnMut(&mut Self::T) -> Flag;
        where F: FnMut(&mut Self::T) -> bool;
}

impl<T> VecTryDrain for Vec<T> {
    type T = T;

    fn try_drain<F>(&mut self, filter: F) -> TryDrain<Self::T, F>
        //where F: FnMut(&mut T) -> Flag
        where F: FnMut(&mut Self::T) -> bool
    {
        let old_len = self.len();

        // Guard against us getting leaked (leak amplification)
        unsafe { self.set_len(0); }

        TryDrain {
            drain: _TryDrain {
                vec: self,
                finished: false,
                idx: 0,
                del: 0,
                old_len,
            },
            pred: filter,
        }
    }
}

fn main() {
    let mut a = (0..10).collect::<Vec<_>>();
    {
        let mut d = a.try_drain(|mut el| {
            // break out
            /*
            match *el >= 4 {
                true => Flag::Return,
                false => Flag::Yield,
            }
            */
            *el < 4
        });
        for element in d.by_ref()
        /*a.try_drain(|mut el| {
            // break out
            match *el.get_mut() > 3 {
                true => el.stop(),
                false => el.take(),
            }
        }) */ {

            println!("2nd: {}", element);
        }
        println!("EXTRA: {:?}", d.by_ref().next());

    }

    //for element in a.try_drain(|el| if *el < 7 { Flag::Yield } else { Flag::Break })
    for element in a.try_drain(|el| *el < 7)
        .take(3)
    {
        println!("{}", element);
    }

    println!("{:?}", a);
}

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

#[bench]
fn drain(b: &mut test::Bencher) {
    let vec = (0..1_000_000).collect::<Vec<_>>();
    b.iter(|| {
        let mut v = vec.clone();
        v.drain(..).for_each(drop);
    })
}

#[bench]
fn try_drain(b: &mut test::Bencher) {
    let vec = (0..1_000_000).collect::<Vec<_>>();
    b.iter(|| {
        let mut v = vec.clone();
        v.try_drain(|el| true).for_each(|_| ());
    })
}

#[bench]
fn drain_filter(b: &mut test::Bencher) {
    let vec = (0..1_000_000).collect::<Vec<_>>();
    b.iter(|| {
        let mut v = vec.clone();
        v.drain_filter(|_| true).for_each(drop);
    })
}

#[bench]
fn drain_string(b: &mut test::Bencher) {
    let vec = (0..1000).map(|n| n.to_string()).collect::<Vec<_>>();
    b.iter(|| {
        let mut v = vec.clone();
        v.drain(..);
    })
}

#[bench]
fn try_drain_string(b: &mut test::Bencher) {
    let vec = (0..1000).map(|n| n.to_string()).collect::<Vec<_>>();
    b.iter(|| {
        let mut v = vec.clone();
        v.try_drain(|el| true).for_each(drop);
    })
}
