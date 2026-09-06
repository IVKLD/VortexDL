use crate::{
    api::{errors::ApiError, state::AppState},
    utils::proxy::race_proxies,
};

pub async fn resolve_youtube_stream(state: &AppState, target: &str) -> Result<String, ApiError> {
    let settings = state.settings.read().await.clone();
    let proxy_url = settings.network.get_proxy_url();
    let client = state.http_client().await;

    if let Ok(info) = yt_audio_downloader::get_stream_info_with_client(
        target,
        client,
        proxy_url.map(String::from),
    )
    .await
    {
        return Ok(info.stream_url);
    }

    let target = target.to_string();
    let url = race_proxies(&settings, move |_, proxy| {
        let target = target.clone();
        async move {
            let client = yt_audio_downloader::create_http_client_with_proxy(Some(&proxy));
            let info =
                yt_audio_downloader::get_stream_info_with_client(&target, client, Some(proxy))
                    .await?;
            Ok(info.stream_url)
        }
    })
    .await?;

    Ok(url)
}
