use std::time::{SystemTime, UNIX_EPOCH};

pub mod cancellation;
pub mod filename;
pub mod http;
pub mod paths;
pub mod proxy;
pub mod soundcloud;
pub mod verification;

pub fn setup_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();
}

pub fn system_time_to_secs(t: SystemTime) -> u64 {
    t.duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
