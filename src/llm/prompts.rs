//! Prompt templates for LLM interactions

/// System prompt for configuration updates
pub const CONFIG_UPDATE_SYSTEM_PROMPT: &str = r#"
You are an expert assistant that helps users modify YAML configuration files for a Markdown to docx converter.

Your task is to update YAML configuration based on natural language descriptions. The configuration controls how Markdown elements are converted to docx format.

Key configuration sections:
- document: Page settings (size, margins, default font)
- styles: Formatting for different elements (headings, paragraphs, code blocks, tables)
- elements: Settings for images, lists, links

When updating configurations:
1. Preserve existing structure and settings unless specifically asked to change them
2. Use appropriate font names, sizes, and formatting options
3. Ensure all YAML syntax is correct
4. Only modify the parts mentioned in the user's request
5. Return ONLY the updated YAML configuration, no explanations

Example font names: "宋体", "微软雅黑", "Times New Roman", "Arial"
Example sizes: Use point values like 12, 14, 16, 18
Example styles: bold, italic, underline
"#;

/// Validation prompt for configuration changes
pub const CONFIG_VALIDATION_PROMPT: &str = r#"
You are a configuration validator. Review the following YAML configuration for a Markdown to docx converter and check for:

1. Valid YAML syntax
2. Reasonable font sizes (8-72 points)
3. Valid font names
4. Proper structure for document, styles, and elements sections
5. Logical formatting combinations

If the configuration is valid, respond with "VALID".
If there are issues, respond with "INVALID: [specific issues]".
"#;

/// Generate a prompt for configuration updates
pub fn create_config_update_prompt(current_config: &str, update_description: &str) -> String {
    format!(
        "Current YAML configuration:\n```yaml\n{}\n```\n\nUser request: {}\n\nProvide the updated YAML configuration:",
        current_config, update_description
    )
}

/// Generate a validation prompt for configuration
pub fn create_config_validation_prompt(config: &str) -> String {
    format!(
        "Please validate this YAML configuration:\n```yaml\n{}\n```",
        config
    )
}

/// Generate examples for common configuration updates
pub fn get_example_prompts() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "将所有二级标题改为14pt微软雅黑加粗",
            "Change all H2 headings to 14pt Microsoft YaHei bold"
        ),
        (
            "设置页面边距为2.5cm",
            "Set page margins to 2.5cm"
        ),
        (
            "代码块使用Consolas字体，背景色为浅灰色",
            "Use Consolas font for code blocks with light gray background"
        ),
        (
            "表格边框设为1pt黑色实线",
            "Set table borders to 1pt black solid lines"
        ),
        (
            "段落行间距设为1.5倍",
            "Set paragraph line spacing to 1.5x"
        ),
    ]
}

/// Extract YAML from LLM response (removes markdown code blocks if present)
pub fn extract_yaml_from_response(response: &str) -> String {
    let trimmed = response.trim();
    
    // Check if response is wrapped in markdown code blocks
    if trimmed.starts_with("```yaml") || trimmed.starts_with("```yml") {
        // Find the end of the code block
        if let Some(end_pos) = trimmed.rfind("```") {
            let start_pos = if trimmed.starts_with("```yaml") { 7 } else { 6 };
            return trimmed[start_pos..end_pos].trim().to_string();
        }
    } else if trimmed.starts_with("```") {
        // Generic code block
        if let Some(end_pos) = trimmed.rfind("```") {
            let lines: Vec<&str> = trimmed.lines().collect();
            if lines.len() > 2 {
                return lines[1..lines.len()-1].join("\n");
            }
        }
    }
    
    // Return as-is if no code block markers found
    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_yaml_from_response_with_yaml_block() {
        let response = "```yaml\nkey: value\nother: data\n```";
        let result = extract_yaml_from_response(response);
        assert_eq!(result, "key: value\nother: data");
    }

    #[test]
    fn test_extract_yaml_from_response_with_yml_block() {
        let response = "```yml\nkey: value\n```";
        let result = extract_yaml_from_response(response);
        assert_eq!(result, "key: value");
    }

    #[test]
    fn test_extract_yaml_from_response_with_generic_block() {
        let response = "```\nkey: value\nother: data\n```";
        let result = extract_yaml_from_response(response);
        assert_eq!(result, "key: value\nother: data");
    }

    #[test]
    fn test_extract_yaml_from_response_plain_text() {
        let response = "key: value\nother: data";
        let result = extract_yaml_from_response(response);
        assert_eq!(result, "key: value\nother: data");
    }

    #[test]
    fn test_create_config_update_prompt() {
        let config = "key: value";
        let description = "change key to newvalue";
        let prompt = create_config_update_prompt(config, description);
        
        assert!(prompt.contains("Current YAML configuration:"));
        assert!(prompt.contains("key: value"));
        assert!(prompt.contains("change key to newvalue"));
    }

    #[test]
    fn test_get_example_prompts() {
        let examples = get_example_prompts();
        assert!(!examples.is_empty());
        assert_eq!(examples.len(), 5);
        
        // Check that each example has both Chinese and English versions
        for (chinese, english) in examples {
            assert!(!chinese.is_empty());
            assert!(!english.is_empty());
        }
    }
}