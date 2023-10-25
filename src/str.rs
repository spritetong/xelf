#[cfg(feature = "ffi")]
pub use ::std::ffi::{CStr, CString};
pub use ::std::os::raw::c_char;

/// Extension for &str and &String.
pub trait StrRsx {
    /// Convert the string to CString
    #[cfg(feature = "ffi")]
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
    #[cfg(feature = "ffi")]
    fn strlcpy(&self, dst: &mut [c_char]);

    /// Format a string by the template.
    #[cfg(feature = "regex")]
    fn render<F>(&self, f: F) -> std::borrow::Cow<'_, str>
    where
        F: Fn(&str, &mut String);
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
    #[cfg(feature = "ffi")]
    #[inline]
    fn as_cstring(&self) -> CString {
        CString::new(self.as_ref()).unwrap()
    }

    #[cfg(feature = "ffi")]
    fn strlcpy(&self, dst: &mut [c_char]) {
        if !dst.is_empty() {
            let mut len = 0;
            for (a, b) in dst.iter_mut().zip(self.as_ref().bytes()) {
                *a = b as c_char;
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
    /// "{{" is converted into "{", and "}}" is converted into "}".
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

        static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?:\{\{|\}\}|\{:?[[:word:]]+\})").unwrap());
        let replacer = _Replacer(f);
        RE.replace_all(self.as_ref(), replacer)
    }
}

#[cfg(test)]
mod tests {
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
