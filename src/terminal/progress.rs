use std::{fmt::Display, time::Duration};

use colored::Colorize;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

pub fn create_total_progress_bar(mp: &MultiProgress, len: u64) -> ProgressBar {
    let total_pb = mp.add(ProgressBar::new(len));
    total_pb.enable_steady_tick(Duration::from_millis(500));
    total_pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.yellow}[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_bar())
            .progress_chars("#>-"),
    );
    total_pb
}

pub fn create_device_progress_bar(total: u64, device: &str) -> ProgressBar {
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

pub fn pb_warn(pb: &ProgressBar, msg: impl Display) {
    pb.println(format!("{} {msg}", "[WARN]".yellow().bold()));
}

pub fn pb_err(pb: &ProgressBar, msg: impl Display) {
    pb.println(format!("{} {msg}", "[ERROR]".red().bold()));
}
