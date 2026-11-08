use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use yt_audio_downloader::AudioStreamResponse;

use crate::{infra::MutexExt, providers::HlsPlaylist};

#[derive(Clone, Debug)]
pub enum StreamTarget {
    YouTube(Box<AudioStreamResponse>),
    SoundCloud(HlsPlaylist),
}

#[derive(Clone, Debug)]
pub struct CachedStream {
    pub target: StreamTarget,
    pub proxy: Option<String>,
    pub instant: Instant,
}

#[derive(Default, Clone)]
pub struct StreamCache {
    streams: Arc<Mutex<HashMap<i64, CachedStream>>>,
}

impl StreamCache {
    pub fn get(&self, id: i64) -> Option<CachedStream> {
        let streams = self.streams.lock_unpoisoned();
        let entry = streams.get(&id)?;
        (entry.instant.elapsed() < Duration::from_secs(3 * 3600)).then(|| entry.clone())
    }

    pub fn insert(&self, id: i64, stream: CachedStream) {
        self.streams.lock_unpoisoned().insert(id, stream);
    }

    pub fn remove(&self, id: i64) {
        self.streams.lock_unpoisoned().remove(&id);
    }
}
