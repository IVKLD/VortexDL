use std::future::Future;

use tokio::sync::watch;

/// Races an async future against a tokio watch cancellation receiver.
/// Returns Some(result) on completion, or None if the cancellation signal was triggered.
pub async fn run_with_cancellation<F, T>(
    cancel_rx: Option<watch::Receiver<bool>>,
    future: F,
) -> Option<T>
where
    F: Future<Output = T>,
{
    match cancel_rx {
        Some(mut rx) => {
            if *rx.borrow() {
                return None;
            }
            tokio::select! {
                res = future => Some(res),
                _ = rx.changed() => None,
            }
        }
        None => Some(future.await),
    }
}
