use indicatif::{ProgressBar, ProgressStyle};

pub fn build_progress_bar(total: u64, device_id: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(&format!(
                "{{spinner:.cyan}} [{device_id}] [{{bar:30.green/white}}] {{pos}}/{{len}} {{msg}}"
            ))
            .unwrap()
            .progress_chars("█▉▊▋▌▍▎▏ "),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    pb
}
