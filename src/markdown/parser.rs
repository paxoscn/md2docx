//! Markdown parser using pulldown-cmark

use crate::error::ConversionError;
use crate::markdown::ast::{MarkdownDocument, MarkdownElement, InlineElement, ListItem};
use pulldown_cmark::{Event, Parser, Tag, CodeBlockKind, HeadingLevel};

/// Markdown parser that converts Markdown text to AST
pub struct MarkdownParser {
    /// Parser options for pulldown-cmark
    options: pulldown_cmark::Options,
}

impl MarkdownParser {
    /// Create a new Markdown parser with default options
    pub fn new() -> Self {
        let mut options = pulldown_cmark::Options::empty();
        options.insert(pulldown_cmark::Options::ENABLE_TABLES);
        options.insert(pulldown_cmark::Options::ENABLE_FOOTNOTES);
        options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
        options.insert(pulldown_cmark::Options::ENABLE_TASKLISTS);
        
        Self { options }
    }

    /// Create a new Markdown parser with custom options
    pub fn with_options(options: pulldown_cmark::Options) -> Self {
        Self { options }
    }

    /// Parse Markdown string into document AST
    pub fn parse(&self, markdown: &str) -> Result<MarkdownDocument, ConversionError> {
        let parser = Parser::new_ext(markdown, self.options);
        let mut document = MarkdownDocument::new();
        let events: Vec<Event> = parser.collect();
        
        let mut i = 0;
        while i < events.len() {
            // println!("event = {:?}, i = {}, len = {}", &events[i], i, events.len());
            match &events[i] {
                Event::Start(Tag::Heading(level, _, _)) => {
                    i += 1; // Skip start event
                    let text = self.collect_text_until_end(&events, &mut i, "Heading")?;
                    document.add_element(MarkdownElement::Heading {
                        level: heading_level_to_u8(*level),
                        text,
                    });
                },
                Event::Start(Tag::Paragraph) => {
                    i += 1; // Skip start event
                    let (content, standalone_image) = self.collect_paragraph_content(&events, &mut i)?;
                    
                    // If paragraph contains only an image, treat it as a standalone image
                    if let Some(image) = standalone_image {
                        document.add_element(image);
                    } else if !content.is_empty() {
                        document.add_element(MarkdownElement::Paragraph { content });
                    }
                },
                Event::Start(Tag::CodeBlock(kind)) => {
                    let language = match kind {
                        CodeBlockKind::Fenced(lang) => {
                            if lang.is_empty() {
                                None
                            } else {
                                Some(lang.to_string())
                            }
                        },
                        CodeBlockKind::Indented => None,
                    };
                    i += 1; // Skip start event
                    let code = self.collect_text_until_end(&events, &mut i, "CodeBlock")?;
                    document.add_element(MarkdownElement::CodeBlock { language, code });
                },
                Event::Start(Tag::List(first_item_number)) => {
                    let ordered = first_item_number.is_some();
                    i += 1; // Skip start event
                    let items = self.collect_list_items(&events, &mut i)?;
                    document.add_element(MarkdownElement::List { ordered, items });
                },
                Event::Start(Tag::Table(_)) => {
                    i += 1; // Skip start event
                    let (headers, rows) = self.collect_table_content(&events, &mut i)?;
                    document.add_element(MarkdownElement::Table { headers, rows });
                },

                Event::Rule => {
                    document.add_element(MarkdownElement::HorizontalRule);
                    i += 1;
                },
                _ => {
                    // println!("other event = {:?}", &events[i]);
                    i += 1; // Skip other events
                }
            }
        }
        
        Ok(document)
    }

    /// Collect text content until matching end tag
    fn collect_text_until_end(&self, events: &[Event], index: &mut usize, end_tag_name: &str) -> Result<String, ConversionError> {
        let mut text = String::new();
        
        while *index < events.len() {
            match &events[*index] {
                Event::End(tag) => {
                    if self.tag_matches_name(tag, end_tag_name) {
                        *index += 1;
                        break;
                    }
                },
                Event::Text(t) => text.push_str(t),
                Event::Code(c) => text.push_str(c),
                Event::SoftBreak => text.push(' '),
                Event::HardBreak => text.push('\n'),
                _ => {}, // Skip other events
            }
            *index += 1;
        }
        
        Ok(text)
    }

    /// Collect paragraph content and detect standalone images
    fn collect_paragraph_content(&self, events: &[Event], index: &mut usize) -> Result<(Vec<InlineElement>, Option<MarkdownElement>), ConversionError> {
        let mut elements = Vec::new();
        let mut image_element = None;
        let mut has_other_content = false;
        
        while *index < events.len() {
            match &events[*index] {
                Event::End(tag) => {
                    if self.tag_matches_name(tag, "Paragraph") {
                        *index += 1;
                        break;
                    }
                },
                Event::Start(Tag::Image(_, dest_url, title)) => {
                    let url = dest_url.to_string();
                    let title_str = if title.is_empty() { None } else { Some(title.to_string()) };
                    *index += 1;
                    let alt_text = self.collect_text_until_end(events, index, "Image")?;
                    
                    // Store the image element for potential standalone use
                    image_element = Some(MarkdownElement::Image {
                        alt_text: alt_text.clone(),
                        url: url.clone(),
                        title: title_str.clone(),
                    });
                    
                    // Also add as inline element in case it's not standalone
                    elements.push(InlineElement::Link {
                        text: format!("[Image: {}]", alt_text),
                        url,
                        title: title_str,
                    });
                },
                Event::Start(Tag::Strong) => {
                    *index += 1;
                    let text = self.collect_text_until_end(events, index, "Strong")?;
                    elements.push(InlineElement::Bold(text));
                    has_other_content = true;
                },
                Event::Start(Tag::Emphasis) => {
                    *index += 1;
                    let text = self.collect_text_until_end(events, index, "Emphasis")?;
                    elements.push(InlineElement::Italic(text));
                    has_other_content = true;
                },
                Event::Start(Tag::Strikethrough) => {
                    *index += 1;
                    let text = self.collect_text_until_end(events, index, "Strikethrough")?;
                    elements.push(InlineElement::Strikethrough(text));
                    has_other_content = true;
                },
                Event::Start(Tag::Link(_, dest_url, title)) => {
                    let url = dest_url.to_string();
                    let title_str = if title.is_empty() { None } else { Some(title.to_string()) };
                    *index += 1;
                    let text = self.collect_text_until_end(events, index, "Link")?;
                    elements.push(InlineElement::Link { text, url, title: title_str });
                    has_other_content = true;
                },
                Event::Code(code) => {
                    elements.push(InlineElement::Code(code.to_string()));
                    *index += 1;
                    has_other_content = true;
                },
                Event::Text(text) => {
                    let text_str = text.to_string();
                    if !text_str.trim().is_empty() {
                        has_other_content = true;
                    }
                    elements.push(InlineElement::Text(text_str));
                    *index += 1;
                },
                Event::SoftBreak => {
                    elements.push(InlineElement::Text(" ".to_string()));
                    *index += 1;
                },
                Event::HardBreak => {
                    elements.push(InlineElement::Text("\n".to_string()));
                    *index += 1;
                    has_other_content = true;
                },
                _ => {
                    *index += 1; // Skip other events
                }
            }
        }
        
        // If we have an image and no other meaningful content, return it as standalone
        if let Some(image) = image_element {
            if !has_other_content && elements.iter().all(|e| {
                match e {
                    InlineElement::Text(t) => t.trim().is_empty(),
                    InlineElement::Link { text, .. } => text.starts_with("[Image:"),
                    _ => false,
                }
            }) {
                return Ok((Vec::new(), Some(image)));
            }
        }
        
        Ok((elements, None))
    }

    /// Collect inline elements until matching end tag
    fn collect_inline_until_end(&self, events: &[Event], index: &mut usize, end_tag_name: &str) -> Result<Vec<InlineElement>, ConversionError> {
        let mut elements = Vec::new();
        
        while *index < events.len() {
            // println!("event = {:?}, index = {}, len = {}", &events[*index], *index, events.len());
            match &events[*index] {
                Event::End(tag) => {
                    *index += 1;
                    if self.tag_matches_name(tag, end_tag_name) {
                        break;
                    }
                },
                Event::Start(Tag::Strong) => {
                    *index += 1;
                    let text = self.collect_text_until_end(events, index, "Strong")?;
                    elements.push(InlineElement::Bold(text));
                },
                Event::Start(Tag::Emphasis) => {
                    *index += 1;
                    let text = self.collect_text_until_end(events, index, "Emphasis")?;
                    elements.push(InlineElement::Italic(text));
                },
                Event::Start(Tag::Strikethrough) => {
                    *index += 1;
                    let text = self.collect_text_until_end(events, index, "Strikethrough")?;
                    elements.push(InlineElement::Strikethrough(text));
                },
                Event::Start(Tag::Link(_, dest_url, title)) => {
                    let url = dest_url.to_string();
                    let title_str = if title.is_empty() { None } else { Some(title.to_string()) };
                    *index += 1;
                    let text = self.collect_text_until_end(events, index, "Link")?;
                    elements.push(InlineElement::Link { text, url, title: title_str });
                },
                Event::Code(code) => {
                    elements.push(InlineElement::Code(code.to_string()));
                    *index += 1;
                },
                Event::Text(text) => {
                    elements.push(InlineElement::Text(text.to_string()));
                    *index += 1;
                },
                Event::SoftBreak => {
                    elements.push(InlineElement::Text(" ".to_string()));
                    *index += 1;
                },
                Event::HardBreak => {
                    elements.push(InlineElement::Text("\n".to_string()));
                    *index += 1;
                },
                _ => {
                    *index += 1; // Skip other events
                }
            }
        }
        
        Ok(elements)
    }

    /// Collect list items until end of list
    fn collect_list_items(&self, events: &[Event], index: &mut usize) -> Result<Vec<ListItem>, ConversionError> {
        let mut items = Vec::new();
        
        while *index < events.len() {
            // println!("event = {:?}, index = {}, len = {}", &events[*index], *index, events.len());
            match &events[*index] {
                Event::End(Tag::List(_)) => {
                    *index += 1;
                    break;
                },
                Event::Start(Tag::Item) => {
                    *index += 1;
                    let content = self.collect_inline_until_end(events, index, "Item")?;
                    items.push(ListItem::new(content));
                },
                _ => {
                    *index += 1; // Skip other events
                }
            }
        }
        
        Ok(items)
    }

    /// Collect table content until end of table
    fn collect_table_content(&self, events: &[Event], index: &mut usize) -> Result<(Vec<String>, Vec<Vec<String>>), ConversionError> {
        let mut headers = Vec::new();
        let mut rows = Vec::new();
        let mut current_row = Vec::new();
        
        while *index < events.len() {
            match &events[*index] {
                Event::End(Tag::Table(_)) => {
                    *index += 1;
                    break;
                },
                Event::Start(Tag::TableHead) => {
                    *index += 1;
                },
                Event::End(Tag::TableHead) => {
                    headers = current_row.clone();
                    *index += 1;
                },
                Event::Start(Tag::TableRow) => {
                    current_row.clear();
                    *index += 1;
                },
                Event::End(Tag::TableRow) => {
                    rows.push(current_row.clone());
                    *index += 1;
                },
                Event::Start(Tag::TableCell) => {
                    *index += 1;
                    let cell_text = self.collect_text_until_end(events, index, "TableCell")?;
                    current_row.push(cell_text);
                },
                _ => {
                    *index += 1; // Skip other events
                }
            }
        }
        
        Ok((headers, rows))
    }

    /// Check if a tag matches the expected name
    fn tag_matches_name(&self, tag: &Tag, name: &str) -> bool {
        match (tag, name) {
            (Tag::Heading(_, _, _), "Heading") => true,
            (Tag::Paragraph, "Paragraph") => true,
            (Tag::CodeBlock(_), "CodeBlock") => true,
            (Tag::List(_), "List") => true,
            (Tag::Item, "Item") => true,
            (Tag::Strong, "Strong") => true,
            (Tag::Emphasis, "Emphasis") => true,
            (Tag::Strikethrough, "Strikethrough") => true,
            (Tag::Link(_, _, _), "Link") => true,
            (Tag::Image(_, _, _), "Image") => true,
            (Tag::Table(_), "Table") => true,
            (Tag::TableHead, "TableHead") => true,
            (Tag::TableRow, "TableRow") => true,
            (Tag::TableCell, "TableCell") => true,
            _ => false,
        }
    }

    /// Check if a URL is a local file path
    pub fn is_local_path(url: &str) -> bool {
        !url.starts_with("http://") && !url.starts_with("https://") && !url.starts_with("ftp://")
    }

    /// Normalize a URL or file path
    pub fn normalize_url(url: &str) -> String {
        if Self::is_local_path(url) {
            // Handle relative paths
            if url.starts_with("./") {
                url.to_string()
            } else if url.starts_with("../") {
                url.to_string()
            } else if url.starts_with('/') {
                url.to_string()
            } else {
                // Assume it's a relative path and add ./
                format!("./{}", url)
            }
        } else {
            url.to_string()
        }
    }

    /// Extract domain from URL for validation
    pub fn extract_domain(url: &str) -> Option<String> {
        if url.starts_with("http://") || url.starts_with("https://") {
            let without_protocol = if url.starts_with("https://") {
                &url[8..]
            } else {
                &url[7..]
            };
            
            if let Some(slash_pos) = without_protocol.find('/') {
                Some(without_protocol[..slash_pos].to_string())
            } else {
                Some(without_protocol.to_string())
            }
        } else {
            None
        }
    }

    /// Validate if an image URL has a supported extension
    pub fn is_supported_image_format(url: &str) -> bool {
        let supported_extensions = [".jpg", ".jpeg", ".png", ".gif", ".bmp", ".svg", ".webp"];
        let url_lower = url.to_lowercase();
        
        supported_extensions.iter().any(|ext| url_lower.ends_with(ext))
    }
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert pulldown-cmark HeadingLevel to u8
fn heading_level_to_u8(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_heading() {
        let parser = MarkdownParser::new();
        let result = parser.parse("# Hello World").unwrap();
        
        assert_eq!(result.elements.len(), 1);
        match &result.elements[0] {
            MarkdownElement::Heading { level, text } => {
                assert_eq!(*level, 1);
                assert_eq!(text, "Hello World");
            },
            _ => panic!("Expected heading element"),
        }
    }

    #[test]
    fn test_parse_paragraph() {
        let parser = MarkdownParser::new();
        let result = parser.parse("This is a simple paragraph.").unwrap();
        
        assert_eq!(result.elements.len(), 1);
        match &result.elements[0] {
            MarkdownElement::Paragraph { content } => {
                assert_eq!(content.len(), 1);
                match &content[0] {
                    InlineElement::Text(text) => assert_eq!(text, "This is a simple paragraph."),
                    _ => panic!("Expected text element"),
                }
            },
            _ => panic!("Expected paragraph element"),
        }
    }

    #[test]
    fn test_parse_code_block() {
        let parser = MarkdownParser::new();
        let markdown = "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```";
        let result = parser.parse(markdown).unwrap();
        
        assert_eq!(result.elements.len(), 1);
        match &result.elements[0] {
            MarkdownElement::CodeBlock { language, code } => {
                assert_eq!(language.as_ref().unwrap(), "rust");
                assert!(code.contains("fn main()"));
                assert!(code.contains("println!"));
            },
            _ => panic!("Expected code block element"),
        }
    }

    #[test]
    fn test_parse_horizontal_rule() {
        let parser = MarkdownParser::new();
        let result = parser.parse("---").unwrap();
        
        assert_eq!(result.elements.len(), 1);
        match &result.elements[0] {
            MarkdownElement::HorizontalRule => {},
            _ => panic!("Expected horizontal rule element"),
        }
    }

    #[test]
    fn test_parse_formatted_text() {
        let parser = MarkdownParser::new();
        let result = parser.parse("This is **bold** and *italic* text.").unwrap();
        
        assert_eq!(result.elements.len(), 1);
        match &result.elements[0] {
            MarkdownElement::Paragraph { content } => {
                assert_eq!(content.len(), 5);
                
                match &content[0] {
                    InlineElement::Text(text) => assert_eq!(text, "This is "),
                    _ => panic!("Expected text element"),
                }
                
                match &content[1] {
                    InlineElement::Bold(text) => assert_eq!(text, "bold"),
                    _ => panic!("Expected bold element"),
                }
                
                match &content[2] {
                    InlineElement::Text(text) => assert_eq!(text, " and "),
                    _ => panic!("Expected text element"),
                }
                
                match &content[3] {
                    InlineElement::Italic(text) => assert_eq!(text, "italic"),
                    _ => panic!("Expected italic element"),
                }
                
                match &content[4] {
                    InlineElement::Text(text) => assert_eq!(text, " text."),
                    _ => panic!("Expected text element"),
                }
            },
            _ => panic!("Expected paragraph element"),
        }
    }

    #[test]
    fn test_parse_inline_code() {
        let parser = MarkdownParser::new();
        let result = parser.parse("Use `println!()` to print.").unwrap();
        
        assert_eq!(result.elements.len(), 1);
        match &result.elements[0] {
            MarkdownElement::Paragraph { content } => {
                assert_eq!(content.len(), 3);
                
                match &content[1] {
                    InlineElement::Code(code) => assert_eq!(code, "println!()"),
                    _ => panic!("Expected code element"),
                }
            },
            _ => panic!("Expected paragraph element"),
        }
    }

    #[test]
    fn test_parse_link() {
        let parser = MarkdownParser::new();
        let result = parser.parse("Visit [Google](https://google.com) for search.").unwrap();
        
        assert_eq!(result.elements.len(), 1);
        match &result.elements[0] {
            MarkdownElement::Paragraph { content } => {
                assert_eq!(content.len(), 3);
                
                match &content[0] {
                    InlineElement::Text(text) => assert_eq!(text, "Visit "),
                    _ => panic!("Expected text element"),
                }
                
                match &content[1] {
                    InlineElement::Link { text, url, title } => {
                        assert_eq!(text, "Google");
                        assert_eq!(url, "https://google.com");
                        assert_eq!(title, &None);
                    },
                    _ => panic!("Expected link element"),
                }
                
                match &content[2] {
                    InlineElement::Text(text) => assert_eq!(text, " for search."),
                    _ => panic!("Expected text element"),
                }
            },
            _ => panic!("Expected paragraph element"),
        }
    }

    #[test]
    fn test_parse_link_with_title() {
        let parser = MarkdownParser::new();
        let result = parser.parse(r#"Visit [Google](https://google.com "Search Engine") for search."#).unwrap();
        
        assert_eq!(result.elements.len(), 1);
        match &result.elements[0] {
            MarkdownElement::Paragraph { content } => {
                match &content[1] {
                    InlineElement::Link { text, url, title } => {
                        assert_eq!(text, "Google");
                        assert_eq!(url, "https://google.com");
                        assert_eq!(title, &Some("Search Engine".to_string()));
                    },
                    _ => panic!("Expected link element"),
                }
            },
            _ => panic!("Expected paragraph element"),
        }
    }

    #[test]
    fn test_debug_image_events() {
        use pulldown_cmark::{Parser, Event};
        let markdown = "![Alt text](https://example.com/image.jpg)";
        let parser = Parser::new(markdown);
        let events: Vec<Event> = parser.collect();
        
        println!("Events for image: {:?}", events);
        
        // Test standalone image on its own line
        let markdown2 = "\n![Alt text](https://example.com/image.jpg)\n";
        let parser2 = Parser::new(markdown2);
        let events2: Vec<Event> = parser2.collect();
        
        println!("Events for standalone image: {:?}", events2);
        // This test is just for debugging - it will always pass
        assert!(true);
    }

    #[test]
    fn test_parse_image() {
        let parser = MarkdownParser::new();
        let result = parser.parse("![Alt text](https://example.com/image.jpg)").unwrap();
        
        assert_eq!(result.elements.len(), 1);
        match &result.elements[0] {
            MarkdownElement::Image { alt_text, url, title } => {
                assert_eq!(alt_text, "Alt text");
                assert_eq!(url, "https://example.com/image.jpg");
                assert_eq!(title, &None);
            },
            _ => panic!("Expected image element"),
        }
    }

    #[test]
    fn test_parse_image_with_title() {
        let parser = MarkdownParser::new();
        let result = parser.parse(r#"![Alt text](https://example.com/image.jpg "Image Title")"#).unwrap();
        
        assert_eq!(result.elements.len(), 1);
        match &result.elements[0] {
            MarkdownElement::Image { alt_text, url, title } => {
                assert_eq!(alt_text, "Alt text");
                assert_eq!(url, "https://example.com/image.jpg");
                assert_eq!(title, &Some("Image Title".to_string()));
            },
            _ => panic!("Expected image element"),
        }
    }

    #[test]
    fn test_parse_local_image_path() {
        let parser = MarkdownParser::new();
        let result = parser.parse("![Local image](./images/local.png)").unwrap();
        
        assert_eq!(result.elements.len(), 1);
        match &result.elements[0] {
            MarkdownElement::Image { alt_text, url, title } => {
                assert_eq!(alt_text, "Local image");
                assert_eq!(url, "./images/local.png");
                assert_eq!(title, &None);
            },
            _ => panic!("Expected image element"),
        }
    }

    #[test]
    fn test_parse_unordered_list() {
        let parser = MarkdownParser::new();
        let markdown = "- Item 1\n- Item 2\n- Item 3";
        let result = parser.parse(markdown).unwrap();
        
        assert_eq!(result.elements.len(), 1);
        match &result.elements[0] {
            MarkdownElement::List { ordered, items } => {
                assert!(!ordered);
                assert_eq!(items.len(), 3);
                
                let item1_text = items[0].extract_text();
                assert_eq!(item1_text, "Item 1");
                
                let item2_text = items[1].extract_text();
                assert_eq!(item2_text, "Item 2");
                
                let item3_text = items[2].extract_text();
                assert_eq!(item3_text, "Item 3");
            },
            _ => panic!("Expected list element"),
        }
    }

    #[test]
    fn test_parse_ordered_list() {
        let parser = MarkdownParser::new();
        let markdown = "1. First item\n2. Second item\n3. Third item";
        let result = parser.parse(markdown).unwrap();
        
        assert_eq!(result.elements.len(), 1);
        match &result.elements[0] {
            MarkdownElement::List { ordered, items } => {
                assert!(ordered);
                assert_eq!(items.len(), 3);
                
                let item1_text = items[0].extract_text();
                assert_eq!(item1_text, "First item");
            },
            _ => panic!("Expected list element"),
        }
    }

    #[test]
    fn test_debug_table_events() {
        use pulldown_cmark::{Parser, Event};
        let markdown = "| Name | Age |\n|------|-----|\n| Alice | 30 |\n| Bob | 25 |";
        let parser = Parser::new_ext(markdown, pulldown_cmark::Options::ENABLE_TABLES);
        let events: Vec<Event> = parser.collect();
        
        println!("Events for table: {:?}", events);
        assert!(true);
    }

    #[test]
    fn test_parse_table() {
        let parser = MarkdownParser::new();
        let markdown = "| Name | Age |\n|------|-----|\n| Alice | 30 |\n| Bob | 25 |";
        let result = parser.parse(markdown).unwrap();
        
        println!("Parsed elements: {:?}", result.elements);
        assert_eq!(result.elements.len(), 1);
        match &result.elements[0] {
            MarkdownElement::Table { headers, rows } => {
                assert_eq!(headers.len(), 2);
                assert_eq!(headers[0], "Name");
                assert_eq!(headers[1], "Age");
                
                assert_eq!(rows.len(), 2);
                assert_eq!(rows[0][0], "Alice");
                assert_eq!(rows[0][1], "30");
                assert_eq!(rows[1][0], "Bob");
                assert_eq!(rows[1][1], "25");
            },
            _ => panic!("Expected table element"),
        }
    }

    #[test]
    fn test_debug_complex_paragraph_events() {
        use pulldown_cmark::{Parser, Event};
        let markdown = "Check out [this link](https://example.com) and ![this image](image.jpg).";
        let parser = Parser::new(markdown);
        let events: Vec<Event> = parser.collect();
        
        println!("Events for complex paragraph: {:?}", events);
        assert!(true);
    }

    #[test]
    fn test_parse_complex_paragraph_with_links_and_images() {
        let parser = MarkdownParser::new();
        let markdown = "Check out [this link](https://example.com) and ![this image](image.jpg).";
        let result = parser.parse(markdown).unwrap();
        
        println!("Parsed elements: {:?}", result.elements);
        
        // The image should be treated as standalone since it's the only meaningful content after "and "
        // But actually, this should be a single paragraph with both link and image inline
        assert_eq!(result.elements.len(), 1); // Just one paragraph
        
        // Check paragraph with link and image
        match &result.elements[0] {
            MarkdownElement::Paragraph { content } => {
                println!("Paragraph content: {:?}", content);
                // Should contain: "Check out ", link, " and ", image placeholder
                assert!(content.len() >= 3);
                
                // Find the link
                let link_found = content.iter().any(|element| {
                    matches!(element, InlineElement::Link { text, url, .. } 
                        if text == "this link" && url == "https://example.com")
                });
                assert!(link_found, "Expected to find link in paragraph");
            },
            _ => panic!("Expected paragraph element"),
        }
    }

    #[test]
    fn test_empty_document() {
        let parser = MarkdownParser::new();
        let result = parser.parse("").unwrap();
        assert_eq!(result.elements.len(), 0);
    }

    #[test]
    fn test_is_local_path() {
        assert!(MarkdownParser::is_local_path("./image.jpg"));
        assert!(MarkdownParser::is_local_path("../image.jpg"));
        assert!(MarkdownParser::is_local_path("/absolute/path.jpg"));
        assert!(MarkdownParser::is_local_path("relative/path.jpg"));
        assert!(MarkdownParser::is_local_path("image.jpg"));
        
        assert!(!MarkdownParser::is_local_path("https://example.com/image.jpg"));
        assert!(!MarkdownParser::is_local_path("http://example.com/image.jpg"));
        assert!(!MarkdownParser::is_local_path("ftp://example.com/file.jpg"));
    }

    #[test]
    fn test_normalize_url() {
        assert_eq!(MarkdownParser::normalize_url("./image.jpg"), "./image.jpg");
        assert_eq!(MarkdownParser::normalize_url("../image.jpg"), "../image.jpg");
        assert_eq!(MarkdownParser::normalize_url("/absolute/path.jpg"), "/absolute/path.jpg");
        assert_eq!(MarkdownParser::normalize_url("image.jpg"), "./image.jpg");
        assert_eq!(MarkdownParser::normalize_url("folder/image.jpg"), "./folder/image.jpg");
        
        assert_eq!(MarkdownParser::normalize_url("https://example.com/image.jpg"), "https://example.com/image.jpg");
        assert_eq!(MarkdownParser::normalize_url("http://example.com/image.jpg"), "http://example.com/image.jpg");
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(MarkdownParser::extract_domain("https://example.com/path"), Some("example.com".to_string()));
        assert_eq!(MarkdownParser::extract_domain("http://example.com/path"), Some("example.com".to_string()));
        assert_eq!(MarkdownParser::extract_domain("https://subdomain.example.com"), Some("subdomain.example.com".to_string()));
        assert_eq!(MarkdownParser::extract_domain("https://example.com"), Some("example.com".to_string()));
        
        assert_eq!(MarkdownParser::extract_domain("./local/path"), None);
        assert_eq!(MarkdownParser::extract_domain("relative/path"), None);
    }

    #[test]
    fn test_is_supported_image_format() {
        assert!(MarkdownParser::is_supported_image_format("image.jpg"));
        assert!(MarkdownParser::is_supported_image_format("image.jpeg"));
        assert!(MarkdownParser::is_supported_image_format("image.png"));
        assert!(MarkdownParser::is_supported_image_format("image.gif"));
        assert!(MarkdownParser::is_supported_image_format("image.bmp"));
        assert!(MarkdownParser::is_supported_image_format("image.svg"));
        assert!(MarkdownParser::is_supported_image_format("image.webp"));
        assert!(MarkdownParser::is_supported_image_format("IMAGE.JPG")); // Case insensitive
        
        assert!(!MarkdownParser::is_supported_image_format("document.pdf"));
        assert!(!MarkdownParser::is_supported_image_format("video.mp4"));
        assert!(!MarkdownParser::is_supported_image_format("text.txt"));
    }
}