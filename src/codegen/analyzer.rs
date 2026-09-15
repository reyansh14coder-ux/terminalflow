#![allow(dead_code)]

use anyhow::Result;
use std::path::Path;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct CodeAnalysis {
    pub total_lines: usize,
    pub blank_lines: usize,
    pub comment_lines: usize,
    pub code_lines: usize,
    pub functions: Vec<FunctionInfo>,
    pub structs: Vec<StructInfo>,
    pub enums: Vec<EnumInfo>,
    pub imports: Vec<String>,
    pub complexity: ComplexityMetrics,
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub line_start: usize,
    pub line_end: usize,
    pub parameters: usize,
    pub is_public: bool,
    pub doc_comment: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StructInfo {
    pub name: String,
    pub line: usize,
    pub fields: usize,
    pub is_public: bool,
}

#[derive(Debug, Clone)]
pub struct EnumInfo {
    pub name: String,
    pub line: usize,
    pub variants: usize,
    pub is_public: bool,
}

#[derive(Debug, Clone)]
pub struct ComplexityMetrics {
    pub cyclomatic_complexity: usize,
    pub cognitive_complexity: usize,
    pub max_nesting_depth: usize,
}

pub struct CodeAnalyzer {
    function_regex: Regex,
    struct_regex: Regex,
    enum_regex: Regex,
    import_regex: Regex,
}

impl CodeAnalyzer {
    pub fn new() -> Self {
        Self {
            function_regex: Regex::new(r"(?:pub\s+)?(?:async\s+)?fn\s+(\w+)").unwrap(),
            struct_regex: Regex::new(r"(?:pub\s+)?struct\s+(\w+)").unwrap(),
            enum_regex: Regex::new(r"(?:pub\s+)?enum\s+(\w+)").unwrap(),
            import_regex: Regex::new(r"use\s+([\w:]+(?:::\w+)*)").unwrap(),
        }
    }

    pub fn analyze_file(&self, path: &Path) -> Result<CodeAnalysis> {
        let content = std::fs::read_to_string(path)?;
        self.analyze_code(&content)
    }

    pub fn analyze_code(&self, code: &str) -> Result<CodeAnalysis> {
        let lines: Vec<&str> = code.lines().collect();
        let total_lines = lines.len();
        
        let mut blank_lines = 0;
        let mut comment_lines = 0;
        let mut code_lines = 0;
        let mut functions = Vec::new();
        let mut structs = Vec::new();
        let mut enums = Vec::new();
        let mut imports = Vec::new();
        
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            if trimmed.is_empty() {
                blank_lines += 1;
            } else if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
                comment_lines += 1;
            } else {
                code_lines += 1;
            }
            
            // Find functions
            if let Some(caps) = self.function_regex.captures(line) {
                functions.push(FunctionInfo {
                    name: caps[1].to_string(),
                    line_start: i + 1,
                    line_end: i + 1,
                    parameters: line.matches(',').count() + if line.contains('(') && !line.contains(",") && line.contains(')') { 0 } else { 0 },
                    is_public: line.contains("pub"),
                    doc_comment: None,
                });
            }
            
            // Find structs
            if let Some(caps) = self.struct_regex.captures(line) {
                structs.push(StructInfo {
                    name: caps[1].to_string(),
                    line: i + 1,
                    fields: 0,
                    is_public: line.contains("pub"),
                });
            }
            
            // Find enums
            if let Some(caps) = self.enum_regex.captures(line) {
                enums.push(EnumInfo {
                    name: caps[1].to_string(),
                    line: i + 1,
                    variants: 0,
                    is_public: line.contains("pub"),
                });
            }
            
            // Find imports
            if let Some(caps) = self.import_regex.captures(line) {
                imports.push(caps[1].to_string());
            }
        }
        
        Ok(CodeAnalysis {
            total_lines,
            blank_lines,
            comment_lines,
            code_lines,
            functions,
            structs,
            enums,
            imports,
            complexity: ComplexityMetrics {
                cyclomatic_complexity: 0,
                cognitive_complexity: 0,
                max_nesting_depth: 0,
            },
        })
    }
}
