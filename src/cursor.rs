use ::std::{ops, slice, ptr};

pub struct Cursor<'a, T: 'a> {
    vec: &'a mut Vec<T>,
    idx: usize,
    del: usize,
    old_len: usize,
}

impl<'a, T: 'a> Cursor<'a, T> {
    pub fn new(vec: &'a mut Vec<T>) -> Self {
        let old_len = vec.len();

        // Guard against us getting leaked (leak amplification)
        unsafe { vec.set_len(0); }

        Cursor {
            vec,
            idx: 0,
            del: 0,
            old_len
        }
    }

    /// keep the current element and advance to the next
    /// does nothing if cursor is already at the end
    pub fn advance(&mut self) {//, n: usize) {
        if self.idx == self.old_len {
            return
        }
        let i = self.idx;
        self.idx += 1;
        let del = self.del;
        if del > 0 {
            unsafe {
                let v = slice::from_raw_parts_mut(self.vec.as_mut_ptr(), self.old_len);
                let src: *const T = &v[i];
                let dst: *mut T = &mut v[i - del];
                // This is safe because self.vec has length 0
                // thus its elements will not have Drop::drop
                // called on them in the event of a panic.
                ptr::copy_nonoverlapping(src, dst, 1);
            }
        }
    }

    /// Take a reference to the current element
    /// Returns `None` if cursor is exhausted
    /// The current element can be removed through the `MoveElement` without additional checks
    pub fn get<'b>(&'b mut self) -> Option<MoveElement<'a, 'b, T>> {
        match self.idx != self.old_len {
            true => Some ( MoveElement { cursor: self } ) ,
            false => None,
        }
    }

    /*
    pub fn get_mut(&mut self) -> Option<&mut T> {
        unsafe {
            let i = self.idx;
            let v = slice::from_raw_parts_mut(self.vec.as_mut_ptr(), self.old_len);
            v.get_mut(i)
        }
    }
    */

    /// Remove current element and advance to next
    /// Returns `None` if cursor is exhausted
    pub fn take(&mut self) -> Option<T> {
        // equivalent but slow
        //self.get().map(|el| el.take())
        if self.idx != self.old_len {
            unsafe {
                let i = self.idx;
                self.idx += 1;
                self.del += 1;
                let v = slice::from_raw_parts_mut(self.vec.as_mut_ptr(), self.old_len);
                Some(ptr::read(&v[i]))
            }
        } else {
            None
        }
    }

    /// Drop current element in place and advance to next
    /// Returns false if cursor is exhausted
    pub fn delete(&mut self) -> bool {
        if self.idx != self.old_len {
            unsafe {
                let i = self.idx;
                self.idx += 1;
                self.del += 1;
                let v = slice::from_raw_parts_mut(self.vec.as_mut_ptr(), self.old_len);
                ptr::drop_in_place(&mut v[i]);
                true
            }
        } else {
            false
        }

    }

    /// Take a reference to the next element if it exists
    pub fn peek(&mut self) -> Option<&mut T> {
        unsafe {
            let v = slice::from_raw_parts_mut(self.vec.as_mut_ptr(), self.old_len);
            v.get_mut(self.idx+1)
        }
    }

    /// Take a reference to all elements that are still in the collection
    /// The first slice is to elements that were iterated over and kept
    /// The second slice is to the elements that are yet to be iterated including the current element
    pub fn peek_slices(&mut self) -> (&mut [T], &mut [T]) {
        unsafe {
            let v = slice::from_raw_parts_mut(self.vec.as_mut_ptr(), self.old_len);
            let end_head = self.idx-self.del;
            //let start_tail = self.idx;
            let (head, mut tail) = v.split_at_mut(end_head);
            // exclude the hole of invalid memory and the current element to avoid mutable aliasing
            tail = &mut {tail}[self.del+1..];
            (head, tail)
        }
    }

    /// Create an iterator out of the cursor
    /// Takes a closure that gives a mutable reference to the cursor and expects
    /// elements to yield. This means you have to control the iteration of the cursor
    /// explicitly.
    pub fn iter<F>(self, pred: F) -> CursorIter<'a, T, F>
    where
        F: FnMut(&mut Cursor<T>) -> Option<T>,
    {
        CursorIter {
            cursor: self,
            pred,
        }
    }

    /// Drop all remaining elements after and including the current position
    pub fn clear(&mut self) {
        while self.idx != self.old_len {
            unsafe {
                let i = self.idx;
                self.idx += 1;
                self.del += 1;
                let v = slice::from_raw_parts_mut(self.vec.as_mut_ptr(), self.old_len);
                ptr::drop_in_place(&mut v[i]);
            }
        }
    }
}

pub struct CursorIter<'a, T: 'a, F>
where
    F: FnMut(&mut Cursor<T>) -> Option<T>,
{
    cursor: Cursor<'a, T>,
    pred: F,
}

impl<'a, T: 'a, F> Iterator for CursorIter<'a, T, F>
where
    F: FnMut(&mut Cursor<T>) -> Option<T>,
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        (self.pred)(&mut self.cursor)
    }
}
/*

impl<'a, T: 'a> ops::Deref for Cursor<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe {
            let i = self.idx - 1;
            let v = slice::from_raw_parts_mut(self.vec.as_mut_ptr(), self.old_len);
            //unsafe { self.vec.get_unchecked(i) }
            &v[i]
        }

    }
}

impl<'a, T: 'a> ops::DerefMut for Cursor<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            let i = self.idx - 1;
            let v = slice::from_raw_parts_mut(self.vec.as_mut_ptr(), self.old_len);
            &mut v[i]
        }
        //unsafe { self.vec.get_unchecked_mut(i) }
    }
}

*/



pub struct MoveElement<'a: 'b, 'b, T: 'a> {
    pub(crate) cursor: &'b mut Cursor<'a, T>,
}

impl<'a: 'b, 'b, T: 'a> ops::Deref for MoveElement<'a, 'b, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        let i = self.cursor.idx;
        //let v = slice::from_raw_parts_mut(self.cursor.vec.as_mut_ptr(), self.cursor.old_len);
        unsafe { self.cursor.vec.get_unchecked(i) }
    }
}

impl<'a: 'b, 'b, T: 'a> ops::DerefMut for MoveElement<'a, 'b, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let i = self.cursor.idx;
        //let v = slice::from_raw_parts_mut(self.cursor.vec.as_mut_ptr(), self.cursor.old_len);
        unsafe { self.cursor.vec.get_unchecked_mut(i) }
    }
}

impl<'a: 'b, 'b, T: 'a> MoveElement<'a, 'b, T> {
    #[inline]
    fn take(self) -> T {
        unsafe {
            let i = self.cursor.idx;
            self.cursor.idx += 1;
            self.cursor.del += 1;
            let v = slice::from_raw_parts_mut(self.cursor.vec.as_mut_ptr(), self.cursor.old_len);
            ptr::read(&v[i])
        }
    }
}
