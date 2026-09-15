#![allow(dead_code)]

use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, SystemTime};

use super::event::{EventType, FileEvent};

pub struct DirectoryWatcher {
    path: PathBuf,
    interval: Duration,
    recursive: bool,
    ignore_patterns: Vec<String>,
}

impl DirectoryWatcher {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            interval: Duration::from_millis(500),
            recursive: true,
            ignore_patterns: vec![
                ".git".to_string(),
                "node_modules".to_string(),
                "__pycache__".to_string(),
                ".DS_Store".to_string(),
            ],
        }
    }

    pub fn with_interval(mut self, interval_ms: u64) -> Self {
        self.interval = Duration::from_millis(interval_ms);
        self
    }

    pub fn recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }

    pub fn ignore(mut self, patterns: Vec<String>) -> Self {
        self.ignore_patterns = patterns;
        self
    }

    pub fn watch(&self) -> Result<Receiver<FileEvent>> {
        let (sender, receiver) = mpsc::channel();
        let path = self.path.clone();
        let interval = self.interval;
        let recursive = self.recursive;
        let ignore_patterns = self.ignore_patterns.clone();

        std::thread::spawn(move || {
            let mut file_states: HashMap<PathBuf, SystemTime> = HashMap::new();

            if let Ok(entries) = scan_directory(&path, recursive, &ignore_patterns) {
                for (entry_path, modified) in entries {
                    file_states.insert(entry_path, modified);
                }
            }

            loop {
                std::thread::sleep(interval);

                if let Ok(entries) = scan_directory(&path, recursive, &ignore_patterns) {
                    let mut current_files: HashMap<PathBuf, SystemTime> = HashMap::new();

                    for (entry_path, modified) in entries {
                        current_files.insert(entry_path.clone(), modified);

                        if !file_states.contains_key(&entry_path) {
                            let event = FileEvent::new(EventType::Created, entry_path.clone());
                            let _ = sender.send(event);
                        } else if let Some(last_modified) = file_states.get(&entry_path) {
                            if modified > *last_modified {
                                let event = FileEvent::new(EventType::Modified, entry_path.clone());
                                let _ = sender.send(event);
                            }
                        }
                    }

                    for entry_path in file_states.keys() {
                        if !current_files.contains_key(entry_path) {
                            let event = FileEvent::new(EventType::Deleted, entry_path.clone());
                            let _ = sender.send(event);
                        }
                    }

                    file_states = current_files;
                }
            }
        });

        Ok(receiver)
    }
}

fn scan_directory(
    path: &PathBuf,
    recursive: bool,
    ignore_patterns: &[String],
) -> Result<Vec<(PathBuf, SystemTime)>> {
    let mut entries = Vec::new();

    if let Ok(dir_entries) = std::fs::read_dir(path) {
        for entry in dir_entries.flatten() {
            let entry_path = entry.path();

            let should_ignore = ignore_patterns
                .iter()
                .any(|pattern| entry_path.to_string_lossy().contains(pattern));

            if should_ignore {
                continue;
            }

            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    entries.push((entry_path.clone(), modified));

                    if recursive && metadata.is_dir() {
                        if let Ok(sub_entries) =
                            scan_directory(&entry_path, recursive, ignore_patterns)
                        {
                            entries.extend(sub_entries);
                        }
                    }
                }
            }
        }
    }

    Ok(entries)
}
