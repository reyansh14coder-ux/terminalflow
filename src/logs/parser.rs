#![allow(dead_code)]

use anyhow::{Context, Result};
use regex::Regex;
use std::path::Path;
use std::sync::OnceLock;

use super::viewer::{LogEntry, LogLevel};

pub struct LogParser {
    patterns: Vec<LogPattern>,
}

struct LogPattern {
    regex: Regex,
    timestamp_group: Option<usize>,
    level_group: Option<usize>,
    message_group: usize,
    source_group: Option<usize>,
}

impl LogParser {
    pub fn new() -> Self {
        let patterns = vec![
            LogPattern {
                regex: Regex::new(r#"^(\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})?)\s+(\w+)\s+(.+)$"#).unwrap(),
                timestamp_group: Some(1),
                level_group: Some(2),
                message_group: 3,
                source_group: None,
            },
            LogPattern {
                regex: Regex::new(r#"^(\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})?)\s+\[(\w+)\]\s+\[([^\]]+)\]\s+(.+)$"#).unwrap(),
                timestamp_group: Some(1),
                level_group: Some(2),
                message_group: 4,
                source_group: Some(3),
            },
            LogPattern {
                regex: Regex::new(r#"^(\w+):\s+(.+)$"#).unwrap(),
                timestamp_group: None,
                level_group: Some(1),
                message_group: 2,
                source_group: None,
            },
        ];

        Self { patterns }
    }

    pub fn parse_line(&self, line: &str) -> Option<LogEntry> {
        if line.trim().is_empty() {
            return None;
        }

        for pattern in &self.patterns {
            if let Some(caps) = pattern.regex.captures(line) {
                let timestamp = pattern
                    .timestamp_group
                    .and_then(|g| caps.get(g))
                    .map(|m| m.as_str().to_string());
                let level = pattern
                    .level_group
                    .and_then(|g| caps.get(g))
                    .map(|m| LogLevel::from_str(m.as_str()))
                    .unwrap_or_else(|| detect_level(line));
                let message = caps
                    .get(pattern.message_group)
                    .map(|m| m.as_str().to_string())?;
                let source = pattern
                    .source_group
                    .and_then(|g| caps.get(g))
                    .map(|m| m.as_str().to_string());

                return Some(LogEntry {
                    timestamp,
                    level,
                    message,
                    source,
                    extra: None,
                });
            }
        }

        // Fallback: never silently drop a line — keep it as a raw entry so
        // `--tail` and `--search` still see it.
        let trimmed = line.trim_end();
        Some(LogEntry {
            timestamp: leading_timestamp()
                .captures(trimmed)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_string()),
            level: detect_level(trimmed),
            message: trimmed.to_string(),
            source: None,
            extra: None,
        })
    }

    pub fn parse_file(&self, path: &Path) -> Result<Vec<LogEntry>> {
        let content = std::fs::read_to_string(path).context("Failed to read log file")?;

        let entries = content
            .lines()
            .filter_map(|line| self.parse_line(line))
            .collect();

        Ok(entries)
    }

    pub fn parse_string(&self, input: &str) -> Vec<LogEntry> {
        input
            .lines()
            .filter_map(|line| self.parse_line(line))
            .collect()
    }

    pub fn add_pattern(
        &mut self,
        pattern: &str,
        timestamp_group: Option<usize>,
        level_group: Option<usize>,
        message_group: usize,
        source_group: Option<usize>,
    ) -> Result<()> {
        let regex = Regex::new(pattern).context("Invalid regex pattern")?;

        self.patterns.push(LogPattern {
            regex,
            timestamp_group,
            level_group,
            message_group,
            source_group,
        });

        Ok(())
    }
}

static LEADING_TIMESTAMP: OnceLock<Regex> = OnceLock::new();
static LEVEL_RULES: OnceLock<Vec<(Regex, LogLevel)>> = OnceLock::new();

fn leading_timestamp() -> &'static Regex {
    LEADING_TIMESTAMP.get_or_init(|| {
        Regex::new(r"^\s*(\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})?)")
            .expect("timestamp regex is valid")
    })
}

fn level_rules() -> &'static [(Regex, LogLevel)] {
    LEVEL_RULES.get_or_init(|| {
        [
            ("FATAL", LogLevel::Fatal),
            ("CRITICAL", LogLevel::Fatal),
            ("ERROR", LogLevel::Error),
            ("PANIC", LogLevel::Error),
            ("WARN", LogLevel::Warn),
            ("DEBUG", LogLevel::Debug),
            ("TRACE", LogLevel::Trace),
        ]
        .iter()
        .map(|(pattern, level)| {
            (
                Regex::new(&format!(r"(?i)\b{}\b", pattern)).expect("level regex is valid"),
                level.clone(),
            )
        })
        .collect()
    })
}

fn detect_level(line: &str) -> LogLevel {
    for (regex, level) in level_rules() {
        if regex.is_match(line) {
            return level.clone();
        }
    }
    LogLevel::Info
}

