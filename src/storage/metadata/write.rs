use std::path::Path;

use anyhow::Result;
use id3::{
    Content, TagLike, Version,
    frame::{Frame, Picture, PictureType},
};
use url::Url;

use super::id3::{get_tag, set_txxx};
use crate::{
    constants::{TAG_ARTWORK_URL, TAG_PLATFORM, TAG_SOURCE_URL, TAG_TRACK_ID},
    types::SourcePlatform,
};

pub struct SaveTrackArgs<'a> {
    pub path: &'a Path,
    pub sc_id: &'a str,
    pub title: &'a str,
    pub artist: &'a str,
    pub artwork_url: Option<&'a Url>,
    pub source_url: Option<&'a Url>,
    pub artwork_data: Option<Vec<u8>>,
}

/// What: Writes ID3v2.3 tags and embedded cover artwork into an audio file.
/// Flow: Called by `finalize_single_track` after downloading completes.
/// Why:  Ensures downloaded tracks have consistent artist/title tags, embedded album artwork,
///       and custom metadata (like track ID) so VortexDL can recognize them on future rescans.
pub fn save_track_info(args: SaveTrackArgs) -> Result<()> {
    let mut tag = get_tag(args.path)?;

    tag.set_title(args.title);
    tag.set_artist(args.artist);

    let platform =
        SourcePlatform::detect_from_optional_url(args.source_url.map(|u| u.as_str())).as_str();

    set_txxx(&mut tag, TAG_TRACK_ID, args.sc_id);
    set_txxx(&mut tag, TAG_PLATFORM, platform);

    if let Some(url) = args.artwork_url {
        set_txxx(&mut tag, TAG_ARTWORK_URL, url.as_str());
    }
    if let Some(url) = args.source_url {
        set_txxx(&mut tag, TAG_SOURCE_URL, url.as_str());
    }

    if let Some(data) = args.artwork_data {
        tag.remove_all_pictures();
        tag.add_frame(Frame::with_content(
            "APIC",
            Content::Picture(Picture {
                mime_type: "image/jpeg".to_string(),
                picture_type: PictureType::CoverFront,
                description: "Cover".to_string(),
                data,
            }),
        ));
    }

    tag.write_to_path(args.path, Version::Id3v23)?;
    Ok(())
}
