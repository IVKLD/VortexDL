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
    let parsed = Url::parse(trimmed).map_err(|e| format!("Invalid proxy URL '{val}': {e}"))?;
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

    #[arg(long, default_value_t = false, help = "Scan music library on player")]
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
