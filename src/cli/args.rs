use std::{net::IpAddr, path::PathBuf};

use clap::{ArgAction, ArgGroup, Parser};
use url::Url;

use crate::{domain::SourcePlatform, settings::UserSettings};

fn parse_download_url(val: &str) -> Result<Url, String> {
    let trimmed = val.trim();
    if trimmed.is_empty() {
        return Err("URL cannot be empty".to_string());
    }
    let url_str = if !trimmed.contains("://") {
        format!("https://{trimmed}")
    } else {
        trimmed.to_string()
    };
    let parsed = Url::parse(&url_str).map_err(|e| format!("Invalid URL '{val}': {e}"))?;
    let platform = SourcePlatform::detect_from_url_str(parsed.as_str());
    if platform == SourcePlatform::Local {
        return Err(format!(
            "Unsupported URL '{val}'. Only SoundCloud and YouTube URLs are supported."
        ));
    }
    Ok(parsed)
}

fn parse_proxy_url(val: &str) -> Result<String, String> {
    let trimmed = val.trim();
    if trimmed.is_empty() {
        return Err("Proxy URL cannot be empty".to_string());
    }
    let parsed =
        Url::parse(trimmed).map_err(|e| format!("Invalid proxy URL '{val}': {e}"))?;
    match parsed.scheme() {
        "http" | "https" | "socks5" | "socks5h" => Ok(trimmed.to_string()),
        other => Err(format!(
            "Unsupported proxy protocol '{other}' in '{val}'. Supported protocols: http, https, socks5, socks5h."
        )),
    }
}

#[derive(Parser, Debug)]
#[command(name = "vortex-dl")]
#[command(version)]
#[command(
    about = "VortexDL: High-performance SoundCloud and YouTube downloader with intelligent sync",
    long_about = None
)]
#[command(disable_version_flag = true)]
#[command(arg_required_else_help = true)]
#[command(group(
    ArgGroup::new("mode")
        .args(["url", "serve", "sync_player", "scan_player"])
        .required(true)
))]
pub struct Args {
    #[arg(
        short = 'v',
        long = "version",
        action = ArgAction::Version,
        help = "Print version information"
    )]
    pub version: (),

    #[arg(
        value_name = "URL",
        value_parser = parse_download_url,
        help = "The SoundCloud or YouTube URL to download (track, playlist, or user likes)"
    )]
    pub url: Option<Url>,

    #[arg(
        short,
        long,
        env = "VORTEX_OUTPUT_DIR",
        help = "Directory where the music will be saved"
    )]
    pub output: Option<PathBuf>,

    #[arg(
        long,
        default_value_t = false,
        help = "Run the application as a REST API server"
    )]
    pub serve: bool,

    #[arg(
        long,
        default_value_t = false,
        help = "Sync music files with configured and connected ADB devices / music player"
    )]
    pub sync_player: bool,

    #[arg(
        long,
        default_value_t = false,
        help = "Scan music library on player"
    )]
    pub scan_player: bool,

    #[arg(
        long,
        default_value = "127.0.0.1",
        help = "Host address to bind the server to"
    )]
    pub host: IpAddr,

    #[arg(
        long,
        env = "VORTEX_PORT",
        default_value_t = 3200,
        help = "Port to listen on for the REST API and WebUI"
    )]
    pub port: u16,

    #[arg(
        long,
        value_delimiter = ',',
        value_parser = parse_proxy_url,
        help = "Fallback proxy URLs (comma-separated, e.g. http://... or socks5://...)"
    )]
    pub proxies: Vec<String>,
}

impl Args {
    pub fn resolve_output_dir(&self, settings: &UserSettings) -> PathBuf {
        self.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(&settings.downloads.output_path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_download_url_valid() {
        let sc = parse_download_url("https://soundcloud.com/artist/track").unwrap();
        assert_eq!(sc.as_str(), "https://soundcloud.com/artist/track");

        let sc_no_scheme = parse_download_url("soundcloud.com/artist/track").unwrap();
        assert_eq!(sc_no_scheme.as_str(), "https://soundcloud.com/artist/track");

        let yt = parse_download_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ").unwrap();
        assert_eq!(yt.as_str(), "https://www.youtube.com/watch?v=dQw4w9WgXcQ");

        let yt_short = parse_download_url("youtu.be/dQw4w9WgXcQ").unwrap();
        assert_eq!(yt_short.as_str(), "https://youtu.be/dQw4w9WgXcQ");
    }

    #[test]
    fn test_parse_download_url_invalid() {
        assert!(parse_download_url("").is_err());
        assert!(parse_download_url("   ").is_err());
        let err = parse_download_url("https://google.com").unwrap_err();
        assert!(err.contains("Unsupported URL"));
    }

    #[test]
    fn test_parse_proxy_url() {
        assert!(parse_proxy_url("http://127.0.0.1:8080").is_ok());
        assert!(parse_proxy_url("socks5://user:pass@127.0.0.1:1080").is_ok());
        assert!(parse_proxy_url("ftp://127.0.0.1:21").is_err());
        assert!(parse_proxy_url("not_a_url").is_err());
    }

    #[test]
    fn test_args_modes_mutual_exclusion() {
        // Single action works
        assert!(Args::try_parse_from(["vortex-dl", "https://soundcloud.com/artist/track"]).is_ok());
        assert!(Args::try_parse_from(["vortex-dl", "--serve"]).is_ok());
        assert!(Args::try_parse_from(["vortex-dl", "--sync-player"]).is_ok());
        assert!(Args::try_parse_from(["vortex-dl", "--scan-player"]).is_ok());

        // Conflicting actions fail
        assert!(Args::try_parse_from(["vortex-dl", "--serve", "--sync-player"]).is_err());
        assert!(Args::try_parse_from(["vortex-dl", "--serve", "https://soundcloud.com/artist/track"]).is_err());
        assert!(Args::try_parse_from(["vortex-dl", "--sync-player", "--scan-player"]).is_err());

        // Options only (no action mode) fail
        assert!(Args::try_parse_from(["vortex-dl", "--output", "/tmp"]).is_err());
        assert!(Args::try_parse_from(["vortex-dl", "--port", "3200"]).is_err());
    }
}
