use std::future::Future;

use tokio::sync::watch;

/// What: Runs an asynchronous future while listening for a cancellation signal.
/// Flow: Wraps long-running operations (like track downloading and audio processing).
/// Why:  Uses `tokio::select!` so that if the user cancels the task in the UI, the future aborts immediately
///       without leaving dangling connections or incomplete files on disk.
pub async fn run_with_cancellation<F, T>(
    cancel_rx: Option<watch::Receiver<bool>>,
    future: F,
) -> Option<T>
where
    F: Future<Output = T>,
{
    let Some(mut rx) = cancel_rx else {
        return Some(future.await);
    };

    if *rx.borrow() {
        return None;
    }

    tokio::pin!(future);

    loop {
        tokio::select! {
            res = &mut future => return Some(res),
            changed = rx.changed() => {
                match changed {
                    Ok(()) if *rx.borrow() => return None,
                    Ok(()) => continue,
                    Err(_) => {
                        if *rx.borrow() {
                            return None;
                        }
                        return Some(future.await);
                    }
                }
            }
        }
    }
}
