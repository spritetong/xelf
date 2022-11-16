#[cfg(feature = "num")]
use crate::num::{Float, Num};
use serde::{de, ser};
use std::{fmt, marker::PhantomData, str::FromStr};

#[cfg(feature = "num")]
#[derive(Default)]
struct DeNumVisitor<T: Num> {
    _phantom: std::marker::PhantomData<T>,
}

#[cfg(feature = "num")]
impl<'de, T: Num> de::Visitor<'de> for DeNumVisitor<T> {
    type Value = T;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer or a string")
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Self::Value::from_i64(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Self::Value::from_u64(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        v.parse::<Self::Value>()
            .map_err(|_| E::invalid_value(de::Unexpected::Str(v), &self))
    }
}

/// Function to deserializing a string to an **`integer`**
#[cfg(feature = "num")]
pub fn de_x_num<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: de::Deserializer<'de>,
    T: Num,
{
    deserializer.deserialize_any(DeNumVisitor::<T>::default())
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "num")]
#[derive(Default)]
struct DeFloatVisitor<T: Float> {
    _phantom: std::marker::PhantomData<T>,
}

#[cfg(feature = "num")]
impl<'de, T: Float> de::Visitor<'de> for DeFloatVisitor<T> {
    type Value = T;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer or a string")
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Self::Value::from_f64(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Self::Value::from_i64(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Self::Value::from_u64(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        v.parse::<Self::Value>()
            .map_err(|_| E::invalid_value(de::Unexpected::Str(v), &self))
    }
}

/// Function to deserializing a string to an **`float`**
#[cfg(feature = "num")]
pub fn de_x_float<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: de::Deserializer<'de>,
    T: Float,
{
    deserializer.deserialize_any(DeFloatVisitor::<T>::default())
}

////////////////////////////////////////////////////////////////////////////////

struct DeBoolVisitor;

impl<'de> de::Visitor<'de> for DeBoolVisitor {
    type Value = bool;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a bool, a integer (0/1) or a string")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(v)
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match v {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(E::invalid_value(de::Unexpected::Signed(v), &self)),
        }
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match v {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(E::invalid_value(de::Unexpected::Unsigned(v), &self)),
        }
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        v.parse::<Self::Value>()
            .map_err(|_| E::invalid_value(de::Unexpected::Str(v), &self))
    }
}

/// Function to deserializing a string to a **`number`**
pub fn de_x_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: de::Deserializer<'de>,
{
    deserializer.deserialize_any(DeBoolVisitor)
}

////////////////////////////////////////////////////////////////////////////////

struct DeBytesVisitor<T: From<Vec<u8>>>(PhantomData<T>);

impl<'de, T: From<Vec<u8>>> de::Visitor<'de> for DeBytesVisitor<T> {
    type Value = Option<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a BASE64 encoded string")
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(None)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_str(self)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        base64::decode(v.as_bytes())
            .map(|x| Some(x.into()))
            .map_err(|_| E::invalid_value(de::Unexpected::Str(v), &self))
    }
}

/// Function to serializing a `&[u8]` slice to a BASE64 encoded string.
pub fn ser_x_bytes<T, S>(this: T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: AsRef<[u8]>,
    S: ser::Serializer,
{
    serializer.serialize_str(&base64::encode(this.as_ref()))
}

/// Function to deserializing a BASE64 encoded string to a `&[u8]` slice.
pub fn de_x_bytes<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: From<Vec<u8>>,
    D: de::Deserializer<'de>,
{
    deserializer
        .deserialize_str(DeBytesVisitor::<T>(PhantomData))
        .map(|x| x.unwrap())
}

/// Function to serializing an optional `&[u8]` slice to an optional BASE64 encoded string.
pub fn ser_x_optional_bytes<T, S>(this: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
where
    T: AsRef<[u8]>,
    S: ser::Serializer,
{
    match this {
        Some(x) => serializer.serialize_str(&base64::encode(x.as_ref())),
        None => serializer.serialize_none(),
    }
}

/// Function to deserializing an optional BASE64 encoded string to a an optional `&[u8]` slice.
pub fn de_x_optional_bytes<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: From<Vec<u8>>,
    D: de::Deserializer<'de>,
{
    deserializer.deserialize_option(DeBytesVisitor::<T>(PhantomData))
}

pub mod serde_x_bytes {
    pub use super::de_x_bytes as deserialize;
    pub use super::ser_x_bytes as serialize;
}

pub mod serde_x_optional_bytes {
    pub use super::de_x_optional_bytes as deserialize;
    pub use super::ser_x_optional_bytes as serialize;
}

////////////////////////////////////////////////////////////////////////////////

struct DeStringsVisitor<T: FromStr>(PhantomData<T>);

impl<'de, T: FromStr> de::Visitor<'de> for DeStringsVisitor<T> {
    type Value = Vec<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string seperated with comma")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(v.split(|x| x == ',' || x == ';' || x == '\n')
            .filter_map(|x| x.trim().parse::<T>().ok())
            .collect())
    }
}

/// Function to serializing a **`Vec<String>`** to a simple string, the separator is ','
pub fn ser_x_strings<S>(this: &[String], serializer: S) -> Result<S::Ok, S::Error>
where
    S: ser::Serializer,
{
    serializer.serialize_str(this.join(", ").as_str())
}

/// Function to deserializing a simple string to a **`Vec<String>`**,
/// the separator is ',', ';', or '\n'
pub fn de_x_strings<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: de::Deserializer<'de>,
{
    deserializer.deserialize_str(DeStringsVisitor::<String>(PhantomData))
}

/// Function to serializing an object <T> to a simple string, the separator is ','
///
/// Usually type T is number.
pub fn ser_x_vec<T, S>(this: &[T], serializer: S) -> Result<S::Ok, S::Error>
where
    T: ToString,
    S: ser::Serializer,
{
    serializer.serialize_str(
        this.iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(", ")
            .as_str(),
    )
}

/// Function to deserializing a simple string to an object <T> a **`Vec<String>`**,
/// the separator is ',', ';', or '\n'**`Vec<String>`**
///
/// Usually type T is number.
pub fn de_x_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    T: FromStr,
    D: de::Deserializer<'de>,
{
    deserializer.deserialize_str(DeStringsVisitor::<T>(PhantomData))
}

/// Module to serialize and deserialize a **`Vec<String>`** to/from a simple string,
/// the separator is ',', ';', or '\n'**`Vec<String>`**
pub mod serde_x_strings {
    pub use super::de_x_strings as deserialize;
    pub use super::ser_x_strings as serialize;
}

/// Module to serialize and deserialize a **`Vec<T>`** to/from a simple string,
/// the separator is ',', ';', or '\n'**`Vec<String>`**
///
/// Usually type T is number.
pub mod serde_x_vec {
    pub use super::de_x_vec as deserialize;
    pub use super::ser_x_vec as serialize;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct Person {
        #[serde(with = "serde_x_bytes")]
        name: Vec<u8>,
        #[serde(with = "serde_x_optional_bytes")]
        nickname: Option<Vec<u8>>,
    }

    #[test]
    fn test_serde() {
        let mut a = Person {
            name: vec![1],
            nickname: Some(vec![2]),
        };
        let s = serde_json::to_string(&a).unwrap();
        println!("{}", &s);
        let b: Person = serde_json::from_str(&s).unwrap();
        assert_eq!(&a, &b);

        assert!(serde_json::from_str::<Person>(r#"{"name":null,"nickname":"Ag=="}"#).is_err());
        assert!(serde_json::from_str::<Person>(r#"{"name":1,"nickname":"Ag=="}"#).is_err());
        assert!(serde_json::from_str::<Person>(r#"{"name":"","nickname":"Ag=="}"#).is_ok());

        a.nickname = None;
        let s = serde_json::to_string(&a).unwrap();
        println!("{}", &s);
        let b: Person = serde_json::from_str(&s).unwrap();
        assert_eq!(&a, &b);
    }
}
