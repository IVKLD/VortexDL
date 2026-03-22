use std::{collections::HashMap, fs, path::Path};

use crate::utils::audio_file_manipulator::get_mp3_custom_field;

#[derive(Default)]
pub struct MusicStorage {
    pub tracks: HashMap<i64, String>,
}

impl MusicStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn indexing(&mut self, start_path: &Path) {
        let mut dirs_to_visit = vec![start_path.to_path_buf()];

        while let Some(current_dir) = dirs_to_visit.pop() {
            if let Ok(entries) = fs::read_dir(current_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();

                    if path.is_dir() {
                        dirs_to_visit.push(path);
                    } else if path.is_file() {
                        if let Some(path_str) = path.to_str() {
                            if let Some(content) = get_mp3_custom_field(path_str, "sc-identifier") {
                                if let Ok(id) = content.parse::<i64>() {
                                    let file_name = path
                                        .file_name()
                                        .unwrap_or_default()
                                        .to_string_lossy()
                                        .into_owned();

                                    self.tracks.insert(id, file_name);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
