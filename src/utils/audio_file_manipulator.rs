use id3::frame::{ExtendedText, Frame, Picture, PictureType};
use id3::{Content, Tag, TagLike, Version};
use std::error::Error;

fn get_or_create_tag(file_path: &str) -> crate::models::Result<Tag> {
    match Tag::read_from_path(file_path) {
        Ok(tag) => Ok(tag),
        Err(id3::Error {
            kind: id3::ErrorKind::NoTag,
            ..
        }) => Ok(Tag::new()),
        Err(e) => Err(Box::new(e)),
    }
}

pub fn set_audio_custom_field(
    file_path: &str,
    description: &str,
    value: &str,
) -> crate::models::Result<()> {
    let mut tag = get_or_create_tag(file_path)?;

    let extended_text = ExtendedText {
        description: description.to_string(),
        value: value.to_string(),
    };

    let custom_frame = Frame::with_content("TXXX", Content::ExtendedText(extended_text));

    tag.remove_extended_text(Some(description), None);

    tag.add_frame(custom_frame);

    tag.write_to_path(file_path, Version::Id3v24)?;

    Ok(())
}

pub fn set_artwork(file_path: &str, image_data: Vec<u8>) -> crate::models::Result<()> {
    let mut tag = get_or_create_tag(file_path)?;

    let picture = Picture {
        mime_type: "image/jpeg".to_string(),
        picture_type: PictureType::CoverFront,
        description: "Cover".to_string(),
        data: image_data,
    };

    tag.remove_all_pictures();
    tag.add_picture(picture);

    tag.write_to_path(file_path, Version::Id3v24)?;

    Ok(())
}

pub fn get_mp3_custom_field(file_path: &str, description: &str) -> Option<String> {
    if let Ok(tag) = Tag::read_from_path(file_path) {
        for frame in tag.extended_texts() {
            if frame.description == description {
                return Some(frame.value.clone());
            }
        }
    }
    None
}

pub fn get_artwork(file_path: &str) -> Option<(String, Vec<u8>)> {
    if let Ok(tag) = Tag::read_from_path(file_path) {
        if let Some(picture) = tag.pictures().next() {
            return Some((picture.mime_type.clone(), picture.data.clone()));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_custom_fields() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        set_audio_custom_field(path, "test-key", "test-value").unwrap();
        
        let value = get_mp3_custom_field(path, "test-key");
        assert_eq!(value, Some("test-value".to_string()));
        
        set_audio_custom_field(path, "test-key", "new-value").unwrap();
        let value = get_mp3_custom_field(path, "test-key");
        assert_eq!(value, Some("new-value".to_string()));
        
        set_audio_custom_field(path, "other-key", "other-value").unwrap();
        assert_eq!(get_mp3_custom_field(path, "test-key"), Some("new-value".to_string()));
        assert_eq!(get_mp3_custom_field(path, "other-key"), Some("other-value".to_string()));
    }
}
