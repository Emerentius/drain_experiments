use flag::Flag;
use _try_drain::_TryDrain;
use ::std::{ops, slice, ptr};

pub struct TryDrain<'a, T: 'a, F>
    where F: FnMut(&mut T) -> Flag,
{
    drain: _TryDrain<'a, T>,
    pred: F,
}

impl<'a, T: 'a, F> TryDrain<'a, T, F>
    where F: FnMut(&mut T) -> Flag,
{
    pub fn new(vec: &'a mut Vec<T>, pred: F) -> Self {
        TryDrain {
            drain: _TryDrain::new(vec),
            pred,
        }
    }
}

impl<'a, T: 'a, F> ops::Drop for TryDrain<'a, T, F>
    where F: FnMut(&mut T) -> Flag,
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

impl<'a: 'b, 'b, T: 'a, F> Iterator for TryDrain<'a, T, F>
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
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.drain.old_len - self.drain.idx))
    }
}
