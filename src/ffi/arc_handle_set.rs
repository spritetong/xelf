use crate::prelude::{linked_hash_map::RawEntryMut, LinkedHashMap};
use crossbeam::sync::ShardedLock;
use std::{any::TypeId, mem::forget, sync::Arc};

pub type ArcHandle = usize;
type FnDrop = fn(h: ArcHandle);

pub struct ArcHandleSet {
    map: ShardedLock<LinkedHashMap<ArcHandle, (FnDrop, TypeId)>>,
}

macro_rules! arc_from_handle {
    ($h:ident, $T:ty) => {
        unsafe { Arc::from_raw($h as *const () as *const $T) }
    };
}

impl ArcHandleSet {
    pub fn new() -> Self {
        Self {
            map: ShardedLock::new(LinkedHashMap::new()),
        }
    }

    pub fn insert<T: 'static>(&self, arc: Arc<T>) -> ArcHandle {
        fn drop_arc<T: Sized>(h: ArcHandle) {
            drop(arc_from_handle!(h, T));
        }

        let handle = Arc::as_ptr(&arc) as *const () as ArcHandle;
        let t = TypeId::of::<T>();

        let mut guard = self.map.write().unwrap();
        match guard.raw_entry_mut().from_key(&handle) {
            RawEntryMut::Occupied(_) => {
                #[cfg(debug_assertions)]
                panic!(
                    "Can not insert the handle ({}) of an Arc<{}>, it's already in the set.",
                    handle,
                    std::any::type_name::<T>()
                );
            }
            RawEntryMut::Vacant(vacant) => {
                forget(arc);
                vacant.insert(handle, (drop_arc::<T>, t));
            }
        }

        handle
    }

    pub fn remove<T: 'static>(&self, handle: ArcHandle) -> Option<Arc<T>> {
        if handle != 0 {
            let mut guard = self.map.write().unwrap();
            if let Some((_, t)) = guard.remove(&handle) {
                if t == TypeId::of::<T>() {
                    return Some(arc_from_handle!(handle, T));
                } else {
                    #[cfg(debug_assertions)]
                    panic!(
                        "Attemp to remove an Arc::<{}>, but the handle {} does not match.",
                        std::any::type_name::<T>(),
                        handle,
                    );
                }
            } else {
                #[cfg(debug_assertions)]
                panic!(
                    "Attemp to remove an Arc::<{}>, but the handle {} is not found.",
                    std::any::type_name::<T>(),
                    handle,
                );
            }
        }
        None
    }

    pub fn get<T: 'static>(&self, handle: ArcHandle) -> Option<Arc<T>> {
        if handle != 0 {
            let guard = self.map.read().unwrap();
            if let Some((_, t)) = guard.get(&handle) {
                if *t == TypeId::of::<T>() {
                    let a = arc_from_handle!(handle, T);
                    // Increase the reference count: clone() + forget().
                    forget(a.clone());
                    return Some(a);
                } else {
                    #[cfg(debug_assertions)]
                    panic!(
                        "Attemp to get a Arc::<{}>, but the handle does not match.",
                        std::any::type_name::<T>(),
                    );
                }
            }
        }
        None
    }

    pub fn clear(&self) {
        self.map.write().unwrap().retain(|&h, (drop, _)| {
            drop(h);
            false
        });
    }

    pub fn clear_of<T: 'static>(&self) {
        let t = TypeId::of::<T>();
        self.map.write().unwrap().retain(move |&h, (drop, x)| {
            if *x == t {
                drop(h);
                false
            } else {
                true
            }
        });
    }
}

impl Drop for ArcHandleSet {
    fn drop(&mut self) {
        self.clear();
    }
}

mod tests {
    pub use super::*;

    #[test]
    fn test_arc_handle_set() {
        let set = ArcHandleSet::new();

        let a1 = Arc::new(1);
        let h1 = set.insert(a1.clone());
        assert_eq!(set.get(h1), Some(a1.clone()));
        assert_eq!(set.get(h1), Some(a1.clone()));

        let a2 = Arc::new(100i64);
        let h2 = set.insert(a2.clone());
        assert_eq!(set.get(h2), Some(a2.clone()));
        assert_eq!(set.get(h2), Some(a2.clone()));

        //assert!(set.remove::<i32>(h).is_some());
        assert_eq!(set.map.read().unwrap().len(), 2);
        set.clear_of::<i32>();
        assert_eq!(set.map.read().unwrap().len(), 1);

        set.clear();
        assert_eq!(set.map.read().unwrap().len(), 0);
    }
}
