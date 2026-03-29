use clap::{ArgGroup, Parser};

#[derive(Parser, Debug)]
#[command(name = "sc-downloader")]
#[command(about = "SoundCloud Downloader", long_about = None)]
#[command(group(
    ArgGroup::new("target")
        // .required(true)
        .args(["playlist_url", "user_url"])
))]
pub struct Args {
    #[arg(short, long)]
    pub playlist_url: Option<String>,

    #[arg(short, long)]
    pub user_url: Option<String>,

    #[arg(short, long)]
    pub output: Option<String>,

    /// Start the HTTP REST API server instead of running the CLI downloader.
    #[arg(long, default_value_t = false)]
    pub serve: bool,

    /// Port to bind the HTTP server to (only used with --serve).
    #[arg(long, default_value_t = 3000)]
    pub port: u16,
}
