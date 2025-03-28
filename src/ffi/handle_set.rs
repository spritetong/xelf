use crate::prelude::{linked_hash_map::RawEntryMut, LinkedHashMap};
use parking_lot::RwLock;
use std::any::{Any, TypeId};

pub type Handle = usize;

pub struct HandleSet {
    map: RwLock<LinkedHashMap<Handle, Entry>>,
}

struct Entry {
    type_id: TypeId,
    drop: fn(Handle),
}

impl Default for HandleSet {
    fn default() -> Self {
        Self {
            map: RwLock::new(LinkedHashMap::new()),
        }
    }
}

macro_rules! clone_from_handle {
    ($handle:ident, $T:ty) => {
        unsafe { &*($handle as *const $T) }.clone()
    };
}

macro_rules! drop_handle {
    ($handle:ident, $T:ty) => {
        unsafe { drop(Box::from_raw(($handle as *const $T).cast_mut())) };
    };
}

impl HandleSet {
    pub fn insert<T: Any + Clone + Send + 'static>(&self, instance: T) -> Handle {
        fn drop_handle<T>(handle: Handle) {
            drop_handle!(handle, T);
        }

        let handle = Box::leak(Box::new(instance)) as *const _ as Handle;
        match self.map.write().raw_entry_mut().from_key(&handle) {
            RawEntryMut::Occupied(_) => {
                drop_handle!(handle, T);
                unreachable!();
            }
            RawEntryMut::Vacant(vacant) => {
                vacant.insert(
                    handle,
                    Entry {
                        type_id: TypeId::of::<T>(),
                        drop: drop_handle::<T>,
                    },
                );
            }
        }
        handle
    }

    pub fn remove<T: Any + Clone + Send + 'static>(&self, handle: Handle) -> Option<T> {
        if handle == 0 {
            return None;
        }

        let mut guard = self.map.write();
        if let Some(entry) = guard.get(&handle) {
            if entry.type_id == TypeId::of::<T>() {
                let instance = clone_from_handle!(handle, T);
                drop_handle!(handle, T);
                guard.remove(&handle);
                return Some(instance);
            } else {
                panic!(
                    "Attemp to remove an `{}`, but the handle {} does not match.",
                    std::any::type_name::<T>(),
                    handle,
                );
            }
        } else {
            panic!(
                "Attemp to remove an `{}`, but the handle {} is not found.",
                std::any::type_name::<T>(),
                handle,
            );
        }
    }

    pub fn get<T: Any + Clone + Send + 'static>(&self, handle: Handle) -> Option<T> {
        if handle == 0 {
            return None;
        }

        self.map.read().get(&handle).map(|entry| {
            if entry.type_id == TypeId::of::<T>() {
                clone_from_handle!(handle, T)
            } else {
                panic!(
                    "Attemp to get a Arc::<{}>, but the handle does not match.",
                    std::any::type_name::<T>(),
                );
            }
        })
    }

    pub fn clear(&self) {
        self.map.write().retain(|handle, entry| {
            (entry.drop)(*handle);
            false
        });
    }

    pub fn clear_of<T: 'static>(&self) {
        self.map.write().retain(|handle, entry| {
            if entry.type_id == TypeId::of::<T>() {
                (entry.drop)(*handle);
                false
            } else {
                true
            }
        });
    }
}

impl Drop for HandleSet {
    fn drop(&mut self) {
        self.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_arc_handle_set() {
        struct Counter(Arc<RwLock<usize>>);
        impl Counter {
            fn new(counter: Arc<RwLock<usize>>) -> Self {
                *counter.write() += 1;
                Self(counter)
            }
        }
        impl Clone for Counter {
            fn clone(&self) -> Self {
                Self::new(self.0.clone())
            }
        }
        impl Drop for Counter {
            fn drop(&mut self) {
                *self.0.write() -= 1;
            }
        }

        let set = HandleSet::default();

        let a1 = Arc::new(1);
        let h1 = set.insert(a1.clone());
        assert_eq!(set.get(h1), Some(a1.clone()));
        assert_eq!(set.get(h1), Some(a1.clone()));

        let a2 = Arc::new(100i64);
        let h2 = set.insert(a2.clone());
        assert_eq!(set.get(h2), Some(a2.clone()));
        assert_eq!(set.get(h2), Some(a2.clone()));

        //assert!(set.remove::<i32>(h).is_some());
        assert_eq!(set.map.read().len(), 2);
        set.clear_of::<Arc<i32>>();
        assert_eq!(set.map.read().len(), 1);

        set.clear();
        assert_eq!(set.map.read().len(), 0);

        let counter = Arc::new(RwLock::new(0usize));
        assert_eq!(*counter.read(), 0);
        let h3 = set.insert(Counter::new(counter.clone()));
        assert_eq!(*counter.read(), 1);
        let a3: Option<Counter> = set.get::<Counter>(h3);
        assert_eq!(*counter.read(), 2);
        drop(a3);
        assert_eq!(*counter.read(), 1);
        assert!(set.remove::<Counter>(h3).is_some());
        assert_eq!(*counter.read(), 0);
    }
}
