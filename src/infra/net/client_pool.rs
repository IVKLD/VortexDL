use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

use reqwest::{Client, Proxy};

use crate::infra::MutexExt;

pub const DEFAULT_USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/123.0.0.0 Safari/537.36";

#[derive(Default, Clone)]
pub struct HttpClientPool {
    clients: Arc<Mutex<HashMap<Option<String>, Client>>>,
    active_fallback: Arc<Mutex<Option<String>>>,
}

impl HttpClientPool {
    pub fn get_or_create(&self, proxy: Option<String>) -> Client {
        self.clients
            .lock_unpoisoned()
            .entry(proxy)
            .or_insert_with_key(|p| Self::build_client(p.as_deref()))
            .clone()
    }

    pub fn active_fallback_proxy(&self) -> Option<String> {
        self.active_fallback.lock_unpoisoned().clone()
    }

    pub fn set_active_fallback_proxy(&self, proxy: Option<String>) {
        *self.active_fallback.lock_unpoisoned() = proxy;
    }

    fn build_client(proxy: Option<&str>) -> Client {
        let mut builder = Client::builder()
            .user_agent(DEFAULT_USER_AGENT)
            .timeout(Duration::from_secs(30));

        if let Some(url) = proxy
            && let Ok(p) = Proxy::all(url)
        {
            builder = builder.proxy(p);
        }

        builder.build().unwrap_or_default()
    }
}
