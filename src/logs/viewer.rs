use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: Option<String>,
    pub level: LogLevel,
    pub message: String,
    pub source: Option<String>,
    pub extra: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "TRACE"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Fatal => write!(f, "FATAL"),
        }
    }
}

impl LogLevel {
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "TRACE" | "TRC" => LogLevel::Trace,
            "DEBUG" | "DBG" => LogLevel::Debug,
            "INFO" | "INF" => LogLevel::Info,
            "WARN" | "WARNING" => LogLevel::Warn,
            "ERROR" | "ERR" => LogLevel::Error,
            "FATAL" | "FTL" | "CRITICAL" | "CRIT" => LogLevel::Fatal,
            _ => LogLevel::Info,
        }
    }
}

pub struct LogViewer {
    entries: VecDeque<LogEntry>,
    max_entries: usize,
    filter_level: Option<LogLevel>,
    filter_source: Option<String>,
    filter_text: Option<String>,
}

impl LogViewer {
    pub fn new() -> Self {
        Self {
            entries: VecDeque::new(),
            max_entries: 10000,
            filter_level: None,
            filter_source: None,
            filter_text: None,
        }
    }

    pub fn with_capacity(mut self, max_entries: usize) -> Self {
        self.max_entries = max_entries;
        self
    }

    pub fn filter_level(mut self, level: LogLevel) -> Self {
        self.filter_level = Some(level);
        self
    }

    pub fn filter_source(mut self, source: &str) -> Self {
        self.filter_source = Some(source.to_string());
        self
    }

    pub fn filter_text(mut self, text: &str) -> Self {
        self.filter_text = Some(text.to_string());
        self
    }

    pub fn add_entry(&mut self, entry: LogEntry) {
        if self.should_include(&entry) {
            if self.entries.len() >= self.max_entries {
                self.entries.pop_front();
            }
            self.entries.push_back(entry);
        }
    }

    fn should_include(&self, entry: &LogEntry) -> bool {
        if let Some(ref level) = self.filter_level {
            if entry.level != *level {
                return false;
            }
        }

        if let Some(ref source) = self.filter_source {
            if entry.source.as_deref() != Some(source) {
                return false;
            }
        }

        if let Some(ref text) = self.filter_text {
            if !entry.message.contains(text) {
                return false;
            }
        }

        true
    }

    pub fn get_entries(&self) -> impl Iterator<Item = &LogEntry> {
        self.entries.iter()
    }

    pub fn get_entries_by_level(&self, level: LogLevel) -> impl Iterator<Item = &LogEntry> {
        self.entries.iter().filter(move |e| e.level == level)
    }

    pub fn search(&self, query: &str) -> Vec<&LogEntry> {
        self.entries.iter()
            .filter(|e| e.message.contains(query) || e.source.as_deref().unwrap_or("").contains(query))
            .collect()
    }

    pub fn tail(&self, n: usize) -> Vec<&LogEntry> {
        self.entries.iter().rev().take(n).collect()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn stats(&self) -> LogStats {
        let mut stats = LogStats::default();
        
        for entry in &self.entries {
            match entry.level {
                LogLevel::Trace => stats.trace += 1,
                LogLevel::Debug => stats.debug += 1,
                LogLevel::Info => stats.info += 1,
                LogLevel::Warn => stats.warn += 1,
                LogLevel::Error => stats.error += 1,
                LogLevel::Fatal => stats.fatal += 1,
            }
            stats.total += 1;
        }
        
        stats
    }
}

#[derive(Debug, Default)]
pub struct LogStats {
    pub total: usize,
    pub trace: usize,
    pub debug: usize,
    pub info: usize,
    pub warn: usize,
    pub error: usize,
    pub fatal: usize,
}
