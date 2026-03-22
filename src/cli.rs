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
}
