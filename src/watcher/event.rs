use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEvent {
    pub event_type: EventType,
    pub path: PathBuf,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: Option<EventMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    Created,
    Modified,
    Deleted,
    Renamed,
    Moved,
    Accessed,
    PermissionChanged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub size: Option<u64>,
    pub permissions: Option<String>,
    pub is_dir: bool,
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::Created => write!(f, "created"),
            EventType::Modified => write!(f, "modified"),
            EventType::Deleted => write!(f, "deleted"),
            EventType::Renamed => write!(f, "renamed"),
            EventType::Moved => write!(f, "moved"),
            EventType::Accessed => write!(f, "accessed"),
            EventType::PermissionChanged => write!(f, "permission_changed"),
        }
    }
}

impl FileEvent {
    pub fn new(event_type: EventType, path: PathBuf) -> Self {
        Self {
            event_type,
            path,
            timestamp: chrono::Utc::now(),
            metadata: None,
        }
    }

    pub fn with_metadata(mut self, metadata: EventMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }
}
