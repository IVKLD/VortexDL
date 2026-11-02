use anyhow::Result;
use url::Url;

use crate::app::AppState;

pub async fn run_download(state: &AppState, url: &Url, proxies: &[String]) -> Result<()> {
    let service = state.download_service();

    if !proxies.is_empty() {
        let mut current_settings = state.settings.read().await.clone();
        current_settings.network.use_proxy = true;
        current_settings.network.fallback_proxies = proxies.to_vec();
        state.settings.update(current_settings).await?;
    }

    service.download_url(url).await?;
    Ok(())
}
