use std::time::Duration;

use anyhow::Result;
use soundcloud_rs::Client;

use crate::models::{ResolveQuery, ResolveResponse};

pub async fn resolve_url(client: &Client, url: &str) -> Result<ResolveResponse> {
    let response: ResolveResponse = client
        .get(
            "resolve",
            Some(&ResolveQuery {
                url: Some(url.to_string()),
            }),
        )
        .await?;

    Ok(response)
}

pub async fn fetch_artwork(client: &reqwest::Client, url: Option<&str>) -> Option<Vec<u8>> {
    let url = url?.replace("-large.jpg", "-t500x500.jpg");
    let resp = client
        .get(&url)
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .ok()?;
    resp.bytes().await.ok().map(|b| b.to_vec())
}
