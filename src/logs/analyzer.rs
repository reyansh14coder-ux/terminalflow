use std::collections::HashMap;

use super::viewer::{LogEntry, LogLevel};

pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
}

impl LogAnalyzer {
    pub fn new(entries: Vec<LogEntry>) -> Self {
        Self { entries }
    }

    pub fn analyze(&self) -> LogAnalysis {
        let mut analysis = LogAnalysis::default();
        
        for entry in &self.entries {
            match entry.level {
                LogLevel::Trace => analysis.level_counts.trace += 1,
                LogLevel::Debug => analysis.level_counts.debug += 1,
                LogLevel::Info => analysis.level_counts.info += 1,
                LogLevel::Warn => analysis.level_counts.warn += 1,
                LogLevel::Error => analysis.level_counts.error += 1,
                LogLevel::Fatal => analysis.level_counts.fatal += 1,
            }
            
            if let Some(source) = &entry.source {
                *analysis.source_counts.entry(source.clone()).or_insert(0) += 1;
            }
            
            if entry.level == LogLevel::Error || entry.level == LogLevel::Fatal {
                let pattern = extract_error_pattern(&entry.message);
                *analysis.error_patterns.entry(pattern).or_insert(0) += 1;
            }
        }
        
        analysis.error_rate = if analysis.level_counts.total() > 0 {
            (analysis.level_counts.error + analysis.level_counts.fatal) as f64 / analysis.level_counts.total() as f64
        } else {
            0.0
        };
        
        analysis.time_gaps = self.find_time_gaps();
        
        analysis
    }

    fn find_time_gaps(&self) -> Vec<TimeGap> {
        Vec::new()
    }

    pub fn search_errors(&self, pattern: &str) -> Vec<&LogEntry> {
        self.entries.iter()
            .filter(|e| {
                (e.level == LogLevel::Error || e.level == LogLevel::Fatal)
                    && e.message.contains(pattern)
            })
            .collect()
    }

    pub fn get_error_trend(&self, window_size: usize) -> Vec<(usize, usize)> {
        let mut trend = Vec::new();
        
        for i in 0..self.entries.len() {
            let start = if i >= window_size { i - window_size } else { 0 };
            let window: Vec<&LogEntry> = self.entries[start..=i].iter().collect();
            let error_count = window.iter()
                .filter(|e| e.level == LogLevel::Error || e.level == LogLevel::Fatal)
                .count();
            trend.push((i, error_count));
        }
        
        trend
    }

    pub fn find_repeated_errors(&self, threshold: usize) -> Vec<(String, usize)> {
        let mut error_counts: HashMap<String, usize> = HashMap::new();
        
        for entry in &self.entries {
            if entry.level == LogLevel::Error || entry.level == LogLevel::Fatal {
                let pattern = extract_error_pattern(&entry.message);
                *error_counts.entry(pattern).or_insert(0) += 1;
            }
        }
        
        error_counts.into_iter()
            .filter(|(_, count)| *count >= threshold)
            .collect()
    }
}

#[derive(Debug, Default)]
pub struct LogAnalysis {
    pub level_counts: LevelCounts,
    pub source_counts: HashMap<String, usize>,
    pub error_patterns: HashMap<String, usize>,
    pub error_rate: f64,
    pub time_gaps: Vec<TimeGap>,
}

#[derive(Debug, Default)]
pub struct LevelCounts {
    pub trace: usize,
    pub debug: usize,
    pub info: usize,
    pub warn: usize,
    pub error: usize,
    pub fatal: usize,
}

impl LevelCounts {
    pub fn total(&self) -> usize {
        self.trace + self.debug + self.info + self.warn + self.error + self.fatal
    }
}

#[derive(Debug)]
pub struct TimeGap {
    pub start: String,
    pub end: String,
    pub duration_secs: u64,
}

fn extract_error_pattern(message: &str) -> String {
    let pattern: String = message
        .chars()
        .filter(|c| !c.is_numeric())
        .collect();
    
    if pattern.len() > 100 {
        pattern[..100].to_string()
    } else {
        pattern
    }
}
