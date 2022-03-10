pub mod ptr;
pub mod str;

#[cfg(feature = "collections")]
pub mod collections;

#[cfg(feature = "serde")]
pub mod json;
#[cfg(feature = "serde")]
pub mod num;
#[cfg(feature = "serde")]
pub mod serde;

pub mod prelude {
    pub use crate::str::*;
    pub use crate::{
        cstr, ok_or, ok_or_break, ok_or_continue, ok_or_return, some_or, some_or_break,
        some_or_continue, some_or_return, uninit_assume_init, zeroed_init, If,
    };

    #[cfg(feature = "bytes")]
    pub use ::bytes_::{self as bytes, Buf, BufMut, Bytes, BytesMut};
    #[cfg(feature = "bytes")]
    pub use ::bytestring::{self, ByteString};

    #[cfg(feature = "chrono")]
    pub use ::chrono::{self, prelude::*, Duration as ChronoDuration};
    #[cfg(feature = "chrono")]
    pub use ::std::time::{self, Duration, Instant, SystemTime, UNIX_EPOCH};

    #[cfg(feature = "derive")]
    pub use ::derive_more::{self, Deref, DerefMut};
    #[cfg(feature = "derive")]
    pub use ::smart_default::{self, SmartDefault};
    #[cfg(feature = "strum")]
    pub use ::strum::{self, EnumIter, EnumMessage, IntoEnumIterator, IntoStaticStr};

    #[cfg(feature = "collections")]
    pub use crate::collections::*;
    #[cfg(feature = "collections")]
    pub use ::ahash::{self, AHashMap, AHashSet};
    #[cfg(feature = "collections")]
    pub use ::ritelinked::{self, linked_hash_map, linked_hash_set, LinkedHashMap, LinkedHashSet};
    #[cfg(feature = "collections")]
    pub use ::std::collections::{self, BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

    #[cfg(feature = "fs")]
    pub use ::std::{
        fs,
        io::{self, prelude::*},
        path::{self, Path, PathBuf},
    };
    #[cfg(feature = "fs")]
    pub use ::tempfile::{self, NamedTempFile, TempDir, TempPath};

    #[cfg(feature = "hex")]
    pub use ::hex_::{self as hex, FromHex, ToHex};
    #[cfg(feature = "hex")]
    pub use ::hex_literal::{self, hex};

    #[cfg(feature = "once_cell")]
    pub use once_cell::{
        self,
        sync::{Lazy, OnceCell},
    };

    #[cfg(feature = "log")]
    pub use dotenv;
    #[cfg(feature = "log")]
    pub use env_logger;
    #[cfg(feature = "log")]
    pub use log_::{self as log, debug, error, info, trace};

    #[cfg(feature = "net")]
    pub use ::ipnetwork::{self, IpNetwork, Ipv4Network, Ipv6Network};
    #[cfg(feature = "net")]
    pub use ::std::net::{
        self, IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, ToSocketAddrs,
    };

    #[cfg(feature = "rustls")]
    pub use rustls_ as rustls;
    #[cfg(feature = "rustls")]
    pub use rustls_pemfile;

    #[cfg(feature = "serde")]
    pub use crate::json::*;
    #[cfg(feature = "serde")]
    pub use crate::num::{Float as _, Num as _};
    #[cfg(feature = "serde")]
    pub use crate::serde::*;
    #[cfg(feature = "serde")]
    pub use ::serde_::{
        self as serde,
        de::{DeserializeOwned, Deserializer},
        Deserialize, Serialize,
    };
    #[cfg(feature = "serde")]
    pub use ::serde_json::{self, json, Map as JsonMap, Number, Value};
    #[cfg(feature = "serde")]
    pub use ::serde_repr::{self, Deserialize_repr, Serialize_repr};

    #[cfg(feature = "sync")]
    pub use arc_swap::{self, ArcSwap, ArcSwapAny};
    #[cfg(feature = "sync")]
    pub use crossbeam::{
        self,
        atomic::AtomicCell,
        sync::{Parker, ShardedLock},
    };
    #[cfg(feature = "sync")]
    pub use std::{
        sync::{atomic::*, Arc, Mutex as StdMutex, RwLock as StdRwLock},
        thread,
    };

    #[cfg(feature = "tokio")]
    pub use ::futures::{
        self,
        future::{join_all, try_join_all, FutureExt},
        StreamExt, TryStreamExt,
    };
    #[cfg(feature = "tokio")]
    pub use ::std::pin::Pin;
    #[cfg(feature = "tokio")]
    pub use ::tokio_::{
        self as tokio,
        sync::{broadcast, mpsc, Mutex, RwLock},
    };
    #[cfg(feature = "tokio")]
    pub use ::tokio_stream::{
        self,
        wrappers::{errors::BroadcastStreamRecvError, BroadcastStream, ReceiverStream},
    };

    #[cfg(feature = "unsafe")]
    pub use crate::ptr::*;
    #[cfg(feature = "unsafe")]
    pub use ::memoffset::{self, offset_of};
    #[cfg(feature = "unsafe")]
    pub use ::std::{mem, ptr};
    #[cfg(feature = "unsafe")]
    pub use ::zerocopy::{self, AsBytes};

    pub use ::std::{
        borrow::{self, Borrow, BorrowMut, Cow},
        cell::{self, Cell, RefCell},
        cmp::{max, min},
        convert::{self, AsMut, AsRef, From, Into, TryFrom, TryInto},
        env,
        fmt::{self, Display},
        iter,
        ops::{self, * /*, Deref, DerefMut */},
        rc::Rc,
        slice,
        str::{self, FromStr},
    };
}

/// Create an object and fill its memory with zero.
#[macro_export]
macro_rules! zeroed_init {
    () => (
        unsafe {
            #[allow(invalid_value)]
            std::mem::MaybeUninit::zeroed().assume_init()
        }
    );

    ($x:ident $(,$field:ident: $value:expr)* $(,)?) => (
        unsafe {
            #[allow(invalid_value)]
            $x = std::mem::MaybeUninit::zeroed().assume_init();
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
            std::mem::MaybeUninit::uninit().assume_init()
        }
    };
}

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
        unsafe { std::mem::transmute::<_, &std::ffi::CStr>(concat!($s, "\0")) }
    };
}

/// Simplify an "if" expression
///
/// # Examples
///
/// ```
/// use rsx::If;
///
/// assert_eq!(If!(true, "A", "B"), "A");
/// assert_eq!(If!(false, "A", "B"), "B");
/// ```
#[macro_export]
macro_rules! If {
    ($condition:expr, $if_true:expr, $if_false:expr $(,)?) => {
        if $condition {
            $if_true
        } else {
            $if_false
        }
    };
}

/// Unwrap an Result<T, E> value, execute an expression on error.
#[macro_export]
macro_rules! ok_or {
    ($expr:expr, $expr_on_err:expr $(,)?) => {
        match $expr {
            Ok(v) => v,
            _ => $expr_on_err,
        }
    };
}

/// Unwrap an Result<T, E> value, return from the current function with a result on error.
#[macro_export]
macro_rules! ok_or_return {
    ($expr:expr $(,)?) => {
        ok_or!($expr, return)
    };

    ($expr:expr, $value:expr $(,)?) => {
        ok_or!($expr, return $value)
    };
}

/// Unwrap an Result<T, E> value, break the current loop with a value on error.
#[macro_export]
macro_rules! ok_or_break {
    ($expr:expr $(,)?) => {
        ok_or!($expr, break)
    };

    ($expr:expr, $value:expr $(,)?) => {
        ok_or!($expr, break $value)
    };
}

/// Unwrap an Result<T, E> value, continue the current loop on error.
#[macro_export]
macro_rules! ok_or_continue {
    ($expr:expr $(,)?) => {
        ok_or!($expr, continue)
    };
}

/// Unwrap an Option<T> value, execute an expression on None.
#[macro_export]
macro_rules! some_or {
    ($expr:expr, $expr_on_err:expr $(,)?) => {
        match $expr {
            Some(v) => v,
            _ => $expr_on_err,
        }
    };
}

/// Unwrap an Option<T> value, return from the current function with on None.
#[macro_export]
macro_rules! some_or_return {
    ($expr:expr $(,)?) => {
        some_or!($expr, return)
    };

    ($expr:expr, $value:expr $(,)?) => {
        some_or!($expr, return $value)
    };
}

/// Unwrap an Option<T> value, break the current loop with a value on None.
#[macro_export]
macro_rules! some_or_break {
    ($expr:expr $(,)?) => {
        some_or!($expr, break)
    };

    ($expr:expr, $value:expr $(,)?) => {
        some_or!($expr, break $value)
    };
}

/// Unwrap an Option<T> value, continue the current loop on None.
#[macro_export]
macro_rules! some_or_continue {
    ($expr:expr $(,)?) => {
        some_or!($expr, continue)
    };
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    #[test]
    fn test_cstr_macro() {
        let name: &std::ffi::CStr = cstr!("John");

        assert_eq!(name.to_str(), Ok("John"));
        assert_eq!(unsafe { *name.as_ptr().add(4) }, 0);
    }
}
