use std::time::Duration;

use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use tracing::{error, info, warn};

pub fn sync_start(device: &str, dir: &str) {
    info!(device = %device, dir = %dir, "Syncing ADB device");
}

pub fn sync_complete(device: &str) {
    info!(device = %device, "Sync complete");
}

pub fn sync_not_needed(device: &str) {
    info!(device = %device, "Sync not needed, no changes");
}

pub fn removing(count: usize, device: &str) {
    info!(device = %device, count = %count, "Removing orphaned tracks");
}

pub fn pushing(count: usize, device: &str) {
    info!(device = %device, count = %count, "Pushing tracks");
}

pub fn push_results(pushed: u64, failed: u64) {
    if failed > 0 {
        warn!(pushed = %pushed, failed = %failed, "Track push completed with failures");
    } else {
        info!(pushed = %pushed, "Track push completed successfully");
    }
}

pub fn deleted(path: &str) {
    info!(path = %path, "Deleted file");
}

pub fn delete_failed(path: &str, e: &dyn std::fmt::Display) {
    warn!(path = %path, error = %e, "Failed to delete file");
}

pub fn remote_access_failed(dir: &str, device: &str, e: &dyn std::fmt::Display) {
    error!(dir = %dir, device = %device, error = %e, "Cannot access remote directory");
}

pub fn pb_warn(pb: &ProgressBar, msg: String) {
    pb.println(format!("{} {msg}", "[WARN]".yellow().bold()));
}

pub fn pb_err(pb: &ProgressBar, msg: String) {
    pb.println(format!("{} {msg}", "[ERROR]".red().bold()));
}

pub fn progress_bar(total: u64, device: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(&format!(
                "{{spinner:.cyan}} [{device}] [{{bar:30.green/white}}] {{pos}}/{{len}} {{msg}}"
            ))
            .unwrap_or_else(|_| ProgressStyle::default_bar())
            .progress_chars("█▉▊▋▌▍▎▏ "),
    );
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}
