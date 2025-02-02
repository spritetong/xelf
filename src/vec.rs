/// Extension for vectors which item implements Copy trait.
pub trait VecXlf {
    /// Constructs a new `Vec<T>` with the specified length, and all elements are not initialized.
    ///
    /// **Dangerous!** The caller is responsible for initializing the new elements.
    fn with_length(len: usize) -> Self;

    /// Resize the vector but do not initialize any new element.
    ///
    /// **Dangerous!** The caller is responsible for initializing the new elements.
    fn resize_uninit(&mut self, new_len: usize);
}

impl<T: Sized + Copy> VecXlf for Vec<T> {
    #[inline]
    fn with_length(len: usize) -> Self {
        let mut v = Self::with_capacity(len);
        #[allow(clippy::uninit_vec)]
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
        #[allow(clippy::uninit_vec)]
        unsafe {
            self.set_len(new_len);
        }
    }
}
