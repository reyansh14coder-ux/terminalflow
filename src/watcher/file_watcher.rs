use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::mpsc::{self, Sender, Receiver};
use std::time::{Duration, SystemTime};

use super::event::{FileEvent, EventType};

pub struct FileWatcher {
    path: PathBuf,
    interval: Duration,
    sender: Option<Sender<FileEvent>>,
    last_modified: Option<SystemTime>,
}

impl FileWatcher {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            interval: Duration::from_millis(100),
            sender: None,
            last_modified: None,
        }
    }

    pub fn with_interval(mut self, interval_ms: u64) -> Self {
        self.interval = Duration::from_millis(interval_ms);
        self
    }

    pub fn watch(&mut self) -> Result<Receiver<FileEvent>> {
        let (sender, receiver) = mpsc::channel();
        self.sender = Some(sender);
        
        let path = self.path.clone();
        let interval = self.interval;
        
        std::thread::spawn(move || {
            let mut last_modified = std::fs::metadata(&path)
                .and_then(|m| m.modified())
                .ok();
            
            loop {
                std::thread::sleep(interval);
                
                if let Ok(metadata) = std::fs::metadata(&path) {
                    if let Ok(modified) = metadata.modified() {
                        if let Some(last) = last_modified {
                            if modified > last {
                                let event = FileEvent::new(
                                    EventType::Modified,
                                    path.clone(),
                                );
                                let _ = sender.send(event);
                                last_modified = Some(modified);
                            }
                        } else {
                            last_modified = Some(modified);
                        }
                    }
                }
            }
        });
        
        Ok(receiver)
    }

    pub fn watch_for_creation(&self) -> Result<Receiver<FileEvent>> {
        let (sender, receiver) = mpsc::channel();
        let path = self.path.clone();
        let interval = self.interval;
        
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(interval);
                
                if path.exists() {
                    let event = FileEvent::new(EventType::Created, path.clone());
                    let _ = sender.send(event);
                    break;
                }
            }
        });
        
        Ok(receiver)
    }

    pub fn watch_for_deletion(&self) -> Result<Receiver<FileEvent>> {
        let (sender, receiver) = mpsc::channel();
        let path = self.path.clone();
        let interval = self.interval;
        
        std::thread::spawn(move || {
            let mut existed = path.exists();
            
            loop {
                std::thread::sleep(interval);
                
                let exists = path.exists();
                
                if existed && !exists {
                    let event = FileEvent::new(EventType::Deleted, path.clone());
                    let _ = sender.send(event);
                    break;
                }
                
                existed = exists;
            }
        });
        
        Ok(receiver)
    }
}
