use std::time::Duration;

use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

macro_rules! info  { ($($a:tt)*) => { println!("{} {}", "[INFO]".blue().bold(), format!($($a)*)) } }
macro_rules! ok    { ($($a:tt)*) => { println!("{} {}", "[OK]".green().bold(), format!($($a)*)) } }
macro_rules! warn  { ($($a:tt)*) => { println!("{} {}", "[WARN]".yellow().bold(), format!($($a)*)) } }
macro_rules! err   { ($($a:tt)*) => { println!("{} {}", "[ERROR]".red().bold(), format!($($a)*)) } }

pub fn sync_start(device: &str, dir: &str) {
    info!("Syncing [{device}] → {dir}");
}

pub fn sync_complete(device: &str) {
    ok!("Sync complete [{device}]");
}

pub fn removing(count: usize, device: &str) {
    info!("Removing {count} orphaned tracks from [{device}]");
}

pub fn pushing(count: usize, device: &str) {
    info!("Pushing {count} tracks to [{device}]");
}

pub fn push_results(pushed: u64, failed: u64) {
    if failed > 0 {
        warn!("{pushed} pushed, {failed} failed");
    }
}

pub fn deleted(path: &str) {
    info!("Deleted: {path}");
}

pub fn delete_failed(path: &str, e: &dyn std::fmt::Display) {
    warn!("Failed to delete {path}: {e}");
}

pub fn remote_access_failed(dir: &str, device: &str, e: &dyn std::fmt::Display) {
    err!("Cannot access {dir} on {device}: {e}");
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
