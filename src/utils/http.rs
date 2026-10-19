use std::time::Duration;

pub fn build_http_client(
    proxy_url: Option<&str>,
    connect_timeout_secs: u64,
    request_timeout_secs: u64,
) -> reqwest::Client {
    let mut builder = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(connect_timeout_secs))
        .timeout(Duration::from_secs(request_timeout_secs));

    if let Some(proxy_str) = proxy_url.filter(|p| !p.trim().is_empty()) {
        match reqwest::Proxy::all(proxy_str) {
            Ok(proxy) => {
                builder = builder.proxy(proxy);
            }
            Err(e) => {
                tracing::error!("Invalid proxy URL '{proxy_str}': {e}");
            }
        }
    }

    builder.build().unwrap_or_else(|e| {
        tracing::error!(
            "Failed to build configured HTTP client: {e}. Falling back to default client."
        );
        reqwest::Client::new()
    })
}
