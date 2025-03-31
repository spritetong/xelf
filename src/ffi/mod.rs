mod handle_set;
mod ptr;

pub use handle_set::{Handle as FfiHandle, HandleSet as FfiHandleSet};
pub use ptr::*;

/// Define a literal C-string with a NUL terminator.
///
/// # Examples
///
/// ```
/// use xelf::cstr;
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

/// Create an object and fill its memory with zero.
#[macro_export]
macro_rules! zeroed_init {
    () => (
        unsafe {
            #[allow(invalid_value)]
            ::std::mem::MaybeUninit::zeroed().assume_init()
        }
    );

    ($x:ident $(,$field:ident: $value:expr)* $(,)?) => (
        unsafe {
            $x = {
                #[allow(invalid_value)]
                ::std::mem::MaybeUninit::zeroed().assume_init()
            };
            $(std::ptr::write(&mut $x.$field, $value);)*
        }
    );
}

/// Create an object and do not initialize its memory.
/// Usually used to create an binary array.
#[macro_export]
macro_rules! uninit_assume_init {
    ($(,)?) => {
        unsafe {
            #[allow(invalid_value)]
            ::std::mem::MaybeUninit::uninit().assume_init()
        }
    };
}
