use crossbeam::{
    queue::SegQueue,
    sync::{Parker, Unparker},
};
use futures::Future;
use std::mem::ManuallyDrop;

pub struct ParkerCache {
    q: SegQueue<usize>,
}

impl ParkerCache {
    pub fn new() -> Self {
        Self { q: SegQueue::new() }
    }

    pub fn get(&self) -> (UnparkerGuard, ParkerGuard) {
        let p = self.pop().unwrap_or_else(|| Parker::new());
        let u = p.unparker().clone();
        (
            UnparkerGuard(ManuallyDrop::new(u)),
            ParkerGuard {
                parker: ManuallyDrop::new(p),
                cache: self,
            },
        )
    }

    pub fn clear(&self) {
        // Clear the packer queue.
        while let Some(_) = self.pop() {}
    }

    #[cfg(feature = "async")]
    pub fn block_on<'a, F>(&self, handle: tokio::runtime::Handle, future: F)
    where
        F: Future<Output = ()> + 'a,
    {
        tokio_scope::Scope::new(&self, handle).block_on(future);
    }

    #[inline]
    fn pop(&self) -> Option<Parker> {
        self.q
            .pop()
            .map(|x| unsafe { Parker::from_raw(x as *const ()) })
    }

    #[inline]
    fn push(&self, parker: Parker) {
        // use std::mem::transmute;
        // use std::sync::Arc;
        // println!(
        //     "{}, {}",
        //     Arc::strong_count(unsafe { transmute::<_, &Arc<i64>>(&parker) }),
        //     self.q.len()
        // );
        self.q.push(Parker::into_raw(parker) as usize);
    }
}

impl Drop for ParkerCache {
    fn drop(&mut self) {
        self.clear()
    }
}

pub struct ParkerGuard<'a> {
    cache: &'a ParkerCache,
    parker: ManuallyDrop<Parker>,
}

impl<'a> ParkerGuard<'a> {
    #[inline]
    pub fn park(self) {
        drop(self);
    }
}

impl<'a> Drop for ParkerGuard<'a> {
    fn drop(&mut self) {
        let p = unsafe { ManuallyDrop::take(&mut self.parker) };
        p.park();
        self.cache.push(p);
    }
}

#[repr(transparent)]
pub struct UnparkerGuard(ManuallyDrop<Unparker>);

impl UnparkerGuard {
    #[inline]
    pub fn unpark(self) {
        drop(self);
    }
}

impl Drop for UnparkerGuard {
    fn drop(&mut self) {
        unsafe { ManuallyDrop::take(&mut self.0) }.unpark();
    }
}

#[cfg(feature = "async")]
mod tokio_scope {
    use super::*;
    use futures::Future;
    use std::{
        pin::Pin,
        task::{Context, Poll},
    };
    use tokio::runtime::Handle;

    struct ScopedFuture {
        f: Pin<Box<dyn Future<Output = ()> + Send + 'static>>,
        _unparker: UnparkerGuard,
    }

    impl Future for ScopedFuture {
        type Output = ();

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let inner_future = &mut self.f;
            let future_ref = Pin::as_mut(inner_future);
            future_ref.poll(cx)
            // When then future is dropped, 'unpacker' is auto executed.
        }
    }

    pub(super) struct Scope<'c> {
        cache: &'c ParkerCache,
        handle: Handle,
    }

    impl<'c> Scope<'c> {
        pub(super) fn new(cache: &'c ParkerCache, handle: Handle) -> Scope<'c> {
            Scope { cache, handle }
        }

        pub(super) fn block_on<'a, F>(&self, future: F)
        where
            F: Future<Output = ()> + 'a,
        {
            let (unparker, parker) = self.cache.get();

            let boxed: Pin<Box<dyn Future<Output = ()> + 'a>> = Box::pin(future);
            // This transmute should be safe, as we use the `ScopedFuture` abstraction to prevent the
            // scope from exiting until every spawned `ScopedFuture` object is dropped, signifying that
            // they have completed their execution.
            let boxed: Pin<Box<dyn Future<Output = ()> + Send + 'static>> =
                unsafe { std::mem::transmute(boxed) };

            let future = ScopedFuture {
                f: boxed,
                _unparker: unparker,
            };

            self.handle.spawn(future);
            parker.park();
        }
    }
}

mod tests {
    pub use super::*;

    #[test]
    fn test_parker_cache() {
        let cache = ParkerCache::new();

        {
            let (_, _p) = cache.get();
            cache.get();
        }

        let mut list = vec![];
        for _ in 0..=100 {
            let (_, p) = cache.get();
            list.push(p);
        }
        list.clear();

        cache.clear();
    }

    #[cfg(feature = "async")]
    #[test]
    fn test_block_on() {
        //use std::time::Duration;
        use tokio::runtime::Runtime;

        let cache = ParkerCache::new();
        let rt = Runtime::new().expect("Failed to construct Runtime");
        for _ in 0..10000 {
            let mut uncopy = String::from("Borrowed");
            let mut uncopy2 = String::from("Borrowed");
            cache.block_on(rt.handle().clone(), async {
                let f = 4;
                assert_eq!(f, 4);
                //tokio::time::sleep(Duration::from_millis(500)).await;
                uncopy.push('!');

                uncopy2.push('f');
            });

            assert_eq!(uncopy.as_str(), "Borrowed!");
            assert_eq!(uncopy2.as_str(), "Borrowedf");
        }
    }
}
