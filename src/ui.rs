use std::time::Duration;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

pub fn create_total_progress_bar(mp: &MultiProgress, len: u64) -> ProgressBar {
    let total_pb = mp.add(ProgressBar::new(len));
    total_pb.enable_steady_tick(Duration::from_millis(500));
    total_pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.yellow}[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );
    total_pb
}

pub fn create_spinner(mp: &MultiProgress) -> ProgressBar {
    let pb = mp.add(ProgressBar::new_spinner());
    pb.enable_steady_tick(Duration::from_millis(100));
    pb.set_style(get_spinner_style());
    pb
}

pub fn create_standalone_spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(100));
    pb.set_style(get_spinner_style());
    pb.set_message(msg.to_string());
    pb
}

pub fn upgrade_to_download_bar(pb: &ProgressBar, total_bytes: u64) {
    pb.set_length(total_bytes);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.cyan} [{elapsed_precise}] [{bar:20.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta}) {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );
}

fn get_spinner_style() -> ProgressStyle {
    ProgressStyle::default_spinner()
        .template("{spinner:.cyan} {msg}")
        .unwrap()
        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
}
