//! Abstract Syntax Tree definitions for Markdown documents

/// Represents a complete Markdown document
#[derive(Debug, Clone)]
pub struct MarkdownDocument {
    pub elements: Vec<MarkdownElement>,
}

/// Represents different types of Markdown elements
#[derive(Debug, Clone)]
pub enum MarkdownElement {
    Heading {
        level: u8,
        text: String,
    },
    Paragraph {
        content: Vec<InlineElement>,
    },
    CodeBlock {
        language: Option<String>,
        code: String,
        processed: Option<crate::markdown::code_block::ProcessedCodeBlock>,
    },
    List {
        ordered: bool,
        items: Vec<ListItem>,
    },
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    Image {
        alt_text: String,
        url: String,
        title: Option<String>,
    },
    HorizontalRule,
}

/// Represents inline elements within paragraphs
#[derive(Debug, Clone)]
pub enum InlineElement {
    Text(String),
    Bold(String),
    Italic(String),
    Strikethrough(String),
    Code(String),
    Link {
        text: String,
        url: String,
        title: Option<String>,
    },
}

/// Represents a list item
#[derive(Debug, Clone)]
pub struct ListItem {
    pub content: Vec<InlineElement>,
    pub sub_items: Vec<ListItem>,
}

impl MarkdownDocument {
    /// Create a new empty document
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    /// Add an element to the document
    pub fn add_element(&mut self, element: MarkdownElement) {
        self.elements.push(element);
    }

    /// Get all elements of a specific type
    pub fn get_elements_by_type<F>(&self, predicate: F) -> Vec<&MarkdownElement>
    where
        F: Fn(&MarkdownElement) -> bool,
    {
        self.elements.iter().filter(|e| predicate(e)).collect()
    }

    /// Get all headings in the document
    pub fn get_headings(&self) -> Vec<&MarkdownElement> {
        self.get_elements_by_type(|e| matches!(e, MarkdownElement::Heading { .. }))
    }

    /// Get all images in the document
    pub fn get_images(&self) -> Vec<&MarkdownElement> {
        self.get_elements_by_type(|e| matches!(e, MarkdownElement::Image { .. }))
    }

    /// Get all code blocks in the document
    pub fn get_code_blocks(&self) -> Vec<&MarkdownElement> {
        self.get_elements_by_type(|e| matches!(e, MarkdownElement::CodeBlock { .. }))
    }

    /// Get all code blocks mutably in the document
    pub fn get_code_blocks_mut(&mut self) -> Vec<&mut MarkdownElement> {
        self.elements.iter_mut().filter(|e| e.is_code_block()).collect()
    }

    /// Get code blocks by language
    pub fn get_code_blocks_by_language(&self, language: &str) -> Vec<&MarkdownElement> {
        self.get_code_blocks().into_iter()
            .filter(|e| {
                e.get_code_block_language()
                    .map(|lang| lang.eq_ignore_ascii_case(language))
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Get unprocessed code blocks
    pub fn get_unprocessed_code_blocks(&self) -> Vec<&MarkdownElement> {
        self.get_code_blocks().into_iter()
            .filter(|e| !e.is_code_block_processed())
            .collect()
    }

    /// Get processed code blocks
    pub fn get_processed_code_blocks(&self) -> Vec<&MarkdownElement> {
        self.get_code_blocks().into_iter()
            .filter(|e| e.is_code_block_processed())
            .collect()
    }

    /// Count code blocks by processing status
    pub fn count_code_blocks_by_status(&self) -> (usize, usize) {
        let code_blocks = self.get_code_blocks();
        let processed_count = code_blocks.iter().filter(|e| e.is_code_block_processed()).count();
        let total_count = code_blocks.len();
        (processed_count, total_count - processed_count)
    }

    /// Get all tables in the document
    pub fn get_tables(&self) -> Vec<&MarkdownElement> {
        self.get_elements_by_type(|e| matches!(e, MarkdownElement::Table { .. }))
    }

    /// Traverse all elements and apply a function
    pub fn traverse<F>(&self, mut visitor: F)
    where
        F: FnMut(&MarkdownElement),
    {
        for element in &self.elements {
            visitor(element);
            element.traverse_children(&mut visitor);
        }
    }

    /// Traverse all elements mutably and apply a function
    pub fn traverse_mut<F>(&mut self, mut visitor: F)
    where
        F: FnMut(&mut MarkdownElement),
    {
        for element in &mut self.elements {
            visitor(element);
            element.traverse_children_mut(&mut visitor);
        }
    }

    /// Count total number of elements
    pub fn element_count(&self) -> usize {
        let mut count = 0;
        self.traverse(|_| count += 1);
        count
    }

    /// Extract all text content from the document
    pub fn extract_text(&self) -> String {
        let mut text = String::new();
        self.traverse(|element| {
            match element {
                MarkdownElement::Heading { text: heading_text, .. } => {
                    text.push_str(heading_text);
                    text.push('\n');
                }
                MarkdownElement::Paragraph { content } => {
                    for inline in content {
                        text.push_str(&inline.extract_text());
                    }
                    text.push('\n');
                }
                MarkdownElement::CodeBlock { processed, code, .. } => {
                    // Use processed code if available, otherwise use original
                    let final_code = processed.as_ref()
                        .map(|p| p.get_final_code())
                        .unwrap_or(code);
                    text.push_str(final_code);
                    text.push('\n');
                }
                _ => {}
            }
        });
        text
    }
}

impl MarkdownElement {
    /// Traverse child elements (for lists)
    pub fn traverse_children<F>(&self, visitor: &mut F)
    where
        F: FnMut(&MarkdownElement),
    {
        if let MarkdownElement::List { items, .. } = self {
            for item in items {
                item.traverse_children(visitor);
            }
        }
    }

    /// Traverse child elements mutably (for lists)
    pub fn traverse_children_mut<F>(&mut self, visitor: &mut F)
    where
        F: FnMut(&mut MarkdownElement),
    {
        if let MarkdownElement::List { items, .. } = self {
            for item in items {
                item.traverse_children_mut(visitor);
            }
        }
    }

    /// Get the element type as a string
    pub fn element_type(&self) -> &'static str {
        match self {
            MarkdownElement::Heading { .. } => "heading",
            MarkdownElement::Paragraph { .. } => "paragraph",
            MarkdownElement::CodeBlock { .. } => "code_block",
            MarkdownElement::List { .. } => "list",
            MarkdownElement::Table { .. } => "table",
            MarkdownElement::Image { .. } => "image",
            MarkdownElement::HorizontalRule => "horizontal_rule",
        }
    }

    /// Check if element contains text content
    pub fn has_text_content(&self) -> bool {
        matches!(
            self,
            MarkdownElement::Heading { .. }
                | MarkdownElement::Paragraph { .. }
                | MarkdownElement::CodeBlock { .. }
                | MarkdownElement::List { .. }
                | MarkdownElement::Table { .. }
        )
    }

    /// Check if this is a code block element
    pub fn is_code_block(&self) -> bool {
        matches!(self, MarkdownElement::CodeBlock { .. })
    }

    /// Get the code block's language if this is a code block
    pub fn get_code_block_language(&self) -> Option<&String> {
        match self {
            MarkdownElement::CodeBlock { language, .. } => language.as_ref(),
            _ => None,
        }
    }

    /// Get the code block's original code if this is a code block
    pub fn get_code_block_code(&self) -> Option<&String> {
        match self {
            MarkdownElement::CodeBlock { code, .. } => Some(code),
            _ => None,
        }
    }

    /// Get the code block's processed result if this is a code block
    pub fn get_code_block_processed(&self) -> Option<&crate::markdown::code_block::ProcessedCodeBlock> {
        match self {
            MarkdownElement::CodeBlock { processed, .. } => processed.as_ref(),
            _ => None,
        }
    }

    /// Set the processed result for a code block
    pub fn set_code_block_processed(&mut self, processed_block: crate::markdown::code_block::ProcessedCodeBlock) -> Result<(), &'static str> {
        match self {
            MarkdownElement::CodeBlock { processed, .. } => {
                *processed = Some(processed_block);
                Ok(())
            }
            _ => Err("Element is not a code block"),
        }
    }

    /// Get the final code content for a code block (processed if available, otherwise original)
    pub fn get_code_block_final_code(&self) -> Option<&str> {
        match self {
            MarkdownElement::CodeBlock { processed, code, .. } => {
                Some(processed.as_ref()
                    .map(|p| p.get_final_code())
                    .unwrap_or(code))
            }
            _ => None,
        }
    }

    /// Check if a code block has been processed
    pub fn is_code_block_processed(&self) -> bool {
        match self {
            MarkdownElement::CodeBlock { processed, .. } => processed.is_some(),
            _ => false,
        }
    }

    /// Extract plain text from the element
    pub fn extract_text(&self) -> String {
        match self {
            MarkdownElement::Heading { text, .. } => text.clone(),
            MarkdownElement::Paragraph { content } => {
                content.iter().map(|inline| inline.extract_text()).collect::<Vec<_>>().join("")
            }
            MarkdownElement::CodeBlock { processed, code, .. } => {
                // Use processed code if available, otherwise use original
                processed.as_ref()
                    .map(|p| p.get_final_code().to_string())
                    .unwrap_or_else(|| code.clone())
            },
            MarkdownElement::List { items, .. } => {
                items.iter().map(|item| item.extract_text()).collect::<Vec<_>>().join("\n")
            }
            MarkdownElement::Table { headers, rows } => {
                let mut text = headers.join(" | ");
                text.push('\n');
                for row in rows {
                    text.push_str(&row.join(" | "));
                    text.push('\n');
                }
                text
            }
            MarkdownElement::Image { alt_text, .. } => alt_text.clone(),
            MarkdownElement::HorizontalRule => String::new(),
        }
    }
}

impl InlineElement {
    /// Extract plain text from inline element
    pub fn extract_text(&self) -> String {
        match self {
            InlineElement::Text(text) => text.clone(),
            InlineElement::Bold(text) => text.clone(),
            InlineElement::Italic(text) => text.clone(),
            InlineElement::Strikethrough(text) => text.clone(),
            InlineElement::Code(text) => text.clone(),
            InlineElement::Link { text, .. } => text.clone(),
        }
    }

    /// Get the element type as a string
    pub fn element_type(&self) -> &'static str {
        match self {
            InlineElement::Text(_) => "text",
            InlineElement::Bold(_) => "bold",
            InlineElement::Italic(_) => "italic",
            InlineElement::Strikethrough(_) => "strikethrough",
            InlineElement::Code(_) => "code",
            InlineElement::Link { .. } => "link",
        }
    }

    /// Check if element has formatting
    pub fn has_formatting(&self) -> bool {
        matches!(
            self,
            InlineElement::Bold(_) | InlineElement::Italic(_) | InlineElement::Strikethrough(_) | InlineElement::Code(_)
        )
    }
}

impl ListItem {
    /// Create a new list item
    pub fn new(content: Vec<InlineElement>) -> Self {
        Self {
            content,
            sub_items: Vec::new(),
        }
    }

    /// Add a sub-item to this list item
    pub fn add_sub_item(&mut self, item: ListItem) {
        self.sub_items.push(item);
    }

    /// Traverse child elements
    pub fn traverse_children<F>(&self, visitor: &mut F)
    where
        F: FnMut(&MarkdownElement),
    {
        for sub_item in &self.sub_items {
            sub_item.traverse_children(visitor);
        }
    }

    /// Traverse child elements mutably
    pub fn traverse_children_mut<F>(&mut self, visitor: &mut F)
    where
        F: FnMut(&mut MarkdownElement),
    {
        for sub_item in &mut self.sub_items {
            sub_item.traverse_children_mut(visitor);
        }
    }

    /// Extract text from list item
    pub fn extract_text(&self) -> String {
        let mut text = self.content.iter().map(|inline| inline.extract_text()).collect::<Vec<_>>().join("");
        if !self.sub_items.is_empty() {
            text.push('\n');
            for sub_item in &self.sub_items {
                text.push_str(&format!("  {}", sub_item.extract_text()));
            }
        }
        text
    }

    /// Get the depth of nested sub-items
    pub fn max_depth(&self) -> usize {
        if self.sub_items.is_empty() {
            1
        } else {
            1 + self.sub_items.iter().map(|item| item.max_depth()).max().unwrap_or(0)
        }
    }
}

impl Default for MarkdownDocument {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_document_creation() {
        let doc = MarkdownDocument::new();
        assert!(doc.elements.is_empty());
        assert_eq!(doc.element_count(), 0);
    }

    #[test]
    fn test_add_elements() {
        let mut doc = MarkdownDocument::new();
        
        doc.add_element(MarkdownElement::Heading {
            level: 1,
            text: "Title".to_string(),
        });
        
        doc.add_element(MarkdownElement::Paragraph {
            content: vec![InlineElement::Text("Hello world".to_string())],
        });
        
        assert_eq!(doc.elements.len(), 2);
        assert_eq!(doc.element_count(), 2);
    }

    #[test]
    fn test_get_elements_by_type() {
        let mut doc = MarkdownDocument::new();
        
        doc.add_element(MarkdownElement::Heading {
            level: 1,
            text: "Title".to_string(),
        });
        
        doc.add_element(MarkdownElement::Heading {
            level: 2,
            text: "Subtitle".to_string(),
        });
        
        doc.add_element(MarkdownElement::Paragraph {
            content: vec![InlineElement::Text("Content".to_string())],
        });
        
        let headings = doc.get_headings();
        assert_eq!(headings.len(), 2);
        
        for heading in headings {
            assert!(matches!(heading, MarkdownElement::Heading { .. }));
        }
    }

    #[test]
    fn test_extract_text() {
        let mut doc = MarkdownDocument::new();
        
        doc.add_element(MarkdownElement::Heading {
            level: 1,
            text: "Title".to_string(),
        });
        
        doc.add_element(MarkdownElement::Paragraph {
            content: vec![
                InlineElement::Text("Hello ".to_string()),
                InlineElement::Bold("world".to_string()),
            ],
        });
        
        let text = doc.extract_text();
        assert!(text.contains("Title"));
        assert!(text.contains("Hello world"));
    }

    #[test]
    fn test_markdown_element_types() {
        let heading = MarkdownElement::Heading {
            level: 1,
            text: "Title".to_string(),
        };
        assert_eq!(heading.element_type(), "heading");
        assert!(heading.has_text_content());
        
        let paragraph = MarkdownElement::Paragraph {
            content: vec![InlineElement::Text("Content".to_string())],
        };
        assert_eq!(paragraph.element_type(), "paragraph");
        assert!(paragraph.has_text_content());
        
        let hr = MarkdownElement::HorizontalRule;
        assert_eq!(hr.element_type(), "horizontal_rule");
        assert!(!hr.has_text_content());
    }

    #[test]
    fn test_inline_element_text_extraction() {
        let text = InlineElement::Text("plain text".to_string());
        assert_eq!(text.extract_text(), "plain text");
        assert_eq!(text.element_type(), "text");
        assert!(!text.has_formatting());
        
        let bold = InlineElement::Bold("bold text".to_string());
        assert_eq!(bold.extract_text(), "bold text");
        assert_eq!(bold.element_type(), "bold");
        assert!(bold.has_formatting());
        
        let link = InlineElement::Link {
            text: "link text".to_string(),
            url: "https://example.com".to_string(),
            title: None,
        };
        assert_eq!(link.extract_text(), "link text");
        assert_eq!(link.element_type(), "link");
        assert!(!link.has_formatting());
    }

    #[test]
    fn test_list_item_functionality() {
        let mut item = ListItem::new(vec![
            InlineElement::Text("Main item".to_string()),
        ]);
        
        let sub_item = ListItem::new(vec![
            InlineElement::Text("Sub item".to_string()),
        ]);
        
        item.add_sub_item(sub_item);
        
        assert_eq!(item.max_depth(), 2);
        
        let text = item.extract_text();
        assert!(text.contains("Main item"));
        assert!(text.contains("Sub item"));
    }

    #[test]
    fn test_code_block_element() {
        let code_block = MarkdownElement::CodeBlock {
            language: Some("rust".to_string()),
            code: "fn main() {\n    println!(\"Hello\");\n}".to_string(),
            processed: None,
        };
        
        assert_eq!(code_block.element_type(), "code_block");
        assert!(code_block.has_text_content());
        assert!(code_block.is_code_block());
        assert!(!code_block.is_code_block_processed());
        
        let extracted = code_block.extract_text();
        assert!(extracted.contains("fn main()"));
        assert!(extracted.contains("println!"));
        
        // Test helper methods
        assert_eq!(code_block.get_code_block_language(), Some(&"rust".to_string()));
        assert_eq!(code_block.get_code_block_code(), Some(&"fn main() {\n    println!(\"Hello\");\n}".to_string()));
        assert!(code_block.get_code_block_processed().is_none());
        assert_eq!(code_block.get_code_block_final_code(), Some("fn main() {\n    println!(\"Hello\");\n}"));
    }

    #[test]
    fn test_code_block_with_processed_result() {
        use crate::markdown::code_block::ProcessedCodeBlock;
        
        let original_code = "fn main(){println!(\"Hello\");}";
        let formatted_code = "fn main() {\n    println!(\"Hello\");\n}";
        
        let processed = ProcessedCodeBlock::new(
            original_code.to_string(),
            Some("rust".to_string())
        )
        .with_processed_code(formatted_code.to_string())
        .with_validation(true);
        
        let mut code_block = MarkdownElement::CodeBlock {
            language: Some("rust".to_string()),
            code: original_code.to_string(),
            processed: None,
        };
        
        // Test setting processed result
        assert!(code_block.set_code_block_processed(processed).is_ok());
        assert!(code_block.is_code_block_processed());
        
        // Test that final code returns processed version
        assert_eq!(code_block.get_code_block_final_code(), Some(formatted_code));
        
        // Test that extract_text returns processed version
        let extracted = code_block.extract_text();
        assert_eq!(extracted, formatted_code);
        
        // Test processed result access
        let processed_result = code_block.get_code_block_processed().unwrap();
        assert_eq!(processed_result.original_code, original_code);
        assert_eq!(processed_result.get_final_code(), formatted_code);
        assert!(processed_result.metadata.is_formatted);
        assert!(processed_result.metadata.syntax_valid);
    }

    #[test]
    fn test_code_block_helper_methods_on_non_code_block() {
        let heading = MarkdownElement::Heading {
            level: 1,
            text: "Title".to_string(),
        };
        
        assert!(!heading.is_code_block());
        assert!(heading.get_code_block_language().is_none());
        assert!(heading.get_code_block_code().is_none());
        assert!(heading.get_code_block_processed().is_none());
        assert!(heading.get_code_block_final_code().is_none());
        assert!(!heading.is_code_block_processed());
        
        // Test that setting processed result fails on non-code-block
        use crate::markdown::code_block::ProcessedCodeBlock;
        let processed = ProcessedCodeBlock::new("test".to_string(), None);
        let mut heading_mut = heading;
        assert!(heading_mut.set_code_block_processed(processed).is_err());
    }

    #[test]
    fn test_table_element() {
        let table = MarkdownElement::Table {
            headers: vec!["Name".to_string(), "Age".to_string()],
            rows: vec![
                vec!["Alice".to_string(), "30".to_string()],
                vec!["Bob".to_string(), "25".to_string()],
            ],
        };
        
        assert_eq!(table.element_type(), "table");
        assert!(table.has_text_content());
        
        let extracted = table.extract_text();
        assert!(extracted.contains("Name | Age"));
        assert!(extracted.contains("Alice | 30"));
        assert!(extracted.contains("Bob | 25"));
    }

    #[test]
    fn test_image_element() {
        let image = MarkdownElement::Image {
            alt_text: "A beautiful sunset".to_string(),
            url: "https://example.com/sunset.jpg".to_string(),
            title: Some("Sunset".to_string()),
        };
        
        assert_eq!(image.element_type(), "image");
        assert!(!image.has_text_content());
        
        let extracted = image.extract_text();
        assert_eq!(extracted, "A beautiful sunset");
    }

    #[test]
    fn test_list_element() {
        let list = MarkdownElement::List {
            ordered: false,
            items: vec![
                ListItem::new(vec![InlineElement::Text("Item 1".to_string())]),
                ListItem::new(vec![InlineElement::Text("Item 2".to_string())]),
            ],
        };
        
        assert_eq!(list.element_type(), "list");
        assert!(list.has_text_content());
        
        let extracted = list.extract_text();
        assert!(extracted.contains("Item 1"));
        assert!(extracted.contains("Item 2"));
    }

    #[test]
    fn test_document_traversal() {
        let mut doc = MarkdownDocument::new();
        
        doc.add_element(MarkdownElement::Heading {
            level: 1,
            text: "Title".to_string(),
        });
        
        doc.add_element(MarkdownElement::List {
            ordered: false,
            items: vec![
                ListItem::new(vec![InlineElement::Text("Item 1".to_string())]),
            ],
        });
        
        let mut element_types = Vec::new();
        doc.traverse(|element| {
            element_types.push(element.element_type());
        });
        
        assert_eq!(element_types, vec!["heading", "list"]);
    }

    #[test]
    fn test_nested_list_depth() {
        let mut main_item = ListItem::new(vec![
            InlineElement::Text("Level 1".to_string()),
        ]);
        
        let mut sub_item = ListItem::new(vec![
            InlineElement::Text("Level 2".to_string()),
        ]);
        
        let sub_sub_item = ListItem::new(vec![
            InlineElement::Text("Level 3".to_string()),
        ]);
        
        sub_item.add_sub_item(sub_sub_item);
        main_item.add_sub_item(sub_item);
        
        assert_eq!(main_item.max_depth(), 3);
    }

    #[test]
    fn test_complex_paragraph() {
        let paragraph = MarkdownElement::Paragraph {
            content: vec![
                InlineElement::Text("This is ".to_string()),
                InlineElement::Bold("bold".to_string()),
                InlineElement::Text(" and this is ".to_string()),
                InlineElement::Italic("italic".to_string()),
                InlineElement::Text(" and this is ".to_string()),
                InlineElement::Code("code".to_string()),
                InlineElement::Text(" and this is a ".to_string()),
                InlineElement::Link {
                    text: "link".to_string(),
                    url: "https://example.com".to_string(),
                    title: None,
                },
            ],
        };
        
        let extracted = paragraph.extract_text();
        assert_eq!(extracted, "This is bold and this is italic and this is code and this is a link");
    }

    #[test]
    fn test_document_code_block_methods() {
        use crate::markdown::code_block::ProcessedCodeBlock;
        
        let mut doc = MarkdownDocument::new();
        
        // Add various elements including code blocks
        doc.add_element(MarkdownElement::Heading {
            level: 1,
            text: "Title".to_string(),
        });
        
        doc.add_element(MarkdownElement::CodeBlock {
            language: Some("rust".to_string()),
            code: "fn main() {}".to_string(),
            processed: None,
        });
        
        doc.add_element(MarkdownElement::CodeBlock {
            language: Some("javascript".to_string()),
            code: "console.log('hello');".to_string(),
            processed: None,
        });
        
        doc.add_element(MarkdownElement::CodeBlock {
            language: Some("rust".to_string()),
            code: "let x = 5;".to_string(),
            processed: Some(ProcessedCodeBlock::new(
                "let x = 5;".to_string(),
                Some("rust".to_string())
            )),
        });
        
        // Test code block retrieval methods
        let all_code_blocks = doc.get_code_blocks();
        assert_eq!(all_code_blocks.len(), 3);
        
        let rust_blocks = doc.get_code_blocks_by_language("rust");
        assert_eq!(rust_blocks.len(), 2);
        
        let js_blocks = doc.get_code_blocks_by_language("javascript");
        assert_eq!(js_blocks.len(), 1);
        
        let python_blocks = doc.get_code_blocks_by_language("python");
        assert_eq!(python_blocks.len(), 0);
        
        // Test processing status methods
        let unprocessed = doc.get_unprocessed_code_blocks();
        assert_eq!(unprocessed.len(), 2);
        
        let processed = doc.get_processed_code_blocks();
        assert_eq!(processed.len(), 1);
        
        let (processed_count, unprocessed_count) = doc.count_code_blocks_by_status();
        assert_eq!(processed_count, 1);
        assert_eq!(unprocessed_count, 2);
    }

    #[test]
    fn test_document_code_block_mutation() {
        use crate::markdown::code_block::ProcessedCodeBlock;
        
        let mut doc = MarkdownDocument::new();
        
        doc.add_element(MarkdownElement::CodeBlock {
            language: Some("rust".to_string()),
            code: "fn main(){}".to_string(),
            processed: None,
        });
        
        doc.add_element(MarkdownElement::Paragraph {
            content: vec![InlineElement::Text("Not a code block".to_string())],
        });
        
        // Test mutable access to code blocks
        let mut code_blocks_mut = doc.get_code_blocks_mut();
        assert_eq!(code_blocks_mut.len(), 1);
        
        // Process the code block
        let processed = ProcessedCodeBlock::new(
            "fn main(){}".to_string(),
            Some("rust".to_string())
        )
        .with_processed_code("fn main() {}\n".to_string());
        
        assert!(code_blocks_mut[0].set_code_block_processed(processed).is_ok());
        
        // Verify the change
        let (processed_count, unprocessed_count) = doc.count_code_blocks_by_status();
        assert_eq!(processed_count, 1);
        assert_eq!(unprocessed_count, 0);
    }
}