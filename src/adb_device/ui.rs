use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

pub fn print_err_access_remote(remote_dir: &str, device_id: &str, err: &dyn std::fmt::Display) {
    println!(
        "{} Failed to access remote music folder [{}] on {}: {}",
        "[ERROR]".red().bold(),
        remote_dir,
        device_id,
        err
    );
}

pub fn print_sync_start(device_id: &str, remote_dir: &str) {
    println!(
        "{} Syncing device [{}] to folder: {}",
        "[INFO]".blue().bold(),
        device_id,
        remote_dir
    );
}

pub fn print_removing_orphaned(count: usize, device_id: &str) {
    println!(
        "{} Removing {} orphaned tracks from device [{}]...",
        "[INFO]".blue().bold(),
        count,
        device_id
    );
}

pub fn print_pushing_new(count: usize, device_id: &str) {
    println!(
        "{} Pushing {} new tracks to device [{}]...",
        "[INFO]".blue().bold(),
        count,
        device_id
    );
}

pub fn print_sync_complete(device_id: &str) {
    println!(
        "{} Sync complete for device [{}]!",
        "[SUCCESS]".green().bold(),
        device_id
    );
}

pub fn print_push_results(pushed: u64, failed: u64) {
    if failed > 0 {
        println!(
            "{} Push results: {} succeeded, {} failed.",
            "[INFO]".blue().bold(),
            pushed,
            failed
        );
    }
}

pub fn print_deleted_orphaned(rel_path: &str) {
    println!(
        "{} Deleted orphaned track: {}",
        "[INFO]".blue().bold(),
        rel_path
    );
}

pub fn print_fail_delete_orphaned(rel_path: &str, err: &dyn std::fmt::Display) {
    println!(
        "{} Failed to delete orphaned track {}: {}",
        "[WARN]".yellow().bold(),
        rel_path,
        err
    );
}

pub fn log_warn_invalid_path(pb: &ProgressBar, id: i64) {
    pb.println(format!(
        "{} Track path for ID {} is not valid UTF-8, skipping",
        "[WARN]".yellow().bold(),
        id
    ));
}

pub fn log_err_create_artist_dir(pb: &ProgressBar, dir: &str, err: &dyn std::fmt::Display) {
    pb.println(format!(
        "{} Failed to create remote artist folder {}: {}",
        "[ERROR]".red().bold(),
        dir,
        err
    ));
}

pub fn log_warn_media_scan(pb: &ProgressBar, path: &str, err: &dyn std::fmt::Display) {
    pb.println(format!(
        "{} Failed to trigger media scan for {}: {}",
        "[WARN]".yellow().bold(),
        path,
        err
    ));
}

pub fn log_err_push(pb: &ProgressBar, path: &str, err: &dyn std::fmt::Display) {
    pb.println(format!(
        "{} Failed to push {}: {}",
        "[ERROR]".red().bold(),
        path,
        err
    ));
}

pub fn build_progress_bar(total: u64, device_id: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(&format!(
                "{{spinner:.cyan}} [{device_id}] [{{bar:30.green/white}}] {{pos}}/{{len}} {{msg}}"
            ))
            .unwrap_or_else(|_| ProgressStyle::default_bar())
            .progress_chars("█▉▊▋▌▍▎▏ "),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    pb
}
