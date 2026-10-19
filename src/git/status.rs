#![allow(dead_code)]

use anyhow::Result;
use git2::{Repository, Status, StatusOptions};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct GitStatus {
    pub branch: String,
    pub modified: Vec<String>,
    pub added: Vec<String>,
    pub deleted: Vec<String>,
    pub untracked: Vec<String>,
    pub staged: Vec<String>,
    pub ahead: u32,
    pub behind: u32,
}

impl GitStatus {
    pub fn from_repo(path: &Path) -> Result<Self> {
        let repo = Repository::open(path)?;

        let head = repo.head()?;
        let branch = head.shorthand().unwrap_or("HEAD").to_string();

        let mut options = StatusOptions::new();
        options.include_untracked(true);
        options.recurse_untracked_dirs(true);

        let statuses = repo.statuses(Some(&mut options))?;

        let mut modified = Vec::new();
        let mut added = Vec::new();
        let mut deleted = Vec::new();
        let mut untracked = Vec::new();
        let mut staged = Vec::new();

        for entry in statuses.iter() {
            let path = entry.path().unwrap_or("").to_string();
            let status = entry.status();

            if status.contains(Status::WT_MODIFIED) {
                modified.push(path.clone());
            }
            if status.contains(Status::WT_NEW) {
                untracked.push(path.clone());
            }
            if status.contains(Status::WT_DELETED) {
                deleted.push(path.clone());
            }
            if status.contains(Status::INDEX_NEW) || status.contains(Status::INDEX_MODIFIED) {
                staged.push(path.clone());
            }
            if status.contains(Status::INDEX_NEW) {
                added.push(path.clone());
            }
        }

        let mut ahead = 0u32;
        let mut behind = 0u32;
        if let Some(shorthand) = head.shorthand() {
            if let Ok(branch) = repo.find_branch(shorthand, git2::BranchType::Local) {
                if let Ok(upstream) = branch.upstream() {
                    if let (Some(local), Some(remote)) = (branch.get().target(), upstream.get().target())
                    {
                        if let Ok((a, b)) = repo.graph_ahead_behind(local, remote) {
                            ahead = a as u32;
                            behind = b as u32;
                        }
                    }
                }
            }
        }

        Ok(Self {
            branch,
            modified,
            added,
            deleted,
            untracked,
            staged,
            ahead,
            behind,
        })
    }

    pub fn total_changes(&self) -> usize {
        self.modified.len() + self.added.len() + self.deleted.len() + self.untracked.len()
    }

    pub fn is_clean(&self) -> bool {
        self.modified.is_empty()
            && self.added.is_empty()
            && self.deleted.is_empty()
            && self.untracked.is_empty()
            && self.staged.is_empty()
    }

    pub fn has_changes(&self) -> bool {
        !self.is_clean()
    }
}
