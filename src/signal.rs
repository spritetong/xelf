use ::tokio_stream::{Stream, StreamExt as _};

/// Merge Ctrl+C and other quit signal into an asynchronous stream.
///
/// # Examples
///
/// ```
/// use rsx::prelude::*;
/// 
/// async fn test_ctrl_c() {
///     let stream = rsx::signal::merge_ctrl_c(tokio_stream::empty());
///     tokio::pin!(stream);
///     println!("Please input Ctrl+C ...");
///     let _ = stream.next().await;
///     println!("Ok!");
/// }
/// ```
pub fn merge_ctrl_c<S>(stream: S) -> impl Stream<Item = ()>
where
    S: Stream<Item = ()> + 'static,
{
    use tokio_stream::wrappers::*;

    #[cfg(windows)]
    let stream = {
        use tokio::signal::windows::*;
        stream
            .merge(CtrlBreakStream::new(ctrl_break().unwrap()))
            .merge(CtrlCStream::new(ctrl_c().unwrap()))
    };

    #[cfg(not(windows))]
    let stream = {
        use tokio::signal::unix::{signal, SignalKind};
        stream
            .merge(SignalStream::new(signal(SignalKind::hangup()).unwrap()))
            .merge(SignalStream::new(signal(SignalKind::interrupt()).unwrap()))
            .merge(SignalStream::new(signal(SignalKind::quit()).unwrap()))
            .merge(SignalStream::new(signal(SignalKind::terminate()).unwrap()))
    };

    stream
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_term_signal() {
        let stream = merge_ctrl_c(tokio_stream::once(()));
        tokio::pin!(stream);
        println!("Please input Ctrl+C ...");
        let _ = stream.next().await;
        println!("Ok!");
    }
}
