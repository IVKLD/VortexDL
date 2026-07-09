use std::time::Duration;

pub fn build_http_client(proxy_url: Option<&str>, connect_timeout_secs: u64, request_timeout_secs: u64) -> reqwest::Client {
    let mut builder = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(connect_timeout_secs))
        .timeout(Duration::from_secs(request_timeout_secs));

    if let Some(proxy) = proxy_url.and_then(|p| reqwest::Proxy::all(p).ok()) {
        builder = builder.proxy(proxy);
    }

    builder.build().expect("Failed to build HTTP client")
}
