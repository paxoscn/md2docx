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
                MarkdownElement::CodeBlock { code, .. } => {
                    text.push_str(code);
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

    /// Extract plain text from the element
    pub fn extract_text(&self) -> String {
        match self {
            MarkdownElement::Heading { text, .. } => text.clone(),
            MarkdownElement::Paragraph { content } => {
                content.iter().map(|inline| inline.extract_text()).collect::<Vec<_>>().join("")
            }
            MarkdownElement::CodeBlock { code, .. } => code.clone(),
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
        };
        
        assert_eq!(code_block.element_type(), "code_block");
        assert!(code_block.has_text_content());
        
        let extracted = code_block.extract_text();
        assert!(extracted.contains("fn main()"));
        assert!(extracted.contains("println!"));
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
}