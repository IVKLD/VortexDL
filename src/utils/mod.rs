pub mod filename;
pub mod proxy;
pub mod soundcloud;
pub mod tracing;
pub mod verification;

use std::time::{SystemTime, UNIX_EPOCH};

pub fn system_time_to_secs(t: SystemTime) -> u64 {
    t.duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
