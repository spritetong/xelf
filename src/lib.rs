#[cfg(feature = "collections")]
pub mod collections;
#[cfg(feature = "datetime")]
pub mod datetime;
#[cfg(feature = "db")]
pub mod db;
#[cfg(feature = "ffi")]
pub mod ffi;
#[cfg(feature = "fs")]
pub mod fs;
#[cfg(feature = "future")]
pub mod future;
#[cfg(feature = "json")]
pub mod json;
#[cfg(feature = "net")]
pub mod net;
#[cfg(feature = "serde")]
pub mod serde;
#[cfg(feature = "snowflake")]
pub mod snowflake;
#[cfg(feature = "str")]
pub mod str;
#[cfg(feature = "vec")]
pub mod vec;

pub mod prelude {
    pub use crate::{ok, If};

    #[cfg(feature = "future")]
    pub use ::std::{future::Future, marker::PhantomPinned, pin::Pin};

    #[cfg(feature = "chrono")]
    pub use ::std::time::{self, Duration, Instant, SystemTime, UNIX_EPOCH};

    #[cfg(feature = "ffi")]
    pub use crate::{cstr, ffi::*, uninit_assume_init, zeroed_init};
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
    pub use log::{self, debug, error, info, trace, warn};

    #[cfg(feature = "net")]
    pub use ::std::net::{
        self, IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, ToSocketAddrs,
    };

    #[cfg(feature = "num-traits")]
    pub use num_traits::{self, AsPrimitive, Float, FromPrimitive, PrimInt};

    #[cfg(feature = "serde")]
    pub use crate::serde::*;

    #[cfg(feature = "socket2")]
    pub use crate::net::SocketRsx;
    #[cfg(feature = "socket2")]
    pub use ::socket2;

    #[cfg(feature = "str")]
    pub use crate::str::*;

    #[cfg(feature = "ptr")]
    pub use ::std::{mem, ptr};

    #[cfg(feature = "vec")]
    pub use crate::vec::*;

    pub use ::std::{
        borrow::{self, Borrow, BorrowMut, Cow},
        cell::{self, Cell, RefCell},
        cmp::{max, min, Ordering::*},
        convert::{self, AsMut, AsRef, From, Infallible, Into, TryFrom, TryInto},
        env,
        fmt::{self, Display, Write as FmtWrite},
        iter,
        marker::PhantomData,
        ops::{self, *},
        rc::Rc,
        slice,
        str::{self, FromStr},
        sync::{self, atomic::*, Arc, Mutex as StdMutex, RwLock as StdRwLock},
        thread,
    };

    ////////////////////////////////////////////////////////////////////////////

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
    pub use ::base64::{self, engine::general_purpose::STANDARD as base64_standard, Engine as _};

    #[cfg(feature = "bytes")]
    pub use ::bytes::{self, Buf, BufMut, Bytes, BytesMut};
    #[cfg(feature = "bytestring")]
    pub use ::bytestring::{self, ByteString};

    #[cfg(feature = "cfg-if")]
    pub use ::cfg_if::{self, cfg_if};

    #[cfg(feature = "chrono")]
    pub use ::chrono::{self, prelude::*};

    #[cfg(feature = "crossbeam")]
    pub use crossbeam::{
        self,
        atomic::AtomicCell,
        sync::{Parker, ShardedLock, Unparker},
    };

    #[cfg(feature = "derive_more")]
    pub use ::derive_more::{self, AsMut, AsRef, Deref, DerefMut, Display};

    #[cfg(feature = "dotenv")]
    pub use ::dotenv;

    #[cfg(feature = "env_logger")]
    pub use ::env_logger;

    #[cfg(feature = "flume")]
    pub use ::flume;

    #[cfg(feature = "fs")]
    pub use crate::fs::*;

    #[cfg(feature = "future")]
    pub use crate::future::*;

    #[cfg(feature = "futures")]
    pub use ::futures::{
        self,
        future::{self, join_all, poll_fn, poll_immediate, try_join_all, TryFuture, TryFutureExt},
        stream::{BoxStream, FusedStream, LocalBoxStream, Stream},
        Sink,
    };
    #[cfg(feature = "futures-util")]
    pub use ::futures_util::{self, FutureExt, SinkExt, StreamExt, TryStreamExt};

    #[cfg(feature = "hashlink")]
    pub use ::hashlink;
    #[cfg(all(feature = "hashlink", not(feature = "ritelinked")))]
    pub use ::hashlink::{linked_hash_map, linked_hash_set, LinkedHashMap, LinkedHashSet};
    #[cfg(feature = "ritelinked")]
    pub use ::ritelinked::{self, linked_hash_map, linked_hash_set, LinkedHashMap, LinkedHashSet};

    #[cfg(feature = "hex")]
    pub use ::hex::{self, FromHex, ToHex};
    #[cfg(feature = "hex-literal")]
    pub use ::hex_literal::{self, hex};

    #[cfg(feature = "ipnetwork")]
    pub use ::ipnetwork::{self, IpNetwork, Ipv4Network, Ipv6Network};

    #[cfg(feature = "num_cpus")]
    pub use ::num_cpus;

    #[cfg(feature = "memoffset")]
    pub use ::memoffset::{self, offset_of};

    #[cfg(feature = "maplit")]
    pub use ::maplit::{self, btreemap, btreeset, hashmap, hashset};

    #[cfg(feature = "once_cell")]
    pub use once_cell::{
        self,
        sync::{Lazy, OnceCell},
    };

    #[cfg(feature = "ouroboros")]
    pub use ::ouroboros::{self, self_referencing};

    #[cfg(feature = "parking_lot")]
    pub use ::parking_lot::{self, Mutex as PlMutex, RwLock as PlRwLock};

    #[cfg(feature = "path-absolutize")]
    pub use ::path_absolutize::{self, Absolutize as _};

    #[cfg(feature = "pin-project")]
    pub use ::pin_project::{self, pin_project};

    #[cfg(feature = "regex")]
    pub use ::regex::{self, Regex};

    #[cfg(feature = "rust_decimal")]
    pub use ::rust_decimal::{self, Decimal};

    #[cfg(feature = "rustls")]
    pub use ::rustls;
    #[cfg(feature = "rustls-pemfile")]
    pub use ::rustls_pemfile;

    #[cfg(feature = "sea-orm")]
    pub use ::sea_orm;
    #[cfg(feature = "sqlx")]
    pub use ::sqlx;

    #[cfg(feature = "smallvec")]
    pub use ::smallvec::{self, SmallVec};

    #[cfg(feature = "smart-default")]
    pub use ::smart_default::{self, SmartDefault};

    #[cfg(feature = "snowflake")]
    pub use crate::snowflake::*;

    #[cfg(feature = "strum")]
    pub use ::strum::{
        self, AsRefStr, EnumIter, EnumMessage, FromRepr, IntoEnumIterator, IntoStaticStr,
    };

    #[cfg(feature = "serde")]
    pub use ::serde::{
        self,
        de::{DeserializeOwned, Deserializer},
        ser::Serializer,
        Deserialize, Serialize,
    };
    #[cfg(feature = "serde_json")]
    pub use ::serde_json::{self, json, Number, Number as JsonNumber, Value as Json};
    #[cfg(feature = "serde_json")]
    pub type JsonMap = ::serde_json::Map<String, Json>;
    #[cfg(feature = "serde_repr")]
    pub use ::serde_repr::{self, Deserialize_repr, Serialize_repr};
    #[cfg(feature = "serde_with")]
    pub use ::serde_with::{
        self, base64::Base64, formats::SemicolonSeparator, serde_as, StringWithSeparator,
    };

    #[cfg(feature = "tempfile")]
    pub use ::tempfile::{self, NamedTempFile, TempDir, TempPath};

    #[cfg(feature = "tokio")]
    pub use ::tokio::{
        self,
        io::{AsyncReadExt, AsyncWriteExt},
        sync::{broadcast, mpsc, Mutex, RwLock},
    };
    #[cfg(feature = "tokio-rustls")]
    pub use ::tokio_rustls;
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
    pub use ::zerocopy::{self, AsBytes, FromBytes, FromZeroes};
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

/// Ignore a `Result<T, E>` value, do nothing even if it's an error.
#[macro_export]
macro_rules! ok {
    ($expr:expr $(,)?) => {
        match $expr {
            _ => (),
        }
    };

    ($expr:expr, $value:expr $(,)?) => {
        match $expr {
            _ => $value,
        }
    };
}
