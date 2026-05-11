use reqwest::Client as HttpClient;
use tokio::task::JoinHandle;

use crate::utils::soundcloud::fetch_artwork as fetch_art;

pub(super) fn start_artwork_download(
    http: &HttpClient,
    url: String,
) -> JoinHandle<Option<Vec<u8>>> {
    let http = http.clone();

    tokio::spawn(async move { fetch_art(&http, &url).await })
}
