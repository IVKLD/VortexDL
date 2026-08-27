use std::collections::HashMap;

use anyhow::{Result, anyhow};
use indicatif::ProgressBar;
use soundcloud_rs::{Client, Identifier, ResolvedResource, UserTrackLikesQuery};
use url::Url;

use crate::{
    api::download_manager::{MessageLevel, ServerEvent},
    downloader::{Context, DiscoveredMusicTrack},
    ui::create_standalone_spinner,
    utils::{proxy::race_proxies, soundcloud},
};

pub fn init_progress_spinner(ctx: &Context, msg: &str) -> ProgressBar {
    let pb = create_standalone_spinner(msg);

    if let Some(manager) = &ctx.dm {
        manager.broadcast_event(ServerEvent::Message {
            message: msg.to_string(),
            level: MessageLevel::Info,
        });
    }

    pb
}

pub async fn discover_liked_tracks(
    ctx: &Context,
    client: &Client,
    id: i64,
) -> Result<Vec<DiscoveredMusicTrack>> {
    let mut offset: Option<String> = None;
    let pb = init_progress_spinner(ctx, "Fetching track list...");
    let mut tracks = Vec::new();
    let limit = ctx.settings.read().await.system.limit_per_page;

    loop {
        let query = UserTrackLikesQuery {
            limit: Some(limit),
            offset: offset.clone(),
        };
        let res = client
            .get_user_track_likes(&Identifier::Id(id), Some(&query))
            .await?;
        if res.collection.is_empty() {
            break;
        }

        tracks.extend(
            res.collection
                .into_iter()
                .filter_map(|item| item.track.and_then(DiscoveredMusicTrack::from_track)),
        );

        let Some(href) = res.next_href else {
            break;
        };
        match Url::parse(&href)?
            .query_pairs()
            .find(|(k, _)| k == "offset")
            .map(|(_, v)| v.into_owned())
        {
            Some(next_offset) => offset = Some(next_offset),
            None => break,
        }
    }

    pb.finish_and_clear();
    Ok(tracks)
}

pub async fn discover_playlist_tracks(
    ctx: &Context,
    client: &Client,
    id: i64,
) -> Result<Vec<DiscoveredMusicTrack>> {
    let playlist = client.get_playlist(&Identifier::Id(id)).await?;
    let collection = playlist
        .tracks
        .ok_or_else(|| anyhow!("No tracks found in playlist"))?;

    let mut tracks: Vec<DiscoveredMusicTrack> = collection
        .into_iter()
        .filter_map(DiscoveredMusicTrack::from_track)
        .collect();

    let missing_ids: Vec<i64> = tracks
        .iter()
        .filter(|track| track.title == "Unknown")
        .map(|track| track.id)
        .collect();

    if !missing_ids.is_empty() {
        let pb = init_progress_spinner(ctx, "Resolving playlist track metadata...");

        let index: HashMap<i64, usize> =
            tracks.iter().enumerate().map(|(i, t)| (t.id, i)).collect();

        for chunk in missing_ids.chunks(50) {
            if let Ok(fetched_tracks) = client.get_tracks(chunk).await {
                for track in fetched_tracks {
                    let Some(track_id) = track.id else { continue };
                    let Some(updated) = DiscoveredMusicTrack::from_track(track) else {
                        continue;
                    };
                    if let Some(&idx) = index.get(&track_id) {
                        tracks[idx] = updated;
                    }
                }
            }
        }
        pb.finish_and_clear();
    }

    Ok(tracks)
}

pub async fn resolve_tracks_from_url(
    ctx: &Context,
    url: &Url,
) -> Result<Vec<DiscoveredMusicTrack>> {
    let pb = init_progress_spinner(ctx, "Resolving SoundCloud URL...");

    let result = match discover_tracks_from_url(ctx, url, &ctx.client).await {
        Ok(tracks) => Ok(tracks),
        Err(e) => {
            tracing::debug!("Direct discovery failed: {e}. Trying fallback proxies...");
            let settings = ctx.settings.read().await.clone();

            if !settings.network.use_proxy || settings.network.fallback_proxies.is_empty() {
                return Err(e);
            }

            race_proxies(&settings, |s, proxy| {
                let ctx = ctx.clone();
                let url = url.clone();
                async move {
                    let proxied_client = soundcloud::ClientBuilder::new(&s)
                        .with_proxy(Some(&proxy))
                        .build()
                        .await?;
                    discover_tracks_from_url(&ctx, &url, &proxied_client).await
                }
            })
            .await
            .map_err(|proxy_err| {
                anyhow::anyhow!("SoundCloud discovery failed: {e} (proxies: {proxy_err})")
            })
        }
    };

    pb.finish_and_clear();
    result
}

async fn discover_tracks_from_url(
    ctx: &Context,
    url: &Url,
    client: &soundcloud_rs::Client,
) -> Result<Vec<DiscoveredMusicTrack>> {
    let res = client.resolve_url(url).await?;

    let all_tracks = match res {
        ResolvedResource::User(user) => {
            let id = user
                .id
                .ok_or_else(|| anyhow::anyhow!("User ID is missing"))?;
            discover_liked_tracks(ctx, client, id).await?
        }
        ResolvedResource::Playlist(playlist) => {
            let id = playlist
                .id
                .ok_or_else(|| anyhow::anyhow!("Playlist ID is missing"))?;
            discover_playlist_tracks(ctx, client, id).await?
        }
        ResolvedResource::Track(track) => {
            let discovered = DiscoveredMusicTrack::from_track(track)
                .ok_or_else(|| anyhow::anyhow!("Track missing required ID"))?;
            vec![discovered]
        }
    };

    Ok(all_tracks)
}
