use std::{future::Future, sync::Arc};

use anyhow::{Result, anyhow, bail};
use tokio::{sync::Semaphore, task::JoinSet};

use crate::settings::UserSettings;

/// What: Concurrently tests an async operation across multiple fallback proxies and returns the first success.
/// Flow: Used during stream resolution and metadata retrieval when direct connections fail.
/// Why:  Instead of waiting sequentially for slow or dead proxies to timeout, this races them concurrently
///       using a JoinSet with a concurrency Semaphore, aborting all remaining tasks as soon as one succeeds.
pub async fn race_proxies<T, F, Fut>(settings: &UserSettings, op: F) -> Result<T>
where
    T: Send + 'static,
    F: Fn(Arc<UserSettings>, String) -> Fut,
    Fut: Future<Output = Result<T>> + Send + 'static,
{
    let proxies = &settings.network.fallback_proxies;
    if proxies.is_empty() {
        bail!("No fallback proxies configured");
    }

    let max_concurrent = settings.network.max_concurrent_proxies.max(1);
    let total = proxies.len();
    tracing::debug!("Racing {total} proxies (max {max_concurrent} concurrent)...");

    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    let mut tasks = JoinSet::new();
    let settings_arc = Arc::new(settings.clone());

    for proxy in proxies {
        let permit = semaphore.clone();
        let fut = op(settings_arc.clone(), proxy.clone());
        tasks.spawn(async move {
            let _permit = permit
                .acquire()
                .await
                .map_err(|_| anyhow!("Semaphore closed"))?;
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
                last_err = Some(anyhow!("Task join error: {e}"));
            }
        }
    }

    Err(last_err.unwrap_or_else(|| anyhow!("All proxies failed")))
}
