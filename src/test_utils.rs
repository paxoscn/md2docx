//! Test utilities and mock objects for unit testing

use crate::config::{ConversionConfig, DocumentConfig, StyleConfig, ElementConfig, PageSize, Margins, FontConfig, HeadingStyle, ParagraphStyle, CodeBlockStyle, TableStyle, ImageConfig, ListConfig, LinkConfig};
use crate::markdown::ast::{MarkdownDocument, MarkdownElement, InlineElement, ListItem};
use std::collections::HashMap;

/// Create a minimal valid configuration for testing
pub fn create_test_config() -> ConversionConfig {
    ConversionConfig {
        document: DocumentConfig {
            page_size: PageSize {
                width: 595.0,
                height: 842.0,
            },
            margins: Margins {
                top: 72.0,
                bottom: 72.0,
                left: 72.0,
                right: 72.0,
            },
            default_font: FontConfig {
                family: "Times New Roman".to_string(),
                size: 12.0,
                bold: false,
                italic: false,
            },
        },
        styles: StyleConfig {
            headings: {
                let mut headings = HashMap::new();
                headings.insert(1, HeadingStyle {
                    font: FontConfig {
                        family: "Arial".to_string(),
                        size: 18.0,
                        bold: true,
                        italic: false,
                    },
                    spacing_before: 12.0,
                    spacing_after: 6.0,
                    alignment: None,
                    numbering: None,
                });
                headings.insert(2, HeadingStyle {
                    font: FontConfig {
                        family: "Arial".to_string(),
                        size: 16.0,
                        bold: true,
                        italic: false,
                    },
                    spacing_before: 10.0,
                    spacing_after: 5.0,
                    alignment: None,
                    numbering: None,
                });
                headings.insert(3, HeadingStyle {
                    font: FontConfig {
                        family: "Arial".to_string(),
                        size: 14.0,
                        bold: true,
                        italic: false,
                    },
                    spacing_before: 10.0,
                    spacing_after: 5.0,
                    alignment: None,
                    numbering: None,
                });
                headings
            },
            paragraph: ParagraphStyle {
                font: FontConfig {
                    family: "Times New Roman".to_string(),
                    size: 12.0,
                    bold: false,
                    italic: false,
                },
                line_spacing: 1.15,
                spacing_after: 6.0,
            },
            code_block: CodeBlockStyle {
                font: FontConfig {
                    family: "Courier New".to_string(),
                    size: 10.0,
                    bold: false,
                    italic: false,
                },
                background_color: Some("#f5f5f5".to_string()),
                border_width: 1.0,
                preserve_line_breaks: true,
                line_spacing: 1.0,
                paragraph_spacing: 6.0,
            },
            table: TableStyle {
                header_font: FontConfig {
                    family: "Times New Roman".to_string(),
                    size: 12.0,
                    bold: true,
                    italic: false,
                },
                cell_font: FontConfig {
                    family: "Times New Roman".to_string(),
                    size: 11.0,
                    bold: false,
                    italic: false,
                },
                border_width: 1.0,
            },
        },
        elements: ElementConfig {
            image: ImageConfig {
                max_width: 600.0,
                max_height: 400.0,
            },
            list: ListConfig {
                indent: 20.0,
                spacing: 3.0,
            },
            link: LinkConfig {
                color: "#0066cc".to_string(),
                underline: true,
            },
        },
    }
}

/// Create a test markdown document with various elements
pub fn create_test_document() -> MarkdownDocument {
    MarkdownDocument {
        elements: vec![
            MarkdownElement::Heading {
                level: 1,
                text: "Main Title".to_string(),
            },
            MarkdownElement::Paragraph {
                content: vec![
                    InlineElement::Text("This is a paragraph with ".to_string()),
                    InlineElement::Bold("bold".to_string()),
                    InlineElement::Text(" and ".to_string()),
                    InlineElement::Italic("italic".to_string()),
                    InlineElement::Text(" text.".to_string()),
                ],
            },
            MarkdownElement::Heading {
                level: 2,
                text: "Subtitle".to_string(),
            },
            MarkdownElement::CodeBlock {
                language: Some("rust".to_string()),
                code: "fn main() {\n    println!(\"Hello, world!\");\n}".to_string(),
                processed: None,
            },
            MarkdownElement::List {
                ordered: false,
                items: vec![
                    ListItem {
                        content: vec![InlineElement::Text("First item".to_string())],
                        sub_items: vec![],
                    },
                    ListItem {
                        content: vec![InlineElement::Text("Second item".to_string())],
                        sub_items: vec![],
                    },
                ],
            },
            MarkdownElement::Table {
                headers: vec!["Name".to_string(), "Age".to_string()],
                rows: vec![
                    vec!["Alice".to_string(), "30".to_string()],
                    vec!["Bob".to_string(), "25".to_string()],
                ],
            },
            MarkdownElement::Image {
                alt_text: "Test Image".to_string(),
                url: "https://example.com/image.jpg".to_string(),
                title: None,
            },
            MarkdownElement::HorizontalRule,
        ],
    }
}

/// Create a simple test markdown document
pub fn create_simple_document() -> MarkdownDocument {
    MarkdownDocument {
        elements: vec![
            MarkdownElement::Heading {
                level: 1,
                text: "Simple Title".to_string(),
            },
            MarkdownElement::Paragraph {
                content: vec![
                    InlineElement::Text("Simple paragraph.".to_string()),
                ],
            },
        ],
    }
}

/// Create test markdown content as string
pub fn create_test_markdown() -> String {
    r#"# Main Title

This is a paragraph with **bold** and *italic* text.

## Subtitle

Here's a code block:

```rust
fn main() {
    println!("Hello, world!");
}
```

### List Example

- Item 1
- Item 2
- Item 3

### Table Example

| Name | Age |
|------|-----|
| Alice | 30 |
| Bob | 25 |

![Test Image](https://example.com/image.jpg)

---

That's all!"#.to_string()
}

/// Create invalid configuration for testing error cases
pub fn create_invalid_config() -> ConversionConfig {
    let mut config = create_test_config();
    config.document.page_size.width = -100.0; // Invalid width
    config
}

/// Mock LLM response for testing
pub fn mock_llm_response() -> String {
    r#"```yaml
document:
  page_size:
    width: 595.0
    height: 842.0
  margins:
    top: 72.0
    bottom: 72.0
    left: 72.0
    right: 72.0
  default_font:
    family: "Arial"
    size: 14.0
    bold: false
    italic: false
```"#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_config() {
        let config = create_test_config();
        assert!(config.validate().is_ok());
        assert_eq!(config.document.default_font.family, "Times New Roman");
        assert_eq!(config.document.default_font.size, 12.0);
    }

    #[test]
    fn test_create_test_document() {
        let doc = create_test_document();
        assert_eq!(doc.elements.len(), 8);
        
        // Check first element is heading
        match &doc.elements[0] {
            MarkdownElement::Heading { level, text } => {
                assert_eq!(*level, 1);
                assert_eq!(text, "Main Title");
            },
            _ => panic!("Expected heading element"),
        }
    }

    #[test]
    fn test_create_simple_document() {
        let doc = create_simple_document();
        assert_eq!(doc.elements.len(), 2);
    }

    #[test]
    fn test_create_invalid_config() {
        let config = create_invalid_config();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_mock_llm_response() {
        let response = mock_llm_response();
        assert!(response.contains("yaml"));
        assert!(response.contains("Arial"));
    }
}