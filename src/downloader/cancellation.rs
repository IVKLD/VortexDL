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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[tokio::test]
    async fn test_cancellation_not_triggered() {
        let (tx, rx) = watch::channel(false);
        let result = run_with_cancellation(Some(rx), async { 42 }).await;
        assert_eq!(result, Some(42));
        drop(tx);
    }

    #[tokio::test]
    async fn test_cancellation_triggered() {
        let (tx, rx) = watch::channel(false);
        let future = async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            42
        };
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            let _ = tx.send(true);
        });
        let result = run_with_cancellation(Some(rx), future).await;
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_sender_dropped_without_cancellation() {
        let (tx, rx) = watch::channel(false);
        let future = async {
            tokio::time::sleep(Duration::from_millis(20)).await;
            42
        };
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(5)).await;
            drop(tx);
        });
        let result = run_with_cancellation(Some(rx), future).await;
        assert_eq!(result, Some(42));
    }

    #[tokio::test]
    async fn test_cancellation_triggered_then_sender_dropped() {
        let (tx, rx) = watch::channel(false);
        let future = async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            42
        };
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(5)).await;
            let _ = tx.send(true);
            drop(tx);
        });
        let result = run_with_cancellation(Some(rx), future).await;
        assert_eq!(result, None);
    }
}
