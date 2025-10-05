# Markdown to docx Converter

A powerful, configurable Markdown to Microsoft Word docx converter with web API, CLI, and web interface. Features natural language configuration updates via LLM integration.

## Features

- **🔧 Configurable Conversion**: YAML-based configuration for complete control over formatting rules
- **🤖 Natural Language Config**: Update configurations using natural language descriptions via LLM integration
- **🌐 Multiple Interfaces**: Web API, CLI tool, and intuitive web interface
- **📦 Batch Processing**: Convert multiple files efficiently
- **🎨 Rich Formatting**: Support for headings, tables, code blocks, images, lists, and more
- **⚡ High Performance**: Built with Rust for speed and reliability
- **🔒 Type Safety**: Comprehensive error handling and validation

## Quick Start

### Installation

#### From Source
```bash
git clone https://github.com/yourusername/md2docx-converter
cd md2docx-converter
cargo build --release
```

#### Using Cargo
```bash
cargo install md2docx-converter
```

### Basic Usage

Convert a Markdown file to docx:
```bash
md2docx-cli convert -i document.md -o document.docx
```

Start the web server:
```bash
md2docx-server
```

Then visit `http://localhost:3000` to use the web interface.

## Usage Guide

### Command Line Interface (CLI)

#### Single File Conversion
```bash
# Basic conversion
md2docx-cli convert -i input.md -o output.docx

# With custom configuration
md2docx-cli convert -i input.md -o output.docx -c config.yaml

# With natural language configuration
md2docx-cli convert -i input.md -o output.docx --config-prompt "Make all headings blue and use Arial font"
```

#### Batch Conversion
```bash
# Convert all markdown files in a directory
md2docx-cli convert -i ./docs/ -o ./output/ --batch

# With custom configuration for batch processing
md2docx-cli convert -i ./docs/ -o ./output/ --batch -c config.yaml
```

#### CLI Options
- `-i, --input <PATH>`: Input file or directory path
- `-o, --output <PATH>`: Output file or directory path
- `-c, --config <PATH>`: Configuration file path
- `--config-prompt <TEXT>`: Natural language configuration modification
- `--batch`: Enable batch processing for directories
- `-v, --verbose`: Enable verbose logging
- `-h, --help`: Show help information

### Web Server

#### Starting the Server
```bash
# Default port (3000)
md2docx-server

# Custom port
md2docx-server --port 8080

# With custom configuration
md2docx-server --config config.yaml
```

#### Server Options
- `-p, --port <PORT>`: Server port (default: 3000)
- `-c, --config <PATH>`: Default configuration file
- `--host <HOST>`: Bind address (default: 0.0.0.0)
- `-v, --verbose`: Enable verbose logging

### Web API

#### Convert Endpoint
```http
POST /api/convert
Content-Type: application/json

{
  "markdown": "# Hello World\n\nThis is a **bold** text.",
  "config": "optional YAML config string",
  "natural_language": "Make headings larger and use Times New Roman"
}
```

Response:
```http
HTTP/1.1 200 OK
Content-Type: application/vnd.openxmlformats-officedocument.wordprocessingml.document
Content-Disposition: attachment; filename="document.docx"

[Binary docx data]
```

#### Configuration Update Endpoint
```http
POST /api/config/update
Content-Type: application/json

{
  "config": "current YAML config",
  "natural_language": "Change all headings to blue color"
}
```

Response:
```json
{
  "success": true,
  "updated_config": "updated YAML config string"
}
```

#### Health Check
```http
GET /api/health
```

Response:
```json
{
  "status": "healthy",
  "version": "0.1.0"
}
```

## Configuration

### YAML Configuration Structure

The converter uses YAML files to define formatting rules. Here's the complete structure:

```yaml
document:
  page_size:
    width: 595.0    # A4 width in points
    height: 842.0   # A4 height in points
  margins:
    top: 72.0       # 1 inch in points
    bottom: 72.0
    left: 72.0
    right: 72.0
  default_font:
    family: "Times New Roman"
    size: 12.0
    bold: false
    italic: false

styles:
  headings:
    1:  # H1 style
      font:
        family: "Times New Roman"
        size: 18.0
        bold: true
        italic: false
        color: "#000000"
      spacing_before: 12.0
      spacing_after: 6.0
    2:  # H2 style
      font:
        family: "Times New Roman"
        size: 16.0
        bold: true
      spacing_before: 12.0
      spacing_after: 6.0
    # ... up to H6
  
  paragraph:
    font:
      family: "Times New Roman"
      size: 12.0
    line_spacing: 1.15
    spacing_after: 6.0
    alignment: "left"  # left, center, right, justify
  
  code_block:
    font:
      family: "Courier New"
      size: 10.0
    background_color: "#f5f5f5"
    border: true
    padding: 8.0
  
  table:
    header_font:
      family: "Times New Roman"
      size: 12.0
      bold: true
    cell_font:
      family: "Times New Roman"
      size: 12.0
    border_width: 1.0
    header_background: "#f0f0f0"

elements:
  image:
    max_width: 500.0
    max_height: 400.0
    alignment: "center"
  
  list:
    indent: 36.0
    spacing: 6.0
  
  link:
    color: "#0066cc"
    underline: true
```

### Natural Language Configuration

You can modify configurations using natural language descriptions:

#### Examples
- "Make all headings blue and larger"
- "Use Arial font for the entire document"
- "Set margins to 1.5 inches on all sides"
- "Make code blocks have a gray background"
- "Center all images and make them smaller"

#### Supported Modifications
- **Fonts**: Change font family, size, color, bold, italic
- **Spacing**: Modify margins, padding, line spacing
- **Colors**: Set text colors, background colors
- **Alignment**: Change text and image alignment
- **Sizes**: Adjust font sizes, image dimensions

## Supported Markdown Elements

### Text Formatting
- **Bold**: `**bold text**` or `__bold text__`
- **Italic**: `*italic text*` or `_italic text_`
- **Strikethrough**: `~~strikethrough~~`
- **Inline Code**: `` `code` ``

### Headings
```markdown
# Heading 1
## Heading 2
### Heading 3
#### Heading 4
##### Heading 5
###### Heading 6
```

### Lists
```markdown
# Unordered List
- Item 1
- Item 2
  - Nested item

# Ordered List
1. First item
2. Second item
   1. Nested item
```

### Code Blocks
````markdown
```rust
fn main() {
    println!("Hello, world!");
}
```
````

### Tables
```markdown
| Header 1 | Header 2 | Header 3 |
|----------|----------|----------|
| Cell 1   | Cell 2   | Cell 3   |
| Cell 4   | Cell 5   | Cell 6   |
```

### Images
```markdown
![Alt text](image.png)
![Alt text](https://example.com/image.jpg)
```

### Links
```markdown
[Link text](https://example.com)
```

## Development

### Prerequisites
- Rust 1.70 or later
- Node.js 18+ (for frontend development)

### Building
```bash
# Build the entire project
cargo build --release

# Build specific binary
cargo build --release --bin md2docx-server
cargo build --release --bin md2docx-cli
```

### Testing
```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --lib
cargo test --test integration_tests

# Run with coverage
cargo tarpaulin --out html
```

### Frontend Development
```bash
cd frontend
npm install
npm run dev
```

### Running in Development
```bash
# CLI tool
cargo run --bin md2docx-cli -- convert -i example.md -o output.docx

# Web server
cargo run --bin md2docx-server

# With environment variables
RUST_LOG=debug cargo run --bin md2docx-server
```

## Troubleshooting

### Common Issues

#### "File not found" Error
**Problem**: Input file cannot be found
**Solution**: 
- Check the file path is correct
- Ensure the file has `.md` extension
- Use absolute paths if relative paths don't work

#### "Invalid YAML configuration" Error
**Problem**: Configuration file has syntax errors
**Solution**:
- Validate YAML syntax using online validators
- Check indentation (use spaces, not tabs)
- Ensure all required fields are present

#### "LLM API Error" 
**Problem**: Natural language configuration fails
**Solution**:
- Check internet connection
- Verify API key is set: `export OPENAI_API_KEY=your_key`
- Try simpler natural language descriptions

#### "Memory allocation failed"
**Problem**: Large files cause memory issues
**Solution**:
- Process files in smaller batches
- Increase system memory
- Use streaming processing for very large files

#### "Permission denied" Error
**Problem**: Cannot write output file
**Solution**:
- Check write permissions on output directory
- Ensure output directory exists
- Run with appropriate user permissions

### Performance Issues

#### Slow Conversion
- Use `--batch` for multiple files
- Enable parallel processing
- Check system resources

#### High Memory Usage
- Process files individually instead of batch
- Reduce image sizes in configuration
- Monitor with `--verbose` flag

### Getting Help

1. **Check the logs**: Run with `-v` or `--verbose` flag
2. **Validate configuration**: Use online YAML validators
3. **Test with minimal example**: Try converting a simple markdown file
4. **Check system resources**: Ensure adequate memory and disk space

## FAQ

### Q: What Markdown flavors are supported?
A: We support CommonMark with GitHub Flavored Markdown extensions including tables, strikethrough, and task lists.

### Q: Can I use custom fonts?
A: Yes, specify any font family in the configuration. The font must be installed on the system where the docx file is opened.

### Q: How large files can I convert?
A: The tool can handle files up to several hundred MB, depending on available system memory.

### Q: Is there a rate limit for the API?
A: Yes, the default rate limit is 100 requests per minute per IP address.

### Q: Can I run this in Docker?
A: Yes, see the Docker section in the deployment documentation.

### Q: How do I contribute?
A: See CONTRIBUTING.md for development setup and contribution guidelines.

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Support

- 📖 Documentation: [Full documentation](docs/)
- 🐛 Issues: [GitHub Issues](https://github.com/yourusername/md2docx-converter/issues)
- 💬 Discussions: [GitHub Discussions](https://github.com/yourusername/md2docx-converter/discussions)