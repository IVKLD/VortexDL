use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use reqwest::Client;
use yt_audio_downloader::create_http_client_with_proxy;

use crate::infra::MutexExt;

#[derive(Default, Clone)]
pub struct HttpClientPool {
    clients: Arc<Mutex<HashMap<Option<String>, Client>>>,
    active_fallback: Arc<Mutex<Option<String>>>,
}

impl HttpClientPool {
    pub fn get_or_create(&self, proxy: Option<String>) -> Client {
        self.clients
            .lock_unpoisoned()
            .entry(proxy.clone())
            .or_insert_with(|| create_http_client_with_proxy(proxy.as_deref()))
            .clone()
    }

    pub fn active_fallback_proxy(&self) -> Option<String> {
        self.active_fallback.lock_unpoisoned().clone()
    }

    pub fn set_active_fallback_proxy(&self, proxy: Option<String>) {
        *self.active_fallback.lock_unpoisoned() = proxy;
    }
}
