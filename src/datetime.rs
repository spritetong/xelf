use ::chrono::prelude::*;

#[cfg(feature = "sea-orm")]
pub use ::sea_orm::prelude::DateTimeUtc;
#[cfg(not(feature = "sea-orm"))]
pub type DateTimeUtc = DateTime<Utc>;

/// Get the default value for DateTime<Utc>, the Unix Epoch at 1970-01-01T00:00:00Z.
#[inline]
pub fn utc_default() -> DateTimeUtc {
    Utc.timestamp(0, 0)
}

pub fn utc_from_str(s: &str) -> chrono::ParseResult<DateTimeUtc> {
    DateTime::parse_from_rfc3339(s).map(DateTime::<Utc>::from)
}

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

////////////////////////////////////////////////////////////////////////////////

/// Module to serialize and deserialize a **`DateTimeUtc`**
#[cfg(feature = "chrono-serde")]
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

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match Utc.timestamp_opt(value as i64, 0).single() {
                Some(v) => Ok(v),
                _ => Err(de::Error::invalid_type(Unexpected::Unsigned(value), &self)),
            }
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match Utc.timestamp_opt(value, 0).single() {
                Some(v) => Ok(v),
                _ => Err(de::Error::invalid_type(Unexpected::Signed(value), &self)),
            }
        }

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
}

#[cfg(feature = "chrono-serde")]
pub use serde_x_utc::{deserialize as de_x_utc, serialize as ser_x_utc};
#[cfg(feature = "chrono-serde")]
pub use serde_x_utc::{f64::deserialize as de_x_utc_f64, f64::serialize as ser_x_utc_f64};

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    #[cfg(feature = "chrono-serde")]
    #[test]
    fn test_utc_default() {
        use super::*;
        use crate::prelude::*;

        #[derive(Clone, Debug, SmartDefault, Serialize, Deserialize)]
        struct Person {
            #[serde(default = "Default::default")]
            name: String,
            year: i32,
            optional: Option<i32>,
            //#[serde(with = "serde_x_utc", default = "utc_default")]
            #[serde(default = "utc_default")]
            #[default(_code = "utc_default()")]
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
    }
}
