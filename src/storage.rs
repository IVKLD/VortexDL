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
