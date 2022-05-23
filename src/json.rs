use crate::collections::Contains;
#[cfg(feature = "num")]
use crate::num::*;
use ::serde::{de::DeserializeOwned, ser::Serialize};
use ::serde_json::{json, map::Map, value::Index, Value as Json};
use ::std::{borrow::Borrow, hash::Hash};

////////////////////////////////////////////////////////////////////////////////

/// Trait for JSON object, arrary, associated with an index.
pub trait JsonIndexed<I> {
    fn get_member(&self, index: I) -> Option<&Json>;
}

impl<I: Index> JsonIndexed<I> for Json {
    #[inline(always)]
    fn get_member(&self, index: I) -> Option<&Json> {
        self.get(index)
    }
}

impl<I: AsRef<str>> JsonIndexed<I> for Map<String, Json> {
    #[inline(always)]
    fn get_member(&self, index: I) -> Option<&Json> {
        self.get(index.as_ref())
    }
}

impl<I: Num + Index> JsonIndexed<I> for Vec<Json> {
    #[inline(always)]
    fn get_member(&self, index: I) -> Option<&Json> {
        self.get(index.as_usize())
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Trait to get a field value with a default value.
pub trait JsonGetOr<'a, I, T, _T> {
    /// Get a field value, returns the default value if failed.
    ///
    /// # Arguments
    ///
    /// * `index` - A string (slice) for a child value, or an integer value of an array item.
    ///
    /// * `default` - The default value returned on error.
    ///
    /// # Returns
    ///
    /// If there is an item which matches the index and the default value's type, return its value.
    ///
    /// Otherwize, returns the default value.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_json::json;
    /// use rsx::json::*;
    ///
    /// let jsn = json!({"name": "Tom", "value": 100});
    ///
    /// assert_eq!(jsn.get_or("name", "John"), "Tom");
    /// // type dese not match
    /// assert_eq!(jsn.get_or("name", 1), 1);
    /// // index dese not match
    /// assert_eq!(jsn.get_or("Name", "Json"), "Json");
    ///
    /// assert_eq!(jsn.get_or("value", 1), 100);
    /// // type dese not match
    /// assert_eq!(jsn.get_or("value", "1"), "1");
    /// ```
    fn get_or(&'a self, index: I, default: T) -> T;

    /// Get a field value, returns the default value if failed.
    ///
    /// # Arguments
    ///
    /// * `index` - A string (slice) for a child value, or an integer value of an array item.
    ///
    /// * `f` - A function to return the default value.
    ///
    /// # Returns
    ///
    /// If there is an item which matches the index and the default value's type, return its value.
    ///
    /// Otherwize, call the function and returns it's result.
    fn get_or_else<F: FnOnce() -> T>(&'a self, index: I, f: F) -> T;
}

#[cfg(feature = "num")]
impl<I: Index, T: Num, V: JsonIndexed<I>> JsonGetOr<'_, I, T, i64> for V {
    #[inline]
    fn get_or(&self, index: I, default: T) -> T {
        T::from_i64(
            self.get_member(index)
                .and_then(|x| x.as_i64())
                .unwrap_or(default.as_i64()),
        )
    }

    #[inline]
    fn get_or_else<F: FnOnce() -> T>(&self, index: I, f: F) -> T {
        self.get_member(index)
            .and_then(|x| x.as_i64())
            .map(|x| T::from_i64(x))
            .unwrap_or_else(f)
    }
}

#[cfg(feature = "num")]
impl<'a, I: Index, T: Float, V: JsonIndexed<I>> JsonGetOr<'a, I, T, f64> for V {
    #[inline]
    fn get_or(&self, index: I, default: T) -> T {
        T::from_f64(
            self.get_member(index)
                .and_then(|x| x.as_f64())
                .unwrap_or(default.as_f64()),
        )
    }

    #[inline]
    fn get_or_else<F: FnOnce() -> T>(&self, index: I, f: F) -> T {
        self.get_member(index)
            .and_then(|x| x.as_f64())
            .map(|x| T::from_f64(x))
            .unwrap_or_else(f)
    }
}

impl<'a, I: Index, V: JsonIndexed<I>> JsonGetOr<'a, I, &'a str, char> for V {
    #[inline]
    fn get_or(&'a self, index: I, default: &'a str) -> &'a str {
        self.get_member(index)
            .and_then(|x| x.as_str())
            .unwrap_or(default)
    }

    #[inline]
    fn get_or_else<F: FnOnce() -> &'a str>(&'a self, index: I, f: F) -> &'a str {
        self.get_member(index)
            .and_then(|x| x.as_str())
            .unwrap_or_else(f)
    }
}

impl<I: Index, V: JsonIndexed<I>> JsonGetOr<'_, I, bool, bool> for V {
    #[inline]
    fn get_or(&self, index: I, default: bool) -> bool {
        self.get_member(index)
            .and_then(|x| x.as_bool())
            .unwrap_or(default)
    }

    #[inline]
    fn get_or_else<F: FnOnce() -> bool>(&self, index: I, f: F) -> bool {
        self.get_member(index)
            .and_then(|x| x.as_bool())
            .unwrap_or_else(f)
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Extension for serde_json::Value.
pub trait JsonObjectRsx {
    /// Simplify key-value insertion for a JSON object.
    ///
    /// Insert key-value into a json object with an index and a value.
    ///
    /// # Arguments
    ///
    /// * `k` `v`: can be primitive data type, such as string references or integers.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_json::json;
    /// use rsx::json::*;
    ///
    /// let mut jsn = json!({});
    ///
    /// jsn.insert_s("name", "tom");
    /// jsn.insert_s("age", 16);
    ///
    /// assert_eq!(jsn.get_or("name", ""), "tom");
    /// assert_eq!(jsn.get_or("age", 16), 16);
    /// ```
    fn insert_s<T: Serialize>(&mut self, k: &str, v: T) -> Option<Json>;

    /// Take all fields with the prefix and insert into a new object.
    ///
    /// The prefix of fields name will be removed before insertion.
    ///
    fn take_with_prefix(&mut self, prefix: &str) -> Self;

    /// Merge a JSON object to a serializable object.
    fn merge_to<T, S, K>(&self, dst: &mut T, skip: &S) -> serde_json::Result<()>
    where
        T: Serialize + DeserializeOwned,
        S: ?Sized + Contains<K, str>,
        K: Hash + Ord + Eq + Borrow<str>;
}

impl JsonObjectRsx for Json {
    #[inline]
    fn insert_s<T: Serialize>(&mut self, k: &str, v: T) -> Option<Json> {
        self.as_object_mut().unwrap().insert(k.to_owned(), json!(v))
    }

    fn take_with_prefix(&mut self, prefix: &str) -> Self {
        Json::from(if let Some(src) = self.as_object_mut() {
            src.take_with_prefix(prefix)
        } else {
            Map::<String, Json>::new()
        })
    }

    fn merge_to<T, S, K>(&self, dst: &mut T, skip: &S) -> serde_json::Result<()>
    where
        T: Serialize + DeserializeOwned,
        S: ?Sized + Contains<K, str>,
        K: Hash + Ord + Eq + Borrow<str>,
    {
        if let Some(map) = self.as_object() {
            map.merge_to(dst, skip)
        } else {
            Ok(())
        }
    }
}

impl JsonObjectRsx for Map<String, Json> {
    #[inline]
    fn insert_s<T: Serialize>(&mut self, k: &str, v: T) -> Option<Json> {
        self.insert(k.to_owned(), json!(v))
    }

    fn take_with_prefix(&mut self, prefix: &str) -> Self {
        let mut map = Map::new();
        for (k, v) in self {
            if k.starts_with(prefix) {
                map.insert((&k[prefix.len()..]).to_owned(), Json::from(v.take()));
            }
        }
        map
    }

    fn merge_to<T, S, K>(&self, dst: &mut T, skip: &S) -> serde_json::Result<()>
    where
        T: Serialize + DeserializeOwned,
        S: ?Sized + Contains<K, str>,
        K: Hash + Ord + Eq + Borrow<str>,
    {
        let mut value = serde_json::to_value(&dst)?;
        if let Some(map) = value.as_object_mut() {
            for (k, v) in map {
                if !skip.contains_ref(k.as_str()) {
                    if let Some(o) = self.get(k) {
                        *v = o.clone();
                    }
                }
            }
        }
        T::deserialize_in_place(value, dst)?;
        Ok(())
    }
}

// ////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_get_or() {
        let jsn = json!({"name": "Tom", "value": 100, "weight": 50.});

        assert_eq!(jsn.get_or("name", "John"), "Tom");
        // type dese not match
        assert_eq!(jsn.get_or("name", 1), 1);
        // index dese not match
        assert_eq!(jsn.get_or("Name", "Json"), "Json");

        assert_eq!(jsn.get_or("value", 1), 100);
        // type dese not match
        assert_eq!(jsn.get_or("value", "1"), "1");

        assert_eq!(jsn.get_or("weight", 1.), 50.);
        // type dese not match
        assert_eq!(jsn.get_or("weight", "1"), "1");

        let jsn = jsn.as_object().unwrap();

        assert_eq!(jsn.get_or("name", "John"), "Tom");
        // type dese not match
        assert_eq!(jsn.get_or("name", 1), 1);
        // index dese not match
        assert_eq!(jsn.get_or("Name", "Json"), "Json");

        assert_eq!(jsn.get_or("value", 1), 100);
        assert_eq!(jsn.get_or_else("value", || 1), 100);
        // type dese not match
        assert_eq!(jsn.get_or("value", "1"), "1");

        assert_eq!(jsn.get_or("weight", 1.), 50.);
        // type dese not match
        assert_ne!(jsn.get_or("weight", 1), 50);
        // type dese not match
        assert_eq!(jsn.get_or("weight", "1"), "1");
        assert_eq!(jsn.get_or_else("weight", || "1"), "1");

        let jsn = json!([1, 2., "3"]);
        let jsn = jsn.as_array().unwrap();
        assert_eq!(jsn.get_or(0, 2), 1);
        // type dese not match
        assert_eq!(jsn.get_or(1, 3), 3);
        assert_eq!(jsn.get_or(1, 2.), 2.);
        assert_eq!(jsn.get_or(3, "3"), "3");
    }

    #[test]
    fn test_json_insert() {
        let mut jsn = json!({});

        jsn.insert_s("name", "tom");
        jsn.insert_s("age", 16);

        assert_eq!(jsn.get_or("name", ""), "tom");
        assert_eq!(jsn.get_or("age", 16), 16);
    }

    #[test]
    fn test_json_merge() {
        let jsn1 = json!({"1": 11, "2": 22, "3": 33});
        let mut jsn2 = json!({"1": 1, "2": 2});
        assert!(jsn1.merge_to(&mut jsn2, &["2"]).is_ok());

        assert_eq!(jsn2.get_or("1", 0i32), 11);
        assert_eq!(jsn2.get_or("2", 0i32), 2);
        assert_eq!(jsn2.get_or("3", 0i32), 0);
    }
}
