use std::collections::HashSet;

use anyhow::Result;
use url::Url;

use crate::database::{get_previous_ids, save_sync_ids};

pub async fn sync_url_ids(url: &Url, current_ids: &HashSet<i64>) -> Result<()> {
    let url_str = url.as_str();
    let prev_ids = get_previous_ids(url_str)?;
    let accumulated_ids: HashSet<i64> = prev_ids.union(current_ids).copied().collect();
    save_sync_ids(url_str, &accumulated_ids)?;
    Ok(())
}
