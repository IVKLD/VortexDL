use id3::frame::{ExtendedText, Frame};
use id3::{Content, Tag, TagLike, Version};
use std::error::Error;

pub fn set_audio_custom_field(
    file_path: &str,
    description: &str,
    value: &str,
) -> Result<(), Box<dyn Error>> {
    let mut tag = match Tag::read_from_path(file_path) {
        Ok(tag) => tag,
        Err(id3::Error {
            kind: id3::ErrorKind::NoTag,
            ..
        }) => Tag::new(),
        Err(e) => return Err(Box::new(e)),
    };

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
