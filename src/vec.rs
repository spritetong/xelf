/// Extension for vectors which item implements Copy trait.
pub trait VecRsx {
    /// Constructs a new Vec<T> with the specified length, and all elements are not initialized.
    fn with_length(len: usize) -> Self;

    /// Resize the vector but do not initialize any new element.
    fn resize_uninit(&mut self, new_len: usize);
}

impl<T: Sized + Copy + Default> VecRsx for Vec<T> {
    #[inline]
    fn with_length(len: usize) -> Self {
        let mut v = Self::with_capacity(len);
        unsafe {
            v.set_len(len);
        }
        v
    }

    #[inline]
    fn resize_uninit(&mut self, new_len: usize) {
        if new_len > self.len() {
            self.reserve(new_len - self.len());
        }
        unsafe {
            self.set_len(new_len);
        }
    }
}
