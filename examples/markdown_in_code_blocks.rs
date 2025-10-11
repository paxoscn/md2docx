//! Example demonstrating Markdown formatting within code blocks
//! 
//! This example shows how the DocxGenerator can handle code blocks that contain
//! Markdown formatting like bold, italic, links, and images.

use md2docx_converter::{
    ConversionConfig, DocxGenerator, MarkdownParser,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a Markdown document with code blocks containing Markdown formatting
    let markdown_content = r#"
# Code Block with Markdown Formatting

Here's a code block that contains Markdown formatting:

```markdown
This is **bold text** and *italic text*.
You can also have `inline code` within the block.
Links work too: [Example](https://example.com)
And images: ![Alt text](image.jpg)
```

Here's another example with mixed content:

```text
**Important:** This is a *critical* section.
See the documentation at [docs](https://docs.example.com).
```

And a plain code block for comparison:

```rust
fn main() {
    println!("Hello, world!");
}
```
"#;

    // Parse the Markdown
    let parser = MarkdownParser::new();
    let document = parser.parse(markdown_content)?;

    // Generate docx with default configuration
    let config = ConversionConfig::default();
    let mut generator = DocxGenerator::new(config);
    
    let docx_bytes = generator.generate(&document)?;

    // Save to file
    std::fs::write("markdown_in_code_blocks_example.docx", docx_bytes)?;
    
    println!("Generated docx file: markdown_in_code_blocks_example.docx");
    println!("The code blocks with Markdown formatting should render with:");
    println!("- Bold and italic text properly formatted");
    println!("- Links displayed with URL");
    println!("- Images shown as placeholders");
    println!("- Inline code with different highlighting");

    Ok(())
}