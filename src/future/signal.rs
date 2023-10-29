use ::futures::{stream::BoxStream, StreamExt};
use ::tokio_stream::StreamMap;

/// Merge Ctrl+C and other quit signal into an asynchronous stream.
///
/// # Examples
///
/// ```
/// use rsx::prelude::*;
///
/// async fn test_ctrl_c() {
///     let mut stream = rsx::future::merge_ctrl_c(tokio_stream::empty().boxed());
///     println!("Please input Ctrl+C ...");
///     let _ = stream.next().await;
///     println!("Ok!");
/// }
/// ```
pub fn merge_ctrl_c(stream: BoxStream<()>) -> BoxStream<()> {
    use tokio_stream::wrappers::*;

    let mut map = StreamMap::new();
    map.insert('0', stream);

    #[cfg(windows)]
    {
        use tokio::signal::windows::*;
        map.insert('B', CtrlBreakStream::new(ctrl_break().unwrap()).boxed());
        map.insert('C', CtrlCStream::new(ctrl_c().unwrap()).boxed());
    }

    #[cfg(not(windows))]
    {
        use tokio::signal::unix::{signal, SignalKind};
        map.insert(
            'H',
            SignalStream::new(signal(SignalKind::hangup()).unwrap()).boxed(),
        );
        map.insert(
            'I',
            SignalStream::new(signal(SignalKind::interrupt()).unwrap()).boxed(),
        );
        map.insert(
            'Q',
            SignalStream::new(signal(SignalKind::quit()).unwrap()).boxed(),
        );
        map.insert(
            'T',
            SignalStream::new(signal(SignalKind::terminate()).unwrap()).boxed(),
        );
    }

    map.map(|(_, x)| x).boxed()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_term_signal() {
        let mut stream = merge_ctrl_c(tokio_stream::once(()).boxed());
        println!("Please input Ctrl+C ...");
        let _ = stream.next().await;
        println!("Ok!");
    }
}
