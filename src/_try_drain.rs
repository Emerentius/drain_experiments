pub(crate) struct _TryDrain<'a, T: 'a> {
    pub vec: &'a mut Vec<T>,
    pub finished: bool,
    pub idx: usize,
    pub del: usize,
    pub old_len: usize,
}

impl<'a, T: 'a> _TryDrain<'a, T> {
    pub fn tail_len(&self) -> usize {
        self.old_len - self.idx
    }
}

impl<'a, T: 'a> _TryDrain<'a, T> {
    pub fn new(vec: &'a mut Vec<T>) -> Self {
        let old_len = vec.len();

        // Guard against us getting leaked (leak amplification)
        unsafe { vec.set_len(0); }

        _TryDrain {
            vec,
            finished: false,
            idx: 0,
            del: 0,
            old_len,
        }
    }
}
