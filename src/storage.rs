use colored::Colorize;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::{fs, path::{Path, PathBuf}};

use crate::utils::audio_file_manipulator::get_mp3_custom_field;

#[derive(Default)]
pub struct MusicStorage {
    pub tracks: HashMap<i64, PathBuf>,
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
                                    self.tracks.insert(id, path);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub async fn sync_storage(
        &self,
        remote_ids: &HashSet<i64>,
        output_dir: &str,
        mode: &str,
    ) -> Result<(), Box<dyn Error>> {
        let mut to_remove = Vec::new();

        for (id, path) in &self.tracks {
            if !remote_ids.contains(id) {
                to_remove.push((id, path.clone()));
            }
        }

        if to_remove.is_empty() {
            return Ok(());
        }

        println!(
            "{} Found {} orphaned tracks. Sync mode: {}",
            "[SYNC]".cyan().bold(),
            to_remove.len(),
            mode
        );

        if mode == "full" {
            for (_, path) in to_remove {
                if let Some(file_name) = path.file_name() {
                    println!(
                        "{} Deleting: {}",
                        "[DELETE]".red().bold(),
                        file_name.to_string_lossy()
                    );
                    fs::remove_file(path)?;
                }
            }
        } else {
            let archive_dir = Path::new(output_dir).join("Archive");
            fs::create_dir_all(&archive_dir)?;

            for (_, path) in to_remove {
                if let Some(file_name) = path.file_name() {
                    let dest = archive_dir.join(file_name);
                    println!(
                        "{} Archiving: {}",
                        "[ARCHIVE]".yellow().bold(),
                        file_name.to_string_lossy()
                    );
                    fs::rename(path, dest)?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::Write;

    #[tokio::test]
    async fn test_indexing_and_sync() {
        let dir = tempdir().unwrap();
        let output_path = dir.path();
        
        let mut storage = MusicStorage::new();
        let track1_path = output_path.join("track1.mp3");
        let track2_path = output_path.join("track2.mp3");
        
        File::create(&track1_path).unwrap();
        File::create(&track2_path).unwrap();
        
        storage.tracks.insert(1, track1_path.clone());
        storage.tracks.insert(2, track2_path.clone());
        
        let mut remote_ids = HashSet::new();
        remote_ids.insert(1);
        
        storage.sync_storage(&remote_ids, output_path.to_str().unwrap(), "full").await.unwrap();
        
        assert!(track1_path.exists());
        assert!(!track2_path.exists());
    }

    #[tokio::test]
    async fn test_sync_archive() {
        let dir = tempdir().unwrap();
        let output_path = dir.path();
        
        let mut storage = MusicStorage::new();
        let track1_path = output_path.join("track1.mp3");
        File::create(&track1_path).unwrap();
        
        storage.tracks.insert(1, track1_path.clone());
        
        let remote_ids = HashSet::new();
        
        storage.sync_storage(&remote_ids, output_path.to_str().unwrap(), "archive").await.unwrap();
        
        assert!(!track1_path.exists());
        assert!(output_path.join("Archive").join("track1.mp3").exists());
    }
}
