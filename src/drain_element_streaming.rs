use _try_drain::_TryDrain;
//use element::StreamElement;
use ::std::{ops, slice, ptr, mem};

/*
struct _TryDrain<'a, T: 'a> {
    vec: &'a mut Vec<T>,
    finished: bool,
    idx: usize,
    del: usize,
    old_len: usize,
    last_element_taken: bool,
}
*/

pub struct StreamDrain<'a, T: 'a> {
    drain: _TryDrain<'a, T>,
}

impl<'a, T: 'a> StreamDrain<'a, T> {
    pub(crate) fn new(vec: &'a mut Vec<T>) -> Self {
        StreamDrain {
            drain: _TryDrain::new(vec)
        }
    }
}

impl<'a, T: 'a> ops::Drop for StreamDrain<'a, T> {
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
                drain.vec.set_len(drain.old_len-del);
            }
        }
    }
}

impl<'a: 'b, 'b, T: 'a> StreamDrain<'a, T>
{
    #[inline]
    pub fn streaming_next(&'b mut self) -> Option<StreamElement<'a, 'b, T>> {
        if self.drain.idx != self.drain.old_len {
            self.drain.idx += 1;
            return Some(StreamElement { drain: &mut self.drain });
        }
        None
    }
}


pub struct StreamElement<'a: 'b, 'b, T: 'a> {
    pub(crate) drain: &'b mut _TryDrain<'a, T>,
}

impl<'a: 'b, 'b, T: 'a> ops::Deref for StreamElement<'a, 'b, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<'a: 'b, 'b, T: 'a> ops::DerefMut for StreamElement<'a, 'b, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

impl<'a: 'b, 'b, T: 'a> StreamElement<'a, 'b, T> {
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
    pub fn take(self) -> T {
        // take element out, but don't shift anything over
        self.drain.del += 1;
        let element = unsafe {
            let v = slice::from_raw_parts_mut(self.drain.vec.as_mut_ptr(), self.drain.old_len);
            ptr::read(&v[self.drain.idx - 1])
        };
        mem::forget(self);
        element
    }
}

impl<'a: 'b, 'b, T: 'a> ops::Drop for StreamElement<'a, 'b, T> {
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
