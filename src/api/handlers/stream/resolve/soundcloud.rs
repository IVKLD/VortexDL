use crate::{
    api::{errors::ApiError, state::AppState},
    utils::{
        proxy::race_proxies,
        soundcloud::{ClientBuilder, SoundcloudExt},
    },
};

pub async fn resolve_soundcloud_stream(state: &AppState, id: i64) -> Result<String, ApiError> {
    let cached_track = state.cache.soundcloud_tracks.read().await.get(&id).cloned();

    if let Ok(url) = state.client.resolve_stream_url(cached_track.as_ref(), id).await {
        return Ok(url);
    }

    let settings = state.settings.read().await.clone();
    let url = race_proxies(&settings, move |s, proxy| async move {
        let client = ClientBuilder::new(&s)
            .with_proxy(Some(&proxy))
            .build()
            .await?;
        let (url, _) = client.resolve_stream(id).await?;
        Ok(url)
    })
    .await?;

    Ok(url)
}
