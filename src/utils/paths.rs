pub fn default_output_path() -> String {
    dirs::home_dir()
        .map(|h| h.join("Downloads").to_string_lossy().into_owned())
        .unwrap_or_else(|| "./downloads".to_string())
}
