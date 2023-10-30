#[cfg(feature = "num")]
use num_traits::{Float, FromPrimitive, PrimInt};
use serde::{de, ser};
use std::{fmt, marker::PhantomData, str::FromStr};

#[cfg(feature = "num")]
#[derive(Default)]
struct DeNumVisitor<T: Default + PrimInt> {
    _phantom: std::marker::PhantomData<T>,
}

#[cfg(feature = "num")]
impl<'de, T: Default + PrimInt + FromPrimitive + FromStr> de::Visitor<'de> for DeNumVisitor<T> {
    type Value = T;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer or a string")
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Self::Value::from_i64(v).ok_or_else(|| E::invalid_value(de::Unexpected::Signed(v), &self))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Self::Value::from_u64(v).ok_or_else(|| E::invalid_value(de::Unexpected::Unsigned(v), &self))
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
    T: Default + PrimInt + FromPrimitive + FromStr,
{
    deserializer.deserialize_any(DeNumVisitor::<T>::default())
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "num")]
#[derive(Default)]
struct DeFloatVisitor<T: Default + Float> {
    _phantom: std::marker::PhantomData<T>,
}

#[cfg(feature = "num")]
impl<'de, T: Default + Float + FromPrimitive + FromStr> de::Visitor<'de> for DeFloatVisitor<T> {
    type Value = T;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer or a string")
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Self::Value::from_f64(v).ok_or_else(|| E::invalid_value(de::Unexpected::Float(v), &self))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Self::Value::from_i64(v).ok_or_else(|| E::invalid_value(de::Unexpected::Signed(v), &self))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Self::Value::from_u64(v).ok_or_else(|| E::invalid_value(de::Unexpected::Unsigned(v), &self))
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
    T: Default + Float + FromPrimitive + FromStr,
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

/// Module to serialize and deserialize a **`Vec<String>`** to/from a simple string,
/// the separator is ',', ';', or '\n'**`Vec<String>`**
pub mod serde_x_strings {
    pub use super::de_x_strings as deserialize;
    pub use super::ser_x_strings as serialize;
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[serde_as]
    #[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
    struct Person {
        #[serde_as(as = "Base64")]
        name: Vec<u8>,
        #[serde_as(as = "Option<Base64>")]
        nickname: Option<Vec<u8>>,
        #[serde_as(as = "StringWithSeparator::<SemicolonSeparator, String>")]
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        strings: Vec<String>,
    }

    serde_with::with_prefix!(prefix_player1 "player1_");

    #[serde_as]
    #[derive(Deserialize, Serialize)]
    struct Data(#[serde_as(as = "serde_with::BoolFromInt")] bool);
    
    #[test]
    fn test_serde() {
        let mut a: Person = Person {
            name: vec![1],
            nickname: Some(vec![2]),
            ..Default::default()
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
