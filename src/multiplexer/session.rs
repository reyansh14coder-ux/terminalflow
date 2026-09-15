#![allow(dead_code)]

use anyhow::Result;
use std::collections::HashMap;

use super::pane::Pane;

pub struct Session {
    name: String,
    panes: HashMap<String, Pane>,
    active_pane: Option<String>,
}

impl Session {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            panes: HashMap::new(),
            active_pane: None,
        }
    }

    pub fn create_pane(&mut self, name: &str) -> Result<()> {
        let pane = Pane::new(name);
        self.panes.insert(name.to_string(), pane);
        if self.active_pane.is_none() {
            self.active_pane = Some(name.to_string());
        }
        Ok(())
    }

    pub fn remove_pane(&mut self, name: &str) -> Result<bool> {
        let removed = self.panes.remove(name).is_some();
        if self.active_pane.as_deref() == Some(name) {
            self.active_pane = self.panes.keys().next().cloned();
        }
        Ok(removed)
    }

    pub fn list_panes(&self) -> Vec<&str> {
        self.panes.keys().map(|s| s.as_str()).collect()
    }

    pub fn get_active_pane(&self) -> Option<&Pane> {
        self.active_pane
            .as_ref()
            .and_then(|name| self.panes.get(name))
    }

    pub fn set_active_pane(&mut self, name: &str) -> bool {
        if self.panes.contains_key(name) {
            self.active_pane = Some(name.to_string());
            true
        } else {
            false
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
