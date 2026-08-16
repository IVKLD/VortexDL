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

    use tokio::time::sleep;

    use super::*;

    #[tokio::test]
    async fn test_cancellation_triggered() {
        let (tx, rx) = watch::channel(false);
        let future = async {
            sleep(Duration::from_millis(100)).await;
            42
        };

        tokio::spawn(async move {
            sleep(Duration::from_millis(10)).await;
            let _ = tx.send(true);
        });

        let res = run_with_cancellation(Some(rx), future).await;
        assert_eq!(res, None);
    }

    #[tokio::test]
    async fn test_cancellation_not_triggered() {
        let (_tx, rx) = watch::channel(false);
        let future = async { 42 };

        let res = run_with_cancellation(Some(rx), future).await;
        assert_eq!(res, Some(42));
    }

    #[tokio::test]
    async fn test_sender_dropped_without_cancellation() {
        let (tx, rx) = watch::channel(false);
        let future = async {
            sleep(Duration::from_millis(20)).await;
            42
        };

        tokio::spawn(async move {
            drop(tx);
        });

        let res = run_with_cancellation(Some(rx), future).await;
        assert_eq!(res, Some(42));
    }

    #[tokio::test]
    async fn test_cancellation_triggered_then_sender_dropped() {
        let (tx, rx) = watch::channel(false);
        let future = async {
            sleep(Duration::from_millis(100)).await;
            42
        };

        tokio::spawn(async move {
            let _ = tx.send(true);
            drop(tx);
        });

        let res = run_with_cancellation(Some(rx), future).await;
        assert_eq!(res, None);
    }
}
