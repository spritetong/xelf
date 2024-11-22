use ::std::{future::Future, io, pin::Pin, time::Duration};
use ::tokio_util::sync::CancellationToken;

pub use ::tokio::sync::mpsc as svc_channel;
pub type SvcSender<T> = svc_channel::Sender<T>;
pub type SvcReceiver<T> = svc_channel::Receiver<T>;
pub type SvcSendError<T> = svc_channel::error::SendError<T>;
pub type SvcTrySendError<T> = svc_channel::error::TrySendError<T>;
pub type SvcTryRecvError = svc_channel::error::TryRecvError;
pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

/// Trait for a generic service.
pub trait EasyService: Sync + Send {
    fn start(&self) -> io::Result<()>;
    fn is_terminated(&self) -> bool;
    fn terminate(&self);
    fn blocking_join(&self);
    fn join(&self) -> BoxFuture<()>;
}

/// Trait for a collection of some services.
pub trait EasyServices {
    fn as_vec(&self) -> Vec<&dyn EasyService>;
}

/// Create a tokio runtime on the current thread to run a service.
pub fn easy_service_create_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .max_blocking_threads(1)
        .build()
        .unwrap()
}

/// A safe sender which has a timeout and a cancellation token to prevent deadlock.
pub struct SvcSafeSender<T> {
    pub sender: SvcSender<T>,
    pub timeout: Duration,
    pub token: CancellationToken,
}

impl<T> Clone for SvcSafeSender<T> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            timeout: self.timeout,
            token: self.token.clone(),
        }
    }
}

impl<T> SvcSafeSender<T> {
    /// Create a new sender with a default timeout and a cancellation token.
    pub fn new(sender: SvcSender<T>, timeout: Duration, token: CancellationToken) -> Self {
        Self {
            sender,
            timeout,
            token,
        }
    }

    /// Try to send a value without blocking.
    #[inline]
    pub fn try_send(&self, value: T) -> Result<(), SvcTrySendError<T>> {
        self.sender.try_send(value)
    }

    /// Send a value with the default timeout.
    ///
    /// If the timeout is reached or the token is cancelled, return an error.
    pub async fn send(&self, value: T) -> Result<(), SvcSendError<T>> {
        match self.sender.try_reserve() {
            Ok(permit) => {
                permit.send(value);
                return Ok(());
            }
            Err(SvcTrySendError::Full(_)) => {
                tokio::select! {
                    biased;
                    x = self.sender.reserve() => {
                        if let Ok(permit) = x {
                            permit.send(value);
                            return Ok(())
                        }
                    }
                    _ = self.token.cancelled() => (),
                }
            }
            Err(_) => (),
        }
        Err(svc_channel::error::SendError(value))
    }

    /// Send a value with a timeout.
    ///
    /// If the timeout is reached or the token is cancelled, return an error.
    pub async fn send_timeout(
        &self,
        value: T,
        timeout: Option<Duration>,
    ) -> Result<(), SvcSendError<T>> {
        match self.sender.try_reserve() {
            Ok(permit) => {
                permit.send(value);
                return Ok(());
            }
            Err(SvcTrySendError::Full(_)) => {
                tokio::select! {
                    biased;
                    x = self.sender.reserve() => {
                        if let Ok(permit) = x {
                            permit.send(value);
                            return Ok(())
                        }
                    }
                    _ = self.token.cancelled() => (),
                    _ = tokio::time::sleep(timeout.unwrap_or(self.timeout)) => (),
                }
            }
            Err(_) => (),
        }
        Err(svc_channel::error::SendError(value))
    }
}

#[macro_export]
macro_rules! easy_service {
    (ASYNC
        $service_vis:vis $Service:ident,
        $Task:ident,
        $inner_vis:vis $Inner:ident { $($fields:tt)* }
    ) => {
        easy_service!(@impl $service_vis $Service, $Task, tokio::task::JoinHandle<()>,
            $inner_vis $Inner { $($fields)* });
        impl $Inner {
            easy_service!(@InnerAsync);
        }
    };

    (ASYNC
        $service_vis:vis $Service:ident,
        $Task:ident,
        $inner_vis:vis $Inner:ident <$($S:ident),*> { $($fields:tt)* }
        where $($preds:tt)*
    ) => {
        easy_service!(@impl $service_vis $Service, $Task, tokio::task::JoinHandle<()>,
            $inner_vis $Inner <$($S),*> { $($fields)* } where $($preds)*);
        easy_service! {
            @as_item
            impl<$($S),*> $Inner<$($S),*> where $($preds)* {
                easy_service!(@InnerAsync);
            }
        }
    };

    (SYNC
        $service_vis:vis $Service:ident,
        $Task:ident,
        $inner_vis:vis $Inner:ident { $($fields:tt)* }
    ) => {
        easy_service!(@impl $service_vis $Service, $Task, Box<std::thread::JoinHandle<()>>,
            $inner_vis $Inner { $($fields)* });
        impl $Inner {
            easy_service!(@InnerSync);
        }
    };

    (SYNC
        $service_vis:vis $Service:ident,
        $Task:ident,
        $inner_vis:vis $Inner:ident <$($S:ident),*> { $($fields:tt)* }
        where $($preds:tt)*
    ) => {
        easy_service!(@impl $service_vis $Service, $Task, Box<std::thread::JoinHandle<()>>,
            $inner_vis $Inner <$($S),*> { $($fields)* } where $($preds)*);
        easy_service! {
            @as_item
            impl<$($S),*> $Inner<$($S),*> where $($preds)* {
                easy_service!(@InnerSync);
            }
        }
    };

    (TASK $boxed_task:ident) => {
        AtomicCell::new(Some($boxed_task))
    };

    (TASK $task:expr) => {
        AtomicCell::new(Some(Box::new($task)))
    };

    ($Inner:ident { $($fields:tt)* }) => {
        std::sync::Arc::new($Inner {
            task_handle: AtomicCell::new(None),
            $($fields)*
        })
    };

    ////////////////////////////////////////////////////////////////////////////

    (@impl
        $service_vis:vis $Service:ident,
        $Task:ident,
        $Handle:ty,
        $inner_vis:vis $Inner:ident { $($fields:tt)* }
    ) => {
        #[repr(transparent)]
        $service_vis struct $Service($service_vis std::sync::Arc<$Inner>);
        $inner_vis struct $Inner {
            token: tokio_util::sync::CancellationToken,
            task: crossbeam::atomic::AtomicCell<Option<Box<$Task>>>,
            task_handle: crossbeam::atomic::AtomicCell<Option<$Handle>>,
            $($fields)*
        }
        impl $Inner {
            easy_service!(@Inner);
        }
        impl Drop for $Inner {
            easy_service!(@InnerDrop);
        }
        impl EasyService for $Service {
            easy_service!(@Service);
        }
        impl Clone for $Service {
            easy_service!(@ServiceClone);
        }
    };

    (@impl
        $service_vis:vis $Service:ident,
        $Task:ident,
        $Handle:ty,
        $inner_vis:vis $Inner:ident <$($S:ident),*> { $($fields:tt)* }
        where $($preds:tt)*
    ) => {
        #[repr(transparent)]
        $service_vis struct $Service<$($S),*>($service_vis std::sync::Arc<$Inner<$($S),*>>)
            where $($preds)*;
        $inner_vis struct $Inner<$($S),*> where $($preds)* {
            token: tokio_util::sync::CancellationToken,
            task: crossbeam::atomic::AtomicCell<Option<Box<$Task<$($S),*>>>>,
            task_handle: crossbeam::atomic::AtomicCell<Option<$Handle>>,
            $($fields)*
        }
        easy_service! {
            @as_item
            impl<$($S),*> $Inner<$($S),*> where $($preds)* {
                easy_service!(@Inner);
            }
            impl<$($S),*> Drop for $Inner<$($S),*> where $($preds)* {
                easy_service!(@InnerDrop);
            }
            impl<$($S),*> EasyService for $Service<$($S),*> where $($preds)* {
                easy_service!(@Service);
            }
            impl<$($S),*> Clone for $Service<$($S),*> where $($preds)* {
                easy_service!(@ServiceClone);
            }
        }
    };

    (@as_item $($i:item)*) => { $($i)* };

    (@Service) => {
        fn start(&self) -> io::Result<()> {
            self.0.start()
        }
        #[inline]
        fn is_terminated(&self) -> bool {
            self.0.is_terminated()
        }
        fn terminate(&self) {
            self.0.terminate()
        }
        fn blocking_join(&self) {
            self.0.blocking_join()
        }
        fn join(&self) -> BoxFuture<()> {
            self.0.join()
        }
    };
    (@ServiceClone) => {
        #[inline]
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    };

    (@Inner) => {
        #[inline]
        fn is_terminated(&self) -> bool {
            self.token.is_cancelled()
        }
        fn terminate(&self) {
            self.token.cancel();
        }
    };
    (@InnerSync) => {
        fn start(&self) -> io::Result<()> {
            let task = self
                .task
                .take()
                .ok_or_else(|| io::Error::from(io::ErrorKind::Unsupported))?;
            self.task_handle.store(Some(Box::new(self.run(task)?)));
            Ok(())
        }
        fn blocking_join(&self) {
            if let Some(task_handle) = self.task_handle.take() {
                self.terminate();
                task_handle.join().ok();
            }
        }
        fn join(&self) -> BoxFuture<()> {
            self.blocking_join();
            Box::pin(future::ready(()))
        }
    };
    (@InnerAsync) => {
        fn start(&self) -> io::Result<()> {
            let task = self
                .task
                .take()
                .ok_or_else(|| io::Error::from(io::ErrorKind::Unsupported))?;
            self.task_handle.store(Some(self.run(task)?));
            Ok(())
        }
        fn blocking_join(&self) {
            if let Some(task_handle) = self.task_handle.take() {
                self.terminate();
                tokio::spawn(task_handle);
            }
        }
        fn join(&self) -> BoxFuture<()> {
            match self.task_handle.take() {
                Some(task_handle) => {
                    self.terminate();
                    Box::pin(async move { ok!(task_handle.await) })
                }
                None => Box::pin(future::ready(())),
            }
        }
    };
    (@InnerDrop) => {
        fn drop(&mut self) {
            self.blocking_join();
        }
    };
}
