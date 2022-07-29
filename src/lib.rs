#[cfg(feature = "collections")]
pub mod collections;
#[cfg(feature = "datetime")]
pub mod datetime;
#[cfg(feature = "db")]
pub mod db;
#[cfg(feature = "ffi")]
pub mod ffi;
#[cfg(feature = "json")]
pub mod json;
#[cfg(feature = "num")]
pub mod num;
#[cfg(feature = "ptr")]
pub mod ptr;
#[cfg(feature = "serde")]
pub mod serde;
#[cfg(feature = "signal")]
pub mod signal;
#[cfg(feature = "str")]
pub mod str;
#[cfg(feature = "vec")]
pub mod vec;

pub mod prelude {
    pub use crate::{
        ok_or, ok_or_break, ok_or_continue, ok_or_return, some_or, some_or_break, some_or_continue,
        some_or_return, uninit_assume_init, zeroed_init, If,
    };

    #[cfg(feature = "future")]
    pub use ::std::{future::Future, pin::Pin};

    #[cfg(feature = "chrono")]
    pub use ::std::time::{self, Duration, Instant, SystemTime, UNIX_EPOCH};

    #[cfg(feature = "ffi")]
    pub use crate::{cstr, ffi::*};
    #[cfg(feature = "ffi")]
    pub use ::std::ffi::{self, CStr, CString};

    #[cfg(feature = "io")]
    pub use ::std::io::{self, prelude::*};
    #[cfg(feature = "fs")]
    pub use ::std::{
        fs,
        path::{self, Path, PathBuf},
    };

    #[cfg(feature = "collections")]
    pub use crate::collections::*;
    #[cfg(feature = "collections")]
    pub use ::std::collections::{self, BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

    #[cfg(feature = "datetime")]
    pub use crate::datetime::*;

    #[cfg(feature = "json")]
    pub use crate::json::*;

    #[cfg(feature = "log")]
    pub use log::{self, debug, error, info, trace};

    #[cfg(feature = "net")]
    pub use ::std::net::{
        self, IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, ToSocketAddrs,
    };

    #[cfg(feature = "num")]
    pub use crate::num::{Float as _, Num as _};

    #[cfg(feature = "serde")]
    pub use crate::serde::*;

    #[cfg(feature = "socket2")]
    pub use ::socket2;

    #[cfg(feature = "str")]
    pub use crate::str::*;

    #[cfg(feature = "ptr")]
    pub use crate::ptr::*;
    #[cfg(feature = "ptr")]
    pub use ::std::{mem, ptr};

    #[cfg(feature = "vec")]
    pub use crate::vec::*;

    pub use ::std::{
        borrow::{self, Borrow, BorrowMut, Cow},
        cell::{self, Cell, RefCell},
        cmp::{max, min},
        convert::{self, AsMut, AsRef, From, Infallible, Into, TryFrom, TryInto},
        env,
        fmt::{self, Display, Write as FmtWrite},
        iter,
        ops::{self, *},
        rc::Rc,
        slice,
        str::{self, FromStr},
        sync::{self, atomic::*, Arc, Mutex as StdMutex, RwLock as StdRwLock},
        thread,
    };

    ////////////////////////////////////////////////////////////////////////////

    #[cfg(feature = "actix")]
    pub use ::actix;
    #[cfg(feature = "actix-files")]
    pub use ::actix_files;
    #[cfg(feature = "actix-multipart")]
    pub use ::actix_multipart;
    #[cfg(feature = "actix-service")]
    pub use ::actix_service;
    #[cfg(feature = "actix-web")]
    pub use ::actix_web;
    #[cfg(feature = "actix-web-actors")]
    pub use ::actix_web_actors;
    #[cfg(feature = "actix-web-httpauth")]
    pub use ::actix_web_httpauth;

    #[cfg(feature = "ahash")]
    pub use ::ahash::{self, AHashMap, AHashSet};
    #[cfg(any(feature = "ahash", feature = "collections"))]
    pub use ::std::hash::{BuildHasher, Hash};

    #[cfg(feature = "arc-swap")]
    pub use arc_swap::{self, ArcSwap, ArcSwapAny};

    #[cfg(feature = "async-stream")]
    pub use ::async_stream;
    #[cfg(feature = "async-trait")]
    pub use ::async_trait::{self, async_trait};

    #[cfg(feature = "base64")]
    pub use ::base64;

    #[cfg(feature = "bytes")]
    pub use ::bytes::{self, Buf, BufMut, Bytes, BytesMut};
    #[cfg(feature = "bytestring")]
    pub use ::bytestring::{self, ByteString};

    #[cfg(feature = "chrono")]
    pub use ::chrono::{self, prelude::*};

    #[cfg(feature = "crossbeam")]
    pub use crossbeam::{
        self,
        atomic::AtomicCell,
        sync::{Parker, ShardedLock},
    };

    #[cfg(feature = "derive-more")]
    pub use ::derive_more::{self, AsMut, AsRef, Deref, DerefMut, Display};

    #[cfg(feature = "dotenv")]
    pub use ::dotenv;

    #[cfg(feature = "env-logger")]
    pub use ::env_logger;

    #[cfg(feature = "flume")]
    pub use ::flume;

    #[cfg(feature = "futures")]
    pub use ::futures::{
        self,
        future::{self, join_all, try_join_all, TryFuture},
        stream::{BoxStream, LocalBoxStream, Stream},
    };
    #[cfg(feature = "futures-util")]
    pub use ::futures_util::{self, SinkExt, StreamExt};

    #[cfg(feature = "hex")]
    pub use ::hex::{self, FromHex, ToHex};
    #[cfg(feature = "hex-literal")]
    pub use ::hex_literal::{self, hex};

    #[cfg(feature = "ipnetwork")]
    pub use ::ipnetwork::{self, IpNetwork, Ipv4Network, Ipv6Network};

    #[cfg(feature = "num-cpus")]
    pub use ::num_cpus;

    #[cfg(feature = "memoffset")]
    pub use ::memoffset::{self, offset_of};

    #[cfg(feature = "maplit")]
    pub use ::maplit::{self, btreemap, btreeset, hashmap, hashset};

    #[cfg(feature = "num-enum")]
    pub use ::num_enum::{self, TryFromPrimitive};

    #[cfg(feature = "once-cell")]
    pub use once_cell::{
        self,
        sync::{Lazy, OnceCell},
    };

    #[cfg(feature = "ouroboros")]
    pub use ::ouroboros::{self, self_referencing};

    #[cfg(feature = "parking-lot")]
    pub use ::parking_lot;

    #[cfg(feature = "pin-project")]
    pub use ::pin_project::{self, pin_project};

    #[cfg(feature = "regex")]
    pub use ::regex::{self, Regex};

    #[cfg(feature = "ritelinked")]
    pub use ::ritelinked::{self, linked_hash_map, linked_hash_set, LinkedHashMap, LinkedHashSet};

    #[cfg(feature = "rust-decimal")]
    pub use ::rust_decimal::{self, Decimal};

    #[cfg(feature = "rustls")]
    pub use ::rustls;
    #[cfg(feature = "rustls-pemfile")]
    pub use ::rustls_pemfile;

    #[cfg(feature = "sea-orm")]
    pub use ::sea_orm::{
        self,
        strum::{AsRefStr as _, EnumMessage as _, IntoEnumIterator as _},
    };
    #[cfg(feature = "sqlx")]
    pub use ::sqlx;

    #[cfg(feature = "smallvec")]
    pub use ::smallvec::{self, SmallVec};

    #[cfg(feature = "smart-default")]
    pub use ::smart_default::{self, SmartDefault};
    #[cfg(feature = "strum")]
    pub use ::strum::{self, AsRefStr, EnumIter, EnumMessage, IntoEnumIterator, IntoStaticStr};

    #[cfg(feature = "serde")]
    pub use ::serde::{
        self,
        de::{DeserializeOwned, Deserializer},
        ser::Serializer,
        Deserialize, Serialize,
    };
    #[cfg(feature = "serde-json")]
    pub use ::serde_json::{self, json, Number, Number as JsonNumber, Value as Json};
    #[cfg(feature = "serde-json")]
    pub type JsonMap = ::serde_json::Map<String, Json>;
    #[cfg(feature = "serde-repr")]
    pub use ::serde_repr::{self, Deserialize_repr, Serialize_repr};

    #[cfg(feature = "shellexpand")]
    pub use ::shellexpand;

    #[cfg(feature = "tempfile")]
    pub use ::tempfile::{self, NamedTempFile, TempDir, TempPath};

    #[cfg(feature = "tokio")]
    pub use ::tokio::{
        self,
        io::{AsyncReadExt, AsyncWriteExt},
        sync::{broadcast, mpsc, Mutex, RwLock},
    };
    #[cfg(feature = "tokio-rustls")]
    pub use ::tokio_rustls::{self, webpki};
    #[cfg(feature = "tokio-stream")]
    pub use ::tokio_stream::{
        self,
        wrappers::{errors::BroadcastStreamRecvError, BroadcastStream, ReceiverStream},
    };
    #[cfg(feature = "tokio-util")]
    pub use ::tokio_util::{self, sync::CancellationToken};

    #[cfg(feature = "url")]
    pub use ::url::{self, Url};

    #[cfg(feature = "zerocopy")]
    pub use ::zerocopy::{self, AsBytes, FromBytes};
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
#[cfg(feature = "ffi")]
#[macro_export]
macro_rules! cstr {
    ($s:literal) => {
        unsafe { ::std::mem::transmute::<_, &::std::ffi::CStr>(concat!($s, "\0")) }
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
    #[cfg(feature = "ffi")]
    #[test]
    fn test_cstr_macro() {
        let name: &std::ffi::CStr = cstr!("John");

        assert_eq!(name.to_str(), Ok("John"));
        assert_eq!(unsafe { *name.as_ptr().add(4) }, 0);
    }
}
