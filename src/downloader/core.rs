use std::sync::Arc;
use tokio::sync::RwLock;
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use soundcloud_rs::{Client, Identifier};
use std::time::Duration;
use std::path::PathBuf;
use colored::Colorize;

use crate::storage::MusicStorage;
use crate::api::download_manager::{DownloadManager, DownloadStatus};
use super::utils::clean_filename;

#[derive(Clone)]
struct DownloadCtx {
    mp: MultiProgress,
    master: ProgressBar,
    storage: Arc<RwLock<MusicStorage>>,
    dm: Option<Arc<DownloadManager>>,
    client: Arc<Client>,
    out_dir: String,
}

pub struct DownloadTask {
    pub client: Arc<Client>,
    pub id: i64,
    pub filename: String,
    pub artwork_url: Option<String>,
    pub output_dir: String,
    pub pb: ProgressBar,
    pub master_pb: Option<ProgressBar>,
    pub storage: Arc<RwLock<MusicStorage>>,
    pub dm: Option<Arc<DownloadManager>>,
}

pub async fn download_collection(
    storage: Arc<RwLock<MusicStorage>>,
    client: &Arc<Client>,
    tracks: Vec<(i64, String, Option<String>)>,
    output_dir: &str,
    dm: Option<Arc<DownloadManager>>,
) {
    if tracks.is_empty() { return; }

    let mut to_download = Vec::new();
    let mut skipped = 0;

    {
        let s = storage.read().await;
        for (id, name, art) in tracks {
            let clean_name = clean_filename(&name);
            if s.tracks.contains_key(&id) { skipped += 1; }
            else { to_download.push((id, clean_name, art)); }
        }
    }

    if skipped > 0 {
        println!("{} Skipped {} tracks (already in storage).", "[INFO]".blue().bold(), skipped);
    }

    if to_download.is_empty() {
        println!("{} Everything is already synced!", "[INFO]".blue().bold());
        return;
    }

    let total = to_download.len() as u64;
    println!("{} Starting download of {} tracks.", "[INFO]".blue().bold(), total);

    let mp = MultiProgress::new();
    let master = mp.add(ProgressBar::new(total));
    master.enable_steady_tick(Duration::from_millis(500));
    master.set_style(ProgressStyle::default_bar()
        .template("{spinner:.yellow}[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
        .unwrap()
        .progress_chars("#>-"));

    let ctx = DownloadCtx {
        mp,
        master: master.clone(),
        storage,
        dm,
        client: client.clone(),
        out_dir: output_dir.to_string(),
    };

    stream::iter(to_download)
        .for_each_concurrent(4, |(id, name, art)| {
            let c = ctx.clone();
            async move {
                let pb = c.mp.add(ProgressBar::new_spinner());
                pb.set_style(ProgressStyle::default_spinner().template("{spinner:.cyan} {msg}").unwrap());
                
                download_one(DownloadTask {
                    client: c.client,
                    id,
                    filename: name,
                    artwork_url: art,
                    output_dir: c.out_dir,
                    pb,
                    master_pb: Some(c.master),
                    storage: c.storage,
                    dm: c.dm,
                }).await;
            }
        }).await;

    master.finish_and_clear();
    println!("{} Sync complete. {} new, {} skipped.", "[SUCCESS]".green().bold(), total, skipped);
}

pub async fn download_one(task: DownloadTask) {
    let DownloadTask { client, id, filename, artwork_url, output_dir, pb, master_pb, storage, dm, .. } = task;

    if let Some(ref m) = dm {
        m.update_status(id, DownloadStatus::Downloading).await;
    }

    pb.set_message(format!("Downloading: {}", filename));

    let res = async {
        let track = client.get_track(&Identifier::Id(id)).await?;
        let transcodings = track.media.as_ref().ok_or("No media")?.transcodings.as_ref().ok_or("No transcodings")?;
        let stream = transcodings.last().and_then(|t| t.format.as_ref()).and_then(|f| f.protocol.as_ref());

        let id_obj = Identifier::Id(id);
        let dl_fut = client.download_track(&track, &id_obj, stream, Some(&output_dir), Some(&filename));
        let art_fut = fetch_artwork(artwork_url.as_deref());

        let (dl_res, art_data) = tokio::join!(dl_fut, art_fut);
        dl_res?;

        let file_path = format!("{}/{}.mp3", output_dir, filename);
        crate::utils::metadata::save_track_info(&file_path, &id.to_string(), artwork_url.as_deref(), art_data)?;

        {
            let mut s = storage.write().await;
            s.update_track(id, PathBuf::from(&file_path), artwork_url);
        }
        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    }.await;

    if res.is_ok() {
        if let Some(ref m) = dm { m.update_status(id, DownloadStatus::Finished).await; }
        pb.println(format!("{} Done: {}", "[OK]".green().bold(), filename));
    } else {
        if let Some(ref m) = dm { m.update_status(id, DownloadStatus::Failed).await; }
        pb.println(format!("{} Failed: {}", "[ERROR]".red().bold(), filename));
        if let Err(e) = res { pb.println(format!("Details: {:#?}", e)); }
    }

    if let Some(m) = master_pb { m.inc(1); }
    pb.finish_and_clear();
}

async fn fetch_artwork(url: Option<&str>) -> Option<Vec<u8>> {
    let url = url?.replace("-large.jpg", "-t500x500.jpg");
    let resp = reqwest::get(&url).await.ok()?;
    let bytes = resp.bytes().await.ok()?;
    Some(bytes.to_vec())
}
