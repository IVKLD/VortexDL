use soundcloud_rs::Client;

use crate::models::{ResolveQuery, ResolveResponse};

pub async fn resolve_url(client: &Client, url: &str) -> anyhow::Result<ResolveResponse> {
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
