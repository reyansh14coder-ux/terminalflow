use std::collections::HashMap;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub downloads: u64,
    pub rating: f32,
    pub tags: Vec<String>,
}

pub struct PluginRegistry {
    plugins: HashMap<String, PluginInfo>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        let mut plugins = HashMap::new();
        
        // Add some built-in plugins
        plugins.insert("git-flow".to_string(), PluginInfo {
            name: "git-flow".to_string(),
            version: "1.0.0".to_string(),
            description: "Git flow workflow automation".to_string(),
            author: "TerminalFlow".to_string(),
            downloads: 15000,
            rating: 4.8,
            tags: vec!["git".to_string(), "workflow".to_string()],
        });
        
        plugins.insert("docker-compose".to_string(), PluginInfo {
            name: "docker-compose".to_string(),
            version: "1.2.0".to_string(),
            description: "Docker Compose management".to_string(),
            author: "TerminalFlow".to_string(),
            downloads: 12000,
            rating: 4.7,
            tags: vec!["docker".to_string(), "compose".to_string()],
        });
        
        plugins.insert("k8s-helper".to_string(), PluginInfo {
            name: "k8s-helper".to_string(),
            version: "0.9.0".to_string(),
            description: "Kubernetes management helper".to_string(),
            author: "Community".to_string(),
            downloads: 8000,
            rating: 4.5,
            tags: vec!["kubernetes".to_string(), "k8s".to_string()],
        });
        
        plugins.insert("terraform-fmt".to_string(), PluginInfo {
            name: "terraform-fmt".to_string(),
            version: "1.0.0".to_string(),
            description: "Terraform formatting and validation".to_string(),
            author: "Community".to_string(),
            downloads: 6000,
            rating: 4.6,
            tags: vec!["terraform".to_string(), "iac".to_string()],
        });
        
        plugins.insert("aws-cli".to_string(), PluginInfo {
            name: "aws-cli".to_string(),
            version: "2.0.0".to_string(),
            description: "AWS CLI wrapper with nice output".to_string(),
            author: "TerminalFlow".to_string(),
            downloads: 20000,
            rating: 4.9,
            tags: vec!["aws".to_string(), "cloud".to_string()],
        });
        
        Self { plugins }
    }

    pub fn search(&self, query: &str) -> Vec<&PluginInfo> {
        self.plugins.values()
            .filter(|p| {
                p.name.contains(query)
                    || p.description.contains(query)
                    || p.tags.iter().any(|t| t.contains(query))
            })
            .collect()
    }

    pub fn get_popular(&self, limit: usize) -> Vec<&PluginInfo> {
        let mut plugins: Vec<&PluginInfo> = self.plugins.values().collect();
        plugins.sort_by(|a, b| b.downloads.cmp(&a.downloads));
        plugins.into_iter().take(limit).collect()
    }

    pub fn get_top_rated(&self, limit: usize) -> Vec<&PluginInfo> {
        let mut plugins: Vec<&PluginInfo> = self.plugins.values().collect();
        plugins.sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap());
        plugins.into_iter().take(limit).collect()
    }

    pub fn get_by_tag(&self, tag: &str) -> Vec<&PluginInfo> {
        self.plugins.values()
            .filter(|p| p.tags.contains(&tag.to_string()))
            .collect()
    }
}
