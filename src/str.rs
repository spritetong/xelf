pub use ::std::ffi::{CStr, CString};
pub use ::std::os::raw::c_char;

/// Extension for &str and &String.
pub trait StrRsx {
    /// Convert the string to CString
    fn as_cstring(&self) -> CString;

    /// Copy string to a C-str slice.
    ///
    /// Like GNU
    ///
    /// ```C++
    /// size_t strlcpy(char *dst, const char *src, size_t size);
    /// ```
    ///
    /// take the full size of the buffer (not just the length)
    /// and guarantee to NUL-terminate destination (as long as size is larger than 0).
    fn strlcpy(&self, dst: &mut [c_char]);
}

/// Extension for String.
pub trait StringRsx {
    /// Assign a str slice to this string, replace the whole old content.
    /// 
    /// # Arguments
    /// 
    /// * source: the source string to copy from.
    /// 
    /// # Exmaples
    /// 
    /// ```
    /// use rsx::str::*;
    /// 
    /// let mut s = String::from("abc");
    /// s.assign("12345");
    /// 
    /// assert_eq!(s, "12345");
    /// ```
    /// 
    fn assign<T: AsRef<str>>(&mut self, source: T);
}

impl StringRsx for String {
    #[inline]
    fn assign<T: AsRef<str>>(&mut self, source: T) {
        self.clear();
        *self += source.as_ref();
    }
}

impl<T: AsRef<str>> StrRsx for T {
    #[inline]
    fn as_cstring(&self) -> CString {
        CString::new(self.as_ref()).unwrap()
    }

    #[inline]
    fn strlcpy(&self, dst: &mut [c_char]) {
        strlcpy(dst, self.as_ref());
    }
}

fn strlcpy(dst: &mut [c_char], src: &str) {
    if dst.len() > 0 {
        let mut len = 0;
        for (a, b) in dst.iter_mut().zip(src.bytes()) {
            *a = b as c_char;
            if b == 0 {
                return;
            }
            len += 1;
        }
        dst[std::cmp::min(len, dst.len() - 1)] = 0;
    }
}
