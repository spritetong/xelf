mod arc_handle_set;
mod parker_cache;

pub use arc_handle_set::*;
pub use parker_cache::*;

/// Define a literal C-string with a NUL terminator.
///
/// # Examples
///
/// ```
/// use rsx::cstr;
///
/// let name: &std::ffi::CStr = cstr!("John");
///
/// assert_eq!(name.to_str(), Ok("John"));
/// assert_eq!(unsafe { *name.as_ptr().add(4) }, 0);
/// ```
#[macro_export]
macro_rules! cstr {
    ($s:literal) => {
        unsafe { ::std::mem::transmute::<_, &::std::ffi::CStr>(concat!($s, "\0")) }
    };
}
