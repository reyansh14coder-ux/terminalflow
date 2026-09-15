#![allow(dead_code)]

use anyhow::{Context, Result};
use std::collections::HashMap;

pub struct TemplateEngine {
    templates: HashMap<String, String>,
}

impl TemplateEngine {
    pub fn new() -> Self {
        let mut templates = HashMap::new();

        // Rust struct template
        templates.insert(
            "rust_struct".to_string(),
            r#"pub struct {name} {
{fields}
}

impl {name} {
    pub fn new({params}) -> Self {
        Self {
{init_fields}
        }
    }
}"#
            .to_string(),
        );

        // Rust enum template
        templates.insert(
            "rust_enum".to_string(),
            r#"pub enum {name} {
{variants}
}

impl {name} {
    pub fn as_str(&self) -> &'static str {
        match self {
{match_arms}
        }
    }
}"#
            .to_string(),
        );

        // API handler template
        templates.insert(
            "api_handler".to_string(),
            r#"pub async fn {name}(req: Request) -> Response {
    let body = req.body;
    
    // TODO: Implement logic
    
    Response::ok("{{}}")
}"#
            .to_string(),
        );

        // Test template
        templates.insert(
            "test".to_string(),
            r#"#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_{name}() {
        // TODO: Implement test
    }
}"#
            .to_string(),
        );

        // CLI command template
        templates.insert(
            "cli_command".to_string(),
            r#"/// {description}
#[derive(Parser)]
pub struct {name} {{
{args}
}}

impl {name} {{
    pub fn execute(&self) -> Result<()> {{
        // TODO: Implement command
        Ok(())
    }}
}}"#
            .to_string(),
        );

        Self { templates }
    }

    pub fn render(
        &self,
        template_name: &str,
        variables: &HashMap<String, String>,
    ) -> Result<String> {
        let template = self
            .templates
            .get(template_name)
            .context(format!("Template '{}' not found", template_name))?;

        let mut result = template.clone();

        for (key, value) in variables {
            result = result.replace(&format!("{{{}}}", key), value);
        }

        Ok(result)
    }

    pub fn add_template(&mut self, name: &str, template: &str) {
        self.templates
            .insert(name.to_string(), template.to_string());
    }

    pub fn list_templates(&self) -> Vec<&str> {
        self.templates.keys().map(|s| s.as_str()).collect()
    }
}
