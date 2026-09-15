#![allow(dead_code)]

use anyhow::{Context, Result};
use std::path::PathBuf;

pub struct CodeGenerator {
    output_dir: PathBuf,
    templates: super::templates::TemplateEngine,
}

impl CodeGenerator {
    pub fn new(output_dir: PathBuf) -> Self {
        Self {
            output_dir,
            templates: super::templates::TemplateEngine::new(),
        }
    }

    pub fn generate_struct(&self, name: &str, fields: &[(String, String)]) -> Result<String> {
        let mut code = format!("pub struct {} {{\n", name);

        for (field_name, field_type) in fields {
            code.push_str(&format!("    pub {}: {},\n", field_name, field_type));
        }

        code.push_str("}\n\n");

        // Generate impl block
        code.push_str(&format!("impl {} {{\n", name));
        code.push_str("    pub fn new(");

        for (i, (field_name, field_type)) in fields.iter().enumerate() {
            if i > 0 {
                code.push_str(", ");
            }
            code.push_str(&format!("{}: {}", field_name, field_type));
        }

        code.push_str(") -> Self {\n");
        code.push_str("        Self {\n");

        for (field_name, _) in fields {
            code.push_str(&format!("            {},\n", field_name));
        }

        code.push_str("        }\n");
        code.push_str("    }\n");
        code.push_str("}\n");

        Ok(code)
    }

    pub fn generate_api_endpoint(
        &self,
        _method: &str,
        _path: &str,
        handler_name: &str,
    ) -> Result<String> {
        let code = format!(
            r#"pub async fn {}(req: Request) -> Response {{
    // TODO: Implement handler
    Response::ok("{{}}")
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[tokio::test]
    async fn test_{}() {{
        // TODO: Add tests
    }}
}}"#,
            handler_name, handler_name
        );

        Ok(code)
    }

    pub fn generate_test(&self, _name: &str, test_cases: &[(&str, &str)]) -> Result<String> {
        let mut code = "#[cfg(test)]\nmod tests {\n    use super::*;\n\n".to_string();

        for (test_name, _test_body) in test_cases {
            code.push_str(&format!(
                "    #[test]\n    fn {}() {{\n        // TODO: Implement test\n    }}\n\n",
                test_name
            ));
        }

        code.push_str("}\n");

        Ok(code)
    }

    pub fn generate_cli_command(
        &self,
        name: &str,
        description: &str,
        args: &[(&str, &str, bool)],
    ) -> Result<String> {
        let mut code = format!(
            r#"/// {}
#[derive(Parser)]
pub struct {} {{
"#,
            description, name
        );

        for (arg_name, arg_help, required) in args {
            if *required {
                code.push_str(&format!(
                    "    /// {}\n    {}: String,\n\n",
                    arg_help, arg_name
                ));
            } else {
                code.push_str(&format!(
                    "    /// {}\n    #[arg(default_value_t)]\n    {}: Option<String>,\n\n",
                    arg_help, arg_name
                ));
            }
        }

        code.push_str("}\n");

        Ok(code)
    }

    pub fn save_code(&self, filename: &str, code: &str) -> Result<()> {
        let path = self.output_dir.join(filename);

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&path, code).context(format!("Failed to write {}", filename))?;

        Ok(())
    }
}
