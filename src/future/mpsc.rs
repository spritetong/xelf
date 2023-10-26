use futures::Sink;
use pin_project::pin_project;
use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::sync::mpsc::{Receiver, UnboundedSender};
use tokio_util::sync::PollSender;

pub type MpscStream<S, R> = super::duplex::DuplexStream<PollSender<S>, Receiver<R>, S>;

#[pin_project]
pub struct UnboundedSink<T> {
    #[pin]
    sender: Option<UnboundedSender<T>>,
}

impl<T> UnboundedSink<T> {
    pub fn new(sender: UnboundedSender<T>) -> Self {
        Self {
            sender: Some(sender),
        }
    }

    crate::future_delegate_access_inner!(sender, Option<UnboundedSender<T>>, ());
}

impl<T> Sink<T> for UnboundedSink<T> {
    type Error = io::Error;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if self.sender.is_some() {
            Poll::Ready(Ok(()))
        } else {
            Poll::Ready(Err(io::ErrorKind::NotConnected.into()))
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if self.sender.is_some() {
            Poll::Ready(Ok(()))
        } else {
            Poll::Ready(Err(io::ErrorKind::NotConnected.into()))
        }
    }

    fn start_send(self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        if let Some(sender) = &self.sender {
            if sender.send(item).is_ok() {
                return Ok(());
            }
        }
        Err(io::ErrorKind::NotConnected.into())
    }

    fn poll_close(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.sender.take();
        Poll::Ready(Ok(()))
    }
}
