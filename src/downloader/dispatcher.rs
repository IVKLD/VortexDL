use std::sync::Arc;
use tokio::sync::RwLock;
use soundcloud_rs::Client;
use std::collections::HashSet;
use crate::storage::MusicStorage;
use crate::api::download_manager::DownloadManager;
use crate::config::AppConfig;
use crate::utils::soundcloud::resolve_url;
use super::{download_likes, download_playlist, download_track};

pub async fn dispatch_download(
    url: &str,
    storage: Arc<RwLock<MusicStorage>>,
    client: &Arc<Client>,
    output_dir: &str,
    config: Arc<AppConfig>,
    dm: Option<Arc<DownloadManager>>,
) -> anyhow::Result<HashSet<i64>> {
    let resolve_res = resolve_url(client, url).await?;
    
    match resolve_res.kind.as_str() {
        "user" | "likes" => {
            download_likes(storage, client, url, output_dir, config, dm).await
        }
        "playlist" => {
            download_playlist(storage, client, url, output_dir, dm).await
        }
        "track" => {
            download_track(storage, client, resolve_res.id, output_dir, dm).await
        }
        _ => Err(anyhow::anyhow!("Unsupported resource kind: {}", resolve_res.kind)),
    }
}
