use ::chrono::prelude::*;

#[cfg(feature = "sea-orm")]
pub use ::sea_orm::prelude::DateTimeUtc;
#[cfg(not(feature = "sea-orm"))]
pub type DateTimeUtc = DateTime<Utc>;

/// Unix timestamp in microseconds
pub type UnixTimeMicros = i64;

/// Duration in microseconds
pub type DurationMicros = i64;

pub trait UnixTimestampXlf: Sized {
    /// Convert days into microseconds.
    fn micros_from_days(&self) -> Self;

    /// Convert hours into microseconds.
    fn micros_from_hours(&self) -> Self;

    /// Convert minutes into microseconds.
    fn micros_from_mins(&self) -> Self;

    /// Convert seconds into microseconds.
    fn micros_from_secs(&self) -> Self;

    /// Convert seconds into microseconds.
    fn micros_from_secs_f64(secs: f64) -> Self;

    /// Convert milliseconds into microseconds.
    fn micros_from_millis(&self) -> Self;

    /// Convert the UNIX timestamp in milliseconds into `DateTimeUtc`.
    fn micros_as_unix_timestamp(&self) -> DateTimeUtc;

    /// Convert the UNIX timestamp in milliseconds into `DateTimeUtc` with error.
    fn micros_as_unix_timestamp_opt(&self) -> chrono::LocalResult<DateTimeUtc>;

    /// Get the current UNIX timestamp in microseconds.
    fn micros_now() -> Self;

    /// Get the UNIX timestamp from a UTC string or a timestamp in microseconds.
    fn micros_from_utc_str(s: impl AsRef<str>) -> Option<Self>;

    /// Convert into a UTC string.
    fn micros_into_utc_str(&self) -> String;
}

/// Get the default value for `DateTime<Utc>`, the Unix Epoch at 1970-01-01T00:00:00Z.
#[inline]
pub fn utc_default() -> DateTimeUtc {
    Utc.timestamp_opt(0, 0).unwrap()
}

/// Parses an RFC 3339 date-and-time string into a `DateTimeUtc` value.
pub fn utc_from_str(s: &str) -> chrono::ParseResult<DateTimeUtc> {
    DateTime::parse_from_rfc3339(s).map(DateTime::<Utc>::from)
}

/// Convert a `DateTimeUtc` value into an RFC 3339 date-and-time string
/// with the format `YYYY-MM-DDTHH:MM:SS.SSSSSSZ`.
pub fn utc_into_str(utc: DateTimeUtc) -> String {
    let n = utc.naive_local();
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:06}Z",
        n.year(),
        n.month(),
        n.day(),
        n.hour(),
        n.minute(),
        n.second(),
        n.nanosecond() / 1000
    )
}

impl UnixTimestampXlf for UnixTimeMicros {
    #[inline]
    fn micros_from_days(&self) -> Self {
        self * (24 * 60 * 60 * 1_000_000)
    }

    #[inline]
    fn micros_from_hours(&self) -> Self {
        self * (60 * 60 * 1_000_000)
    }

    #[inline]
    fn micros_from_mins(&self) -> Self {
        self * (60 * 1_000_000)
    }

    #[inline]
    fn micros_from_secs(&self) -> Self {
        self * 1_000_000
    }

    #[inline]
    fn micros_from_secs_f64(secs: f64) -> Self {
        (secs * 1_000_000.0) as Self
    }

    fn micros_from_millis(&self) -> Self {
        self * 1_000
    }

    fn micros_as_unix_timestamp(&self) -> DateTimeUtc {
        self.micros_as_unix_timestamp_opt()
            .single()
            .unwrap_or_else(utc_default)
    }

    fn micros_as_unix_timestamp_opt(&self) -> chrono::LocalResult<DateTimeUtc> {
        let (mut secs, mut micros) = (self / 1_000_000, self % 1_000_000);
        if micros < 0 {
            secs -= 1;
            micros += 1_000_000;
        }
        Utc.timestamp_opt(secs, micros as u32 * 1_000)
    }

    fn micros_now() -> Self {
        Utc::now().timestamp_micros()
    }

    fn micros_from_utc_str(s: impl AsRef<str>) -> Option<Self> {
        let s = s.as_ref();
        s.parse::<UnixTimeMicros>().ok().or_else(|| {
            DateTime::parse_from_rfc3339(s)
                .ok()
                .map(|x| DateTime::<Utc>::from(x).timestamp_micros())
        })
    }

    fn micros_into_utc_str(&self) -> String {
        utc_into_str(self.micros_as_unix_timestamp())
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Module to serialize and deserialize a **`DateTimeUtc`**
#[cfg(feature = "serde")]
pub mod serde_x_utc {
    use super::*;
    use ::serde::{
        de::{self, Unexpected},
        ser::Serializer,
    };
    use ::std::fmt;

    struct DeUtcVisitor;

    impl<'de> de::Visitor<'de> for DeUtcVisitor {
        type Value = DateTime<Utc>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "a formatted date and time string")
        }

        // from seconds
        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match Utc.timestamp_millis_opt(value as i64).single() {
                Some(v) => Ok(v),
                _ => Err(de::Error::invalid_type(Unexpected::Unsigned(value), &self)),
            }
        }

        // from seconds
        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match Utc.timestamp_millis_opt(value).single() {
                Some(v) => Ok(v),
                _ => Err(de::Error::invalid_type(Unexpected::Signed(value), &self)),
            }
        }

        // from seconds as float
        fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let secs = value.floor();
            match Utc
                .timestamp_opt(secs as i64, ((value - secs) * 1000000000.) as u32)
                .single()
            {
                Some(v) => Ok(v),
                _ => Err(de::Error::invalid_type(Unexpected::Float(value), &self)),
            }
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match DateTime::parse_from_rfc3339(value) {
                Ok(v) => Ok(DateTime::<Utc>::from(v)),
                _ => Err(de::Error::invalid_type(Unexpected::Str(value), &self)),
            }
        }
    }

    struct DeUtcMicrosVisitor;

    impl<'de> de::Visitor<'de> for DeUtcMicrosVisitor {
        type Value = UnixTimeMicros;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "a formatted date and time string")
        }

        // from microseconds
        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            UnixTimeMicros::try_from(value)
                .map_err(|_| de::Error::invalid_type(Unexpected::Unsigned(value), &self))
        }

        // from microseconds
        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value)
        }

        // from microseconds as float
        fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value as i64)
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match DateTime::parse_from_rfc3339(value) {
                Ok(v) => Ok(DateTime::<Utc>::from(v).timestamp_micros()),
                _ => Err(de::Error::invalid_type(Unexpected::Str(value), &self)),
            }
        }
    }

    /// Function to serializing a **`DateTimeUtc`**
    pub fn serialize<S>(utc: &DateTimeUtc, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&utc_into_str(*utc))
    }

    /// Function to deserializing a **`DateTimeUtc`**
    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTimeUtc, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_any(DeUtcVisitor)
    }

    pub mod f64 {
        use super::*;

        /// Function to serializing a **`DateTimeUtc`**
        pub fn serialize<S>(utc: &DateTimeUtc, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_f64(utc.timestamp_micros() as f64 / 1000000.)
        }

        /// Function to deserializing a **`DateTimeUtc`**
        pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTimeUtc, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            deserializer.deserialize_f64(DeUtcVisitor)
        }
    }

    pub mod micros {
        use super::*;

        /// Function to serializing a **`DateTimeUtc`**
        pub fn serialize<S>(micros: UnixTimeMicros, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_i64(micros)
        }

        /// Function to deserializing a **`DateTimeUtc`**
        pub fn deserialize<'de, D>(deserializer: D) -> Result<UnixTimeMicros, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            deserializer.deserialize_i64(DeUtcMicrosVisitor)
        }
    }
}

#[cfg(feature = "serde")]
pub use serde_x_utc::{deserialize as de_x_utc, serialize as ser_x_utc};
#[cfg(feature = "serde")]
pub use serde_x_utc::{f64::deserialize as de_x_utc_f64, f64::serialize as ser_x_utc_f64};
#[cfg(feature = "serde")]
pub use serde_x_utc::{
    micros::deserialize as de_x_utc_micros, micros::serialize as ser_x_utc_micros,
};

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    #[cfg(feature = "serde")]
    #[test]
    fn test_utc_default() {
        use super::*;
        use crate::prelude::*;

        #[derive(Clone, Debug, SmartDefault, Serialize, Deserialize)]
        struct Person {
            #[serde(default)]
            name: String,
            year: i32,
            optional: Option<i32>,
            //#[serde(with = "serde_x_utc", default = "utc_default")]
            #[serde(default = "utc_default")]
            #[default(utc_default())]
            birth_on: DateTime<Utc>,
        }

        let jsn = json!({
            "year": 32,
            "birth_on": utc_into_str(Utc::now()),
        });
        let a: Person = serde_json::from_value(jsn).unwrap();
        println!("{:?}", &a);

        let s = serde_json::to_string(&a).unwrap();
        println!("{}", &s);

        let utc = utc_from_str("2022-01-01T00:00:01.345677Z").unwrap();
        assert_eq!(
            UnixTimeMicros::micros_from_utc_str("2022-01-01T00:00:01.345677Z"),
            Some(utc.timestamp_micros())
        );
        assert_eq!(
            UnixTimeMicros::micros_from_utc_str(utc.timestamp_micros().to_string()),
            Some(utc.timestamp_micros())
        );
    }
}
