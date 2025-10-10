//! Prompt templates for LLM interactions

/// System prompt for configuration updates
pub const CONFIG_UPDATE_SYSTEM_PROMPT: &str = r#"
You are an expert assistant that helps users modify YAML configuration files for a Markdown to docx converter.

Your task is to update YAML configuration based on natural language descriptions. The configuration controls how Markdown elements are converted to docx format.

Key configuration sections:
- document: Page settings (size, margins, default font)
- styles: Formatting for different elements (headings, paragraphs, code blocks, tables)
- elements: Settings for images, lists, links

Code block configuration options:
- preserve_line_breaks: true/false - Whether to preserve original line breaks in code blocks
- line_spacing: Number - Line spacing within code blocks (1.0 = single spacing)
- paragraph_spacing: Number - Spacing between code block paragraphs in points

Heading numbering configuration:
- Add "numbering" field to heading styles to enable automatic numbering
- Numbering formats use placeholders: %1 for level 1, %2 for level 2, etc.
- Common formats: "%1." (1., 2., 3.), "%1.%2." (1.1., 1.2., 2.1.), "%1.%2.%3" (1.1.1, 1.1.2)
- Custom separators allowed: "%1-%2-%3" (1-1-1), "Chapter %1, Section %2:" (Chapter 1, Section 1:)
- Levels must be sequential starting from 1 (valid: %1.%2.%3, invalid: %1.%3.)

When updating configurations:
1. Preserve existing structure and settings unless specifically asked to change them
2. Use appropriate font names, sizes, and formatting options
3. Ensure all YAML syntax is correct
4. Only modify the parts mentioned in the user's request
5. For numbering: validate format strings follow the placeholder rules
6. Return ONLY the updated YAML configuration, no explanations

Example font names: "宋体", "微软雅黑", "Times New Roman", "Arial"
Example sizes: Use point values like 12, 14, 16, 18
Example styles: bold, italic, underline
Example numbering formats: "%1.", "%1.%2.", "%1.%2.%3", "%1-%2-%3", "Chapter %1, Section %2:"
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
            "Change all H2 headings to 14pt Microsoft YaHei bold",
        ),
        ("设置页面边距为2.5cm", "Set page margins to 2.5cm"),
        (
            "代码块使用Consolas字体，背景色为浅灰色",
            "Use Consolas font for code blocks with light gray background",
        ),
        (
            "代码块保留原始换行符，行间距设为1.2倍",
            "Preserve line breaks in code blocks with 1.2x line spacing",
        ),
        (
            "关闭代码块换行符保留功能",
            "Disable line break preservation in code blocks",
        ),
        (
            "表格边框设为1pt黑色实线",
            "Set table borders to 1pt black solid lines",
        ),
        ("段落行间距设为1.5倍", "Set paragraph line spacing to 1.5x"),
        (
            "为一级标题添加编号，格式为1.",
            "Add numbering to H1 headings with format 1.",
        ),
        (
            "为二级标题添加编号，格式为1.1.",
            "Add numbering to H2 headings with format 1.1.",
        ),
        (
            "为三级标题添加编号，格式为1.1.1",
            "Add numbering to H3 headings with format 1.1.1",
        ),
        ("取消二级标题的编号", "Remove numbering from H2 headings"),
        (
            "设置标题编号格式为章节样式：第1章，第1.1节",
            "Set heading numbering to chapter style: Chapter 1, Section 1.1",
        ),
    ]
}

/// Parse natural language numbering requests and convert to format strings
///
/// This function analyzes natural language descriptions and extracts numbering
/// configuration information including heading level and format pattern.
pub fn parse_numbering_request(description: &str) -> Option<NumberingRequest> {
    let description_lower = description.to_lowercase();

    // Extract heading level
    let level = extract_heading_level(&description_lower)?;

    // Determine the action (add, remove, modify)
    let action = if description_lower.contains("取消")
        || description_lower.contains("删除")
        || description_lower.contains("remove")
        || description_lower.contains("delete")
    {
        NumberingAction::Remove
    } else if description_lower.contains("添加")
        || description_lower.contains("设置")
        || description_lower.contains("add")
        || description_lower.contains("set")
    {
        // Extract format pattern
        let format = extract_numbering_format(&description_lower, level)?;
        NumberingAction::Add(format)
    } else {
        return None;
    };

    Some(NumberingRequest { level, action })
}

/// Extract heading level from natural language description
fn extract_heading_level(description: &str) -> Option<u8> {
    // Chinese patterns
    if description.contains("一级标题") || description.contains("h1") {
        return Some(1);
    }
    if description.contains("二级标题") || description.contains("h2") {
        return Some(2);
    }
    if description.contains("三级标题") || description.contains("h3") {
        return Some(3);
    }
    if description.contains("四级标题") || description.contains("h4") {
        return Some(4);
    }
    if description.contains("五级标题") || description.contains("h5") {
        return Some(5);
    }
    if description.contains("六级标题") || description.contains("h6") {
        return Some(6);
    }

    // English patterns
    if description.contains("level 1") || description.contains("first level") {
        return Some(1);
    }
    if description.contains("level 2") || description.contains("second level") {
        return Some(2);
    }
    if description.contains("level 3") || description.contains("third level") {
        return Some(3);
    }
    if description.contains("level 4") || description.contains("fourth level") {
        return Some(4);
    }
    if description.contains("level 5") || description.contains("fifth level") {
        return Some(5);
    }
    if description.contains("level 6") || description.contains("sixth level") {
        return Some(6);
    }

    None
}

/// Extract numbering format from natural language description
fn extract_numbering_format(description: &str, level: u8) -> Option<String> {
    // Look for explicit format patterns
    if description.contains("1.1.1") || description.contains("三级编号") {
        return Some("%1.%2.%3".to_string());
    }
    if description.contains("1.1.") || description.contains("二级编号") {
        return Some("%1.%2.".to_string());
    }
    if description.contains("1.") || description.contains("一级编号") {
        return Some("%1.".to_string());
    }

    // Look for chapter/section patterns
    if description.contains("章节") || (description.contains("章") && description.contains("节"))
    {
        if level == 1 {
            return Some("第%1章".to_string());
        } else if level == 2 {
            return Some("第%1.%2节".to_string());
        }
    }

    if description.contains("chapter") {
        if level == 1 {
            return Some("Chapter %1".to_string());
        } else if level == 2 {
            return Some("Chapter %1, Section %2".to_string());
        }
    }

    // Default format based on level
    match level {
        1 => Some("%1.".to_string()),
        2 => Some("%1.%2.".to_string()),
        3 => Some("%1.%2.%3".to_string()),
        4 => Some("%1.%2.%3.%4".to_string()),
        5 => Some("%1.%2.%3.%4.%5".to_string()),
        6 => Some("%1.%2.%3.%4.%5.%6".to_string()),
        _ => None,
    }
}

/// Generate a specialized prompt for numbering configuration updates
pub fn create_numbering_update_prompt(
    current_config: &str,
    numbering_request: &NumberingRequest,
) -> String {
    let action_description = match &numbering_request.action {
        NumberingAction::Add(format) => {
            format!(
                "Add numbering to H{} headings with format '{}'",
                numbering_request.level, format
            )
        }
        NumberingAction::Remove => {
            format!(
                "Remove numbering from H{} headings",
                numbering_request.level
            )
        }
    };

    format!(
        "Current YAML configuration:\n```yaml\n{}\n```\n\nNumbering update: {}\n\nProvide the updated YAML configuration with the numbering field properly set:",
        current_config, action_description
    )
}

/// Represents a parsed numbering request from natural language
#[derive(Debug, Clone, PartialEq)]
pub struct NumberingRequest {
    pub level: u8,
    pub action: NumberingAction,
}

/// Actions that can be performed on numbering configuration
#[derive(Debug, Clone, PartialEq)]
pub enum NumberingAction {
    Add(String), // Add numbering with the specified format
    Remove,      // Remove numbering
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
        if let Some(_end_pos) = trimmed.rfind("```") {
            let lines: Vec<&str> = trimmed.lines().collect();
            if lines.len() > 2 {
                return lines[1..lines.len() - 1].join("\n");
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
        assert_eq!(examples.len(), 12); // Updated count to include numbering and code block examples

        // Check that each example has both Chinese and English versions
        for (chinese, english) in &examples {
            assert!(!chinese.is_empty());
            assert!(!english.is_empty());
        }

        // Verify numbering examples are included
        let chinese_examples: Vec<&str> = examples.iter().map(|(c, _)| *c).collect();
        assert!(chinese_examples.iter().any(|e| e.contains("编号")));
        assert!(chinese_examples.iter().any(|e| e.contains("一级标题")));
        assert!(chinese_examples.iter().any(|e| e.contains("取消")));
    }

    #[test]
    fn test_parse_numbering_request_chinese_add() {
        let request = parse_numbering_request("为一级标题添加编号，格式为1.").unwrap();
        assert_eq!(request.level, 1);
        assert_eq!(request.action, NumberingAction::Add("%1.".to_string()));

        let request = parse_numbering_request("为二级标题添加编号，格式为1.1.").unwrap();
        assert_eq!(request.level, 2);
        assert_eq!(request.action, NumberingAction::Add("%1.%2.".to_string()));

        let request = parse_numbering_request("为三级标题添加编号，格式为1.1.1").unwrap();
        assert_eq!(request.level, 3);
        assert_eq!(request.action, NumberingAction::Add("%1.%2.%3".to_string()));
    }

    #[test]
    fn test_parse_numbering_request_english_add() {
        let request =
            parse_numbering_request("Add numbering to H1 headings with format 1.").unwrap();
        assert_eq!(request.level, 1);
        assert_eq!(request.action, NumberingAction::Add("%1.".to_string()));

        let request = parse_numbering_request("Set level 2 headings to format 1.1.").unwrap();
        assert_eq!(request.level, 2);
        assert_eq!(request.action, NumberingAction::Add("%1.%2.".to_string()));
    }

    #[test]
    fn test_parse_numbering_request_remove() {
        let request = parse_numbering_request("取消二级标题的编号").unwrap();
        assert_eq!(request.level, 2);
        assert_eq!(request.action, NumberingAction::Remove);

        let request = parse_numbering_request("Remove numbering from H3 headings").unwrap();
        assert_eq!(request.level, 3);
        assert_eq!(request.action, NumberingAction::Remove);

        let request = parse_numbering_request("删除一级标题编号").unwrap();
        assert_eq!(request.level, 1);
        assert_eq!(request.action, NumberingAction::Remove);
    }

    #[test]
    fn test_parse_numbering_request_chapter_style() {
        let request = parse_numbering_request("设置一级标题为章节格式").unwrap();
        assert_eq!(request.level, 1);
        assert_eq!(request.action, NumberingAction::Add("第%1章".to_string()));

        let request = parse_numbering_request("Set H1 to chapter style").unwrap();
        assert_eq!(request.level, 1);
        assert_eq!(
            request.action,
            NumberingAction::Add("Chapter %1".to_string())
        );
    }

    #[test]
    fn test_parse_numbering_request_invalid() {
        assert!(parse_numbering_request("这不是编号相关的请求").is_none());
        assert!(parse_numbering_request("change font size").is_none());
        assert!(parse_numbering_request("").is_none());
    }

    #[test]
    fn test_extract_heading_level_chinese() {
        assert_eq!(extract_heading_level("一级标题"), Some(1));
        assert_eq!(extract_heading_level("二级标题"), Some(2));
        assert_eq!(extract_heading_level("三级标题"), Some(3));
        assert_eq!(extract_heading_level("四级标题"), Some(4));
        assert_eq!(extract_heading_level("五级标题"), Some(5));
        assert_eq!(extract_heading_level("六级标题"), Some(6));
    }

    #[test]
    fn test_extract_heading_level_english() {
        assert_eq!(extract_heading_level("h1"), Some(1));
        assert_eq!(extract_heading_level("h2 headings"), Some(2));
        assert_eq!(extract_heading_level("level 3"), Some(3));
        assert_eq!(extract_heading_level("first level"), Some(1));
        assert_eq!(extract_heading_level("second level"), Some(2));
        assert_eq!(extract_heading_level("third level"), Some(3));
    }

    #[test]
    fn test_extract_heading_level_invalid() {
        assert_eq!(extract_heading_level("paragraph"), None);
        assert_eq!(extract_heading_level(""), None);
        assert_eq!(extract_heading_level("七级标题"), None);
    }

    #[test]
    fn test_extract_numbering_format_explicit() {
        assert_eq!(
            extract_numbering_format("格式为1.", 1),
            Some("%1.".to_string())
        );
        assert_eq!(
            extract_numbering_format("格式为1.1.", 2),
            Some("%1.%2.".to_string())
        );
        assert_eq!(
            extract_numbering_format("格式为1.1.1", 3),
            Some("%1.%2.%3".to_string())
        );
    }

    #[test]
    fn test_extract_numbering_format_default() {
        assert_eq!(
            extract_numbering_format("添加编号", 1),
            Some("%1.".to_string())
        );
        assert_eq!(
            extract_numbering_format("add numbering", 2),
            Some("%1.%2.".to_string())
        );
        assert_eq!(
            extract_numbering_format("set numbering", 3),
            Some("%1.%2.%3".to_string())
        );
    }

    #[test]
    fn test_extract_numbering_format_chapter_style() {
        assert_eq!(
            extract_numbering_format("章节格式", 1),
            Some("第%1章".to_string())
        );
        assert_eq!(
            extract_numbering_format("章节格式", 2),
            Some("第%1.%2节".to_string())
        );
        assert_eq!(
            extract_numbering_format("chapter style", 1),
            Some("Chapter %1".to_string())
        );
        assert_eq!(
            extract_numbering_format("chapter and section", 2),
            Some("Chapter %1, Section %2".to_string())
        );
    }

    #[test]
    fn test_create_numbering_update_prompt() {
        let config = "styles:\n  headings:\n    1:\n      font:\n        family: Arial";
        let request = NumberingRequest {
            level: 1,
            action: NumberingAction::Add("%1.".to_string()),
        };

        let prompt = create_numbering_update_prompt(config, &request);

        assert!(prompt.contains("Current YAML configuration:"));
        assert!(prompt.contains("Add numbering to H1 headings with format '%1.'"));
        assert!(prompt.contains("numbering field properly set"));
    }

    #[test]
    fn test_create_numbering_update_prompt_remove() {
        let config = "styles:\n  headings:\n    2:\n      numbering: '%1.%2.'";
        let request = NumberingRequest {
            level: 2,
            action: NumberingAction::Remove,
        };

        let prompt = create_numbering_update_prompt(config, &request);

        assert!(prompt.contains("Remove numbering from H2 headings"));
    }

    #[test]
    fn test_numbering_request_equality() {
        let req1 = NumberingRequest {
            level: 1,
            action: NumberingAction::Add("%1.".to_string()),
        };
        let req2 = NumberingRequest {
            level: 1,
            action: NumberingAction::Add("%1.".to_string()),
        };
        let req3 = NumberingRequest {
            level: 1,
            action: NumberingAction::Remove,
        };

        assert_eq!(req1, req2);
        assert_ne!(req1, req3);
    }

    #[test]
    fn test_numbering_action_equality() {
        let action1 = NumberingAction::Add("%1.".to_string());
        let action2 = NumberingAction::Add("%1.".to_string());
        let action3 = NumberingAction::Add("%1.%2.".to_string());
        let action4 = NumberingAction::Remove;

        assert_eq!(action1, action2);
        assert_ne!(action1, action3);
        assert_ne!(action1, action4);
        assert_eq!(NumberingAction::Remove, NumberingAction::Remove);
    }

    #[test]
    fn test_complex_numbering_descriptions() {
        // Test complex Chinese descriptions
        let request = parse_numbering_request("请为所有的一级标题添加编号，使用1.的格式").unwrap();
        assert_eq!(request.level, 1);
        assert_eq!(request.action, NumberingAction::Add("%1.".to_string()));

        // Test complex English descriptions
        let request = parse_numbering_request(
            "Please set the second level headings to use 1.1. numbering format",
        )
        .unwrap();
        assert_eq!(request.level, 2);
        assert_eq!(request.action, NumberingAction::Add("%1.%2.".to_string()));

        // Test mixed language (should still work)
        let request = parse_numbering_request("Set 三级标题 numbering to 1.1.1 format").unwrap();
        assert_eq!(request.level, 3);
        assert_eq!(request.action, NumberingAction::Add("%1.%2.%3".to_string()));
    }

    #[test]
    fn test_edge_cases() {
        // Test case sensitivity
        let request = parse_numbering_request("ADD NUMBERING TO H1 HEADINGS").unwrap();
        assert_eq!(request.level, 1);

        // Test with extra whitespace
        let request = parse_numbering_request("  为  一级标题  添加  编号  ").unwrap();
        assert_eq!(request.level, 1);

        // Test partial matches
        assert!(parse_numbering_request("标题样式").is_none()); // Contains "标题" but not specific level
        assert!(parse_numbering_request("heading style").is_none()); // Contains "heading" but not specific level
    }

    #[test]
    fn test_all_heading_levels() {
        for level in 1..=6 {
            let description = format!(
                "为{}级标题添加编号",
                match level {
                    1 => "一",
                    2 => "二",
                    3 => "三",
                    4 => "四",
                    5 => "五",
                    6 => "六",
                    _ => unreachable!(),
                }
            );

            let request = parse_numbering_request(&description).unwrap();
            assert_eq!(request.level, level);

            // Test English version
            let description = format!("Add numbering to H{} headings", level);
            let request = parse_numbering_request(&description).unwrap();
            assert_eq!(request.level, level);
        }
    }
}
