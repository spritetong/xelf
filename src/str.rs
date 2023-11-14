#[cfg(feature = "ffi")]
pub use ::std::ffi::{CStr, CString};
pub use ::std::os::raw::c_char;

/// Extension for &str and &String.
pub trait StrRsx {
    /// Convert the string to CString
    #[cfg(feature = "ffi")]
    fn to_cstring(&self) -> CString;

    /// Copy string to a C-str slice.
    ///
    /// Like GNU
    ///
    /// ```C++
    /// size_t strlcpy(char *dst, const char *src, size_t size);
    /// ```
    ///
    /// Take the full size of the buffer (not just the length)
    /// and guarantee to NUL-terminate destination (as long as size is larger than 0).
    #[cfg(feature = "ffi")]
    fn strlcpy(&self, dst: &mut [u8]);

    /// Format a string by the template.
    #[cfg(feature = "regex")]
    fn render<F>(&self, f: F) -> std::borrow::Cow<'_, str>
    where
        F: Fn(&str, &mut String);
}

pub trait BytesRsx {
    /// Try to convert a nul-terminated bytes in UTF-8 to a string slice.
    ///
    /// Allow the nul byte to be in any position of the string.
    ///
    /// If the string does not contain a nul byte, the entire string is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use rsx::str::BytesRsx;
    ///
    /// assert_eq!(b"a\0bc".to_utf8_with_nul(), Ok("a"));
    /// assert_eq!(b"abc".to_utf8_with_nul(), Ok("abc"));
    /// ```
    fn to_utf8_with_nul(&self) -> Result<&str, std::str::Utf8Error>;

    /// Try to convert a nul-terminated bytes in UTF-8 to a string.
    ///
    /// Allow the nul byte at position of the string.
    ///
    /// If the string does not contain a nul byte, the entire string is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use rsx::str::BytesRsx;
    ///
    /// assert_eq!(b"a\0bc".to_utf8_string_with_nul(), Ok("a".to_owned()));
    /// assert_eq!(b"abc".to_utf8_string_with_nul(), Ok("abc".to_owned()));
    /// ```
    fn to_utf8_string_with_nul(&self) -> Result<String, std::str::Utf8Error>;

    /// Convert a nul-terminated bytes in UTF-8 to a string.
    ///
    /// Allow the nul byte at any position of the string.
    ///
    /// If the string does not contain a nul byte, the entire string is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use rsx::str::BytesRsx;
    ///
    /// assert_eq!(b"a\0bc".to_utf8_lossy_with_nul(), "a");
    /// assert_eq!(b"abc".to_utf8_lossy_with_nul(), "abc");
    /// ```
    fn to_utf8_lossy_with_nul(&self) -> std::borrow::Cow<'_, str>;

    /// Try to convert a nul-terminated bytes in UTF-8 to a CStr slice.
    ///
    /// Allow the nul byte at any position of the string.
    ///
    /// If the string does not contain a nul byte, return `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rsx::str::BytesRsx;
    /// use std::ffi::CStr;
    ///
    /// assert_eq!(b"a\0bc".to_cstr_with_nul(), Some(CStr::from_bytes_with_nul(b"a\0").unwrap()));
    /// assert_eq!(b"abc".to_cstr_with_nul(), None);
    /// ```
    #[cfg(feature = "ffi")]
    fn to_cstr_with_nul(&self) -> Option<&CStr>;
}

impl<T: AsRef<str>> StrRsx for T {
    #[cfg(feature = "ffi")]
    #[inline]
    fn to_cstring(&self) -> CString {
        CString::new(self.as_ref()).unwrap()
    }

    #[cfg(feature = "ffi")]
    fn strlcpy(&self, dst: &mut [u8]) {
        if !dst.is_empty() {
            let mut len = 0;
            for (a, b) in dst.iter_mut().zip(self.as_ref().bytes()) {
                *a = b;
                if b == 0 {
                    return;
                }
                len += 1;
            }
            dst[std::cmp::min(len, dst.len() - 1)] = 0;
        }
    }

    /// Render a string by the template with named arguments.
    ///
    /// A name of an argument is wrapped by {}, like "{name}".
    ///
    /// "{{" will be converted into "{", and "}}" well be converted into "}".
    ///
    /// # Arguments
    ///
    /// * f: function to render.
    ///
    /// # Exmaples
    ///
    /// ```
    /// use rsx::str::StrRsx;
    ///
    /// assert_eq!("{{".render(|_, _| ()), "{");
    /// assert_eq!("}}".render(|_, _| ()), "}");
    /// assert_eq!(
    ///     "{:a}-{b}".render(|key, dst| {
    ///         match key {
    ///             ":a" => dst.push_str("111"),
    ///             "b" => dst.push_str("222"),
    ///             _ => unimplemented!(),
    ///         }
    ///     }),
    ///     "111-222"
    /// );
    /// ```
    ///
    #[cfg(feature = "regex")]
    fn render<F>(&self, f: F) -> std::borrow::Cow<'_, str>
    where
        F: Fn(&str, &mut String),
    {
        use once_cell::sync::Lazy;
        use regex::{Regex, Replacer};

        struct _Replacer<F: Fn(&str, &mut String)>(F);
        impl<F: Fn(&str, &mut String)> Replacer for _Replacer<F> {
            fn replace_append(&mut self, caps: &regex::Captures<'_>, dst: &mut String) {
                let name = caps.get(0).unwrap().as_str();
                match name {
                    "{{" => dst.push('{'),
                    "}}" => dst.push('}'),
                    _ => self.0(&name[1..name.len() - 1], dst),
                }
            }
        }

        static RE: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"(?:\{\{|\}\}|\{:?[[:word:]]+\})").unwrap());
        let replacer = _Replacer(f);
        RE.replace_all(self.as_ref(), replacer)
    }
}

impl<T: AsRef<[u8]>> BytesRsx for T {
    fn to_utf8_with_nul(&self) -> Result<&str, std::str::Utf8Error> {
        let data = self.as_ref();
        // default to length if no `\0` present
        let nul_range_end = data.iter().position(|&c| c == 0).unwrap_or(data.len());
        std::str::from_utf8(&data[..nul_range_end])
    }

    fn to_utf8_string_with_nul(&self) -> Result<String, std::str::Utf8Error> {
        self.to_utf8_with_nul().map(|s| s.to_owned())
    }

    fn to_utf8_lossy_with_nul(&self) -> std::borrow::Cow<'_, str> {
        let data = self.as_ref();
        // default to length if no `\0` present
        let nul_range_end = data.iter().position(|&c| c == 0).unwrap_or(data.len());
        String::from_utf8_lossy(&data[..nul_range_end])
    }

    #[cfg(feature = "ffi")]
    fn to_cstr_with_nul(&self) -> Option<&CStr> {
        let data = self.as_ref();
        data.iter()
            .position(|&c| c == 0)
            .and_then(|x| CStr::from_bytes_with_nul(&data[..=x]).ok())
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[cfg(feature = "regex")]
    #[test]
    fn test_template_render() {
        assert_eq!("{{".render(|_, _| ()), "{");

        assert_eq!("}}".render(|_, _| ()), "}");

        assert_eq!(
            "{a}-{b}".render(|key, dst| {
                match key {
                    "a" => dst.push_str("111"),
                    "b" => dst.push_str("222"),
                    _ => unimplemented!(),
                }
            }),
            "111-222"
        );

        assert_eq!(
            "{{{:a_a}}}{bb}{{cc}}".render(|key, dst| {
                match key {
                    ":a_a" => dst.push_str("AAAA"),
                    "bb" => dst.push_str("BBBB"),
                    _ => unimplemented!(),
                }
            }),
            "{AAAA}BBBB{cc}"
        );
    }
}
