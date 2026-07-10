use std::{future::Future, sync::Arc};

use anyhow::Result;
use tokio::sync::Semaphore;

use crate::settings::UserSettings;

/// Maximum number of proxy connections to attempt simultaneously.
/// Keeps file descriptor usage bounded (each attempt opens sockets + reads TLS certs).
const MAX_CONCURRENT_PROXIES: usize = 10;

pub async fn race_proxies<T, F, Fut>(settings: &UserSettings, op: F) -> Result<T>
where
    T: Send + 'static,
    F: Fn(UserSettings, String) -> Fut,
    Fut: Future<Output = Result<T>> + Send + 'static,
{
    let proxies = &settings.network.fallback_proxies;
    if proxies.is_empty() {
        anyhow::bail!("No fallback proxies configured");
    }

    let total = proxies.len();
    tracing::debug!("Racing {total} proxies (max {MAX_CONCURRENT_PROXIES} concurrent)...");

    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_PROXIES));
    let mut tasks = tokio::task::JoinSet::new();

    for proxy in proxies {
        let permit = semaphore.clone();
        let fut = op(settings.clone(), proxy.clone());
        tasks.spawn(async move {
            let _permit = permit
                .acquire()
                .await
                .map_err(|_| anyhow::anyhow!("Semaphore closed"))?;
            fut.await
        });
    }

    let mut last_err = None;
    while let Some(res) = tasks.join_next().await {
        match res {
            Ok(Ok(val)) => {
                tasks.abort_all();
                return Ok(val);
            }
            Ok(Err(e)) => {
                tracing::debug!("Proxy task failed: {e}");
                last_err = Some(e);
            }
            Err(e) => {
                last_err = Some(anyhow::anyhow!("Task join error: {e}"));
            }
        }
    }

    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("All proxies failed")))
}
