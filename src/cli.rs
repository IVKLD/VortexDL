use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "sc-downloader")]
#[command(about = "SoundCloud Downloader", long_about = None)]
pub struct Args {
    #[arg()]
    pub url: Option<String>,

    #[arg(short, long)]
    pub output: Option<String>,

    #[arg(short, long, default_value_t = false)]
    pub sync: bool,

    #[arg(long, default_value = "silent")]
    pub sync_mode: String,

    #[arg(long, default_value_t = false)]
    pub serve: bool,

    #[arg(long, default_value_t = 3000)]
    pub port: u16,
}
