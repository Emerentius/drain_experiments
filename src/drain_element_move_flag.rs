use flag::MoveFlag;
use _try_drain::_TryDrain;
use ::std::{ops, slice, ptr};

pub struct ElementMoveDrain<'a, T: 'a, F>
    where F: FnMut(MoveElement<T>) -> MoveFlag<T>,
{
    drain: _TryDrain<'a, T>,
    pred: F,
}

impl<'a, T: 'a, F> ElementMoveDrain<'a, T, F>
    where F: FnMut(MoveElement<T>) -> MoveFlag<T>,
{
    pub fn new(vec: &'a mut Vec<T>, pred: F) -> Self {
        ElementMoveDrain {
            drain: _TryDrain::new(vec),
            pred,
        }
    }
}

impl<'a, T: 'a, F> ops::Drop for ElementMoveDrain<'a, T, F>
    where F: FnMut(MoveElement<T>) -> MoveFlag<T>,
{
    fn drop(&mut self) {
        let drain = &mut self.drain;
        let del = drain.del;

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

        unsafe {
            drain.vec.set_len(drain.old_len-del);
        }
    }
}

impl<'a: 'b, 'b, T: 'a, F> Iterator for ElementMoveDrain<'a, T, F>
    where F: FnMut(MoveElement<T>) -> MoveFlag<T>,
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while (self.drain.idx != self.drain.old_len) & !self.drain.finished {
            let i = self.drain.idx;
            self.drain.idx += 1;

            unsafe {
                let v = slice::from_raw_parts_mut(self.drain.vec.as_mut_ptr(), self.drain.old_len);
                match (self.pred)(MoveElement { drain: &mut self.drain }) {
                    MoveFlag::Yield(val) => {
                        return Some(val);
                    },
                    MoveFlag::Continue => {
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
                    MoveFlag::Break => {
                        self.drain.finished = true;
                        self.drain.idx -= 1;
                        break
                    },
                    MoveFlag::Return(val) => {
                        self.drain.finished = true;
                        //self.drain.del += 1;
                        return Some(val);
                    }
                }
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.drain.old_len - self.drain.idx))
    }
}


pub struct MoveElement<'a: 'b, 'b, T: 'a> {
    pub(crate) drain: &'b mut _TryDrain<'a, T>,
}

impl<'a: 'b, 'b, T: 'a> ops::Deref for MoveElement<'a, 'b, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<'a: 'b, 'b, T: 'a> ops::DerefMut for MoveElement<'a, 'b, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

impl<'a: 'b, 'b, T: 'a> MoveElement<'a, 'b, T> {
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
    fn take_inner(self) -> T {
        // take element out, but don't shift anything over
        self.drain.del += 1;
        let element = unsafe {
            let v = slice::from_raw_parts_mut(self.drain.vec.as_mut_ptr(), self.drain.old_len);
            ptr::read(&v[self.drain.idx - 1])
        };
        element
    }

    #[inline]
    pub fn take(self) -> MoveFlag<T> {
        MoveFlag::Yield(self.take_inner())
    }

    #[inline]
    pub fn take_and_stop(self) -> MoveFlag<T> {
        MoveFlag::Return(self.take_inner())
    }

    #[inline]
    pub fn stop(self) -> MoveFlag<T> {
        MoveFlag::Break
    }

    #[inline]
    pub fn keep(self) -> MoveFlag<T> {
        MoveFlag::Continue
    }
}
