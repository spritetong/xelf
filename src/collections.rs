#[cfg(feature = "serde-json")]
use ::serde_json::{Map as JsonMap, Value as Json};
use ::std::{
    borrow::Borrow,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque},
    hash::{BuildHasher, Hash},
};

/// Trait for a container which contains a key.
pub trait Contains<K, Q>
where
    K: ?Sized + Eq + Ord + Hash + Borrow<Q>,
    Q: ?Sized + Eq + Hash + Ord,
{
    /// Returns `true` if the container contains a value for the specified key.
    fn contains_it(&self, k: &Q) -> bool;
}

macro_rules! impl_contains {
    (@option $self:ident, $k:ident) => {
        match $self.as_ref() {
            Some(v) if v.borrow() == $k => true,
            _ => false,
        }
    };

    (@iter $self:ident, $k:ident) => {
        'outer: loop {
            for it in $self.iter() {
                if it.borrow() == $k {
                    break 'outer true;
                }
            }
            break false;
        }
    };

    (@set $self:ident, $k:ident) => {
        $self.contains($k)
    };

    (@map $self:ident, $k:ident) => {
        $self.contains_key($k)
    };

    (@iter [$($N:ident)*], $T:ty) => {
        impl_contains!{@iter K, [$($N)*], impl<K, Q> for $T where}
    };

    (@as_item $i:item) => { $i };

    (@$type:ident $K:ident, [$($N:ident),*],
     impl [$($args:ident),*] for $T:ty where [$($preds:tt)*]) => {
        impl_contains! {
            @as_item
            impl<$($args),* $(,const $N: usize)*> Contains<$K, Q> for $T
                where
                    $K: Hash + Ord + Eq + Borrow<Q>,
                    Q: ?Sized + Eq + Hash + Ord,
                    $($preds)*
            {
                fn contains_it(&self, k: &Q) -> bool {
                    impl_contains!(@$type self, k)
                }
            }
        }
    };

    (@$type:ident $K:ident, [$($N:ident),* $(,)*],
     impl<$($args:ident),* $(,)*> for $T:ty where $($preds:tt)*) => {
        impl_contains! { @$type $K, [$($N),*],
            impl [$($args),*] for $T where [$($preds)*] }
    };
}

impl_contains!(@option K, [], impl<K, Q> for Option<K> where);

impl_contains!(@iter [N], [K; N]);
impl_contains!(@iter [], [K]);
impl_contains!(@iter [], Vec<K>);
impl_contains!(@iter [], VecDeque<K>);

impl_contains!(@map K, [], impl<K, Q, V> for BTreeMap<K, V> where);
impl_contains!(@set K, [], impl<K, Q> for BTreeSet<K> where);

impl_contains!(@map K, [], impl<K, Q, V, S> for HashMap<K, V, S> where S: BuildHasher);
impl_contains!(@set K, [], impl<K, Q, S> for HashSet<K, S> where S: BuildHasher);

#[cfg(feature = "ritelinked")]
impl_contains!(@map K, [], impl<K, Q, V, S> for ritelinked::LinkedHashMap<K, V, S> where S: BuildHasher);
#[cfg(feature = "ritelinked")]
impl_contains!(@set K, [], impl<K, Q, S> for ritelinked::LinkedHashSet<K, S> where S: BuildHasher);

#[cfg(feature = "serde-json")]
impl_contains!(@map String, [], impl<Q> for JsonMap<String, Json> where);

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "serde-json")]
    use serde_json::json;

    #[test]
    fn test_container() {
        assert!(Some("1").contains_it("1"));
        assert!(!Some("1").contains_it("2"));
        assert!(!None::<&str>.contains_it("2"));

        assert!(Some(1).contains_it(&1));
        assert!(!Some(1).contains_it(&2));
        assert!(!None::<i32>.contains_it(&2));

        assert!(["1", "2", "3"].contains_it("1"));
        assert!(["1".to_owned(), "2".to_owned(), "3".to_owned()].contains_it("1"));
        assert!(["1".to_owned(), "2".to_owned(), "3".to_owned()].contains_it(&"1".to_owned()));
        assert!(vec!["1".to_owned(), "2".to_owned(), "3".to_owned()].contains_it("1"));

        assert!([1, 2, 3].contains_it(&1));
        assert!(vec![1, 2, 3].contains_it(&1));

        #[cfg(feature = "serde-json")]
        {
            let a = json!({"1": 1, "2": 2, "3": 3});
            assert!(a.as_object().unwrap().contains_it("1"));
        }
    }
}
