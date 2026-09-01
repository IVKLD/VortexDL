use crate::{
    api::{errors::ApiError, state::AppState},
    utils::{proxy::race_proxies, soundcloud},
};

pub async fn resolve_soundcloud_stream(state: &AppState, id: i64) -> Result<String, ApiError> {
    if let Ok((url, _)) = soundcloud::resolve_stream_url(&state.client, id).await {
        return Ok(url);
    }

    let settings = state.settings.read().await.clone();
    let url = race_proxies(&settings, move |s, proxy| async move {
        let client = soundcloud::ClientBuilder::new(&s)
            .with_proxy(Some(&proxy))
            .build()
            .await?;
        let (url, _) = soundcloud::resolve_stream_url(&client, id).await?;
        Ok(url)
    })
    .await?;

    Ok(url)
}
