use std::future::Future;

use anyhow::Result;

use crate::settings::UserSettings;

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

    tracing::debug!("Racing {} proxies concurrently...", proxies.len());
    let mut tasks = tokio::task::JoinSet::new();

    for proxy in proxies {
        let fut = op(settings.clone(), proxy.clone());
        tasks.spawn(fut);
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
