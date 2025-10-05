# Frequently Asked Questions (FAQ)

## General Questions

### Q: What is the Markdown to docx Converter?

**A:** It's a powerful tool that converts Markdown documents to Microsoft Word docx format with configurable formatting rules. It supports natural language configuration updates via LLM integration and provides multiple interfaces: CLI, Web API, and Web UI.

### Q: What makes this converter different from others?

**A:** Key differentiators include:
- **Natural language configuration**: Update formatting rules using plain English
- **Highly configurable**: YAML-based configuration for complete control
- **Multiple interfaces**: CLI, Web API, and Web UI
- **Built with Rust**: High performance and memory safety
- **Batch processing**: Handle multiple files efficiently
- **Rich formatting support**: Tables, code blocks, images, and more

### Q: Is it free to use?

**A:** Yes, the converter is open source under the MIT license. You can use it freely for personal and commercial projects.

## Installation and Setup

### Q: What are the system requirements?

**A:** 
- **Operating System**: Windows, macOS, or Linux
- **Memory**: Minimum 512MB RAM (2GB+ recommended for large files)
- **Disk Space**: 50MB for installation, additional space for processing files
- **Rust**: Version 1.70 or later (for building from source)
- **Node.js**: Version 18+ (for frontend development only)

### Q: How do I install the converter?

**A:** Several options:

1. **From source (recommended):**
   ```bash
   git clone https://github.com/yourusername/md2docx-converter
   cd md2docx-converter
   cargo build --release
   ```

2. **Using Cargo:**
   ```bash
   cargo install md2docx-converter
   ```

3. **Docker:**
   ```bash
   docker pull md2docx-converter:latest
   ```

### Q: Do I need an API key for natural language features?

**A:** Yes, for natural language configuration updates, you need an OpenAI API key:
```bash
export OPENAI_API_KEY=your_api_key_here
```

The converter works without an API key, but you'll need to use YAML configuration files directly.

## Usage Questions

### Q: What Markdown syntax is supported?

**A:** We support CommonMark with GitHub Flavored Markdown extensions:

- **Text formatting**: Bold, italic, strikethrough, inline code
- **Headings**: H1 through H6
- **Lists**: Ordered and unordered, with nesting
- **Code blocks**: With syntax highlighting information
- **Tables**: With headers and alignment
- **Images**: Local files and URLs
- **Links**: Inline and reference style
- **Blockquotes**: Single and nested
- **Horizontal rules**
- **Task lists**: `- [x] Completed task`

### Q: Can I convert multiple files at once?

**A:** Yes, use batch processing:

```bash
# Convert all markdown files in a directory
md2docx-cli convert -i ./docs/ -o ./output/ --batch

# With custom configuration
md2docx-cli convert -i ./docs/ -o ./output/ --batch -c config.yaml
```

### Q: How do I customize the output formatting?

**A:** Three ways:

1. **YAML configuration file:**
   ```bash
   md2docx-cli convert -i input.md -o output.docx -c config.yaml
   ```

2. **Natural language (requires API key):**
   ```bash
   md2docx-cli convert -i input.md -o output.docx --config-prompt "Make headings blue and use Arial font"
   ```

3. **Web interface:** Upload file and use the configuration editor

### Q: Can I use custom fonts?

**A:** Yes, specify any font family in the configuration:

```yaml
document:
  default_font:
    family: "Arial"
    size: 12.0
```

**Note:** The font must be installed on the system where the docx file is opened.

### Q: How large files can I convert?

**A:** The converter can handle:
- **Single files**: Up to several hundred MB (depends on available memory)
- **Batch processing**: Thousands of files
- **Images**: Automatically resized based on configuration
- **Tables**: No practical limit on rows/columns

For very large files, consider splitting them into smaller sections.

## Configuration Questions

### Q: Where can I find example configurations?

**A:** Check the `examples/config.yaml` file in the repository. It contains a complete configuration with comments explaining each option.

### Q: What natural language commands are supported?

**A:** Examples of supported commands:

- **Fonts**: "Change font to Arial", "Make text larger", "Use bold headings"
- **Colors**: "Make headings blue", "Change text color to red"
- **Spacing**: "Increase margins", "Add more space between paragraphs"
- **Alignment**: "Center all images", "Justify paragraph text"
- **Elements**: "Make tables have borders", "Remove underlines from links"

### Q: Can I save and reuse configurations?

**A:** Yes, save your YAML configuration files and reuse them:

```bash
# Save a configuration
md2docx-cli convert -i input.md -o output.docx --config-prompt "My custom style" > my_config.yaml

# Reuse the configuration
md2docx-cli convert -i another.md -o another.docx -c my_config.yaml
```

### Q: How do I reset to default configuration?

**A:** Simply don't specify a configuration file:

```bash
md2docx-cli convert -i input.md -o output.docx
```

Or copy the example configuration:
```bash
cp examples/config.yaml default_config.yaml
```

## Web Interface Questions

### Q: How do I access the web interface?

**A:** 
1. Start the server: `md2docx-server`
2. Open your browser to: `http://localhost:3000`
3. Upload markdown files and configure formatting

### Q: Can I use the web interface without internet?

**A:** Yes, the web interface works offline. However, natural language configuration updates require internet access for the LLM API.

### Q: Is the web interface secure?

**A:** The web interface includes:
- Input validation and sanitization
- Rate limiting
- File size limits
- No external resource fetching

For production use, consider adding authentication and HTTPS.

## API Questions

### Q: How do I use the REST API?

**A:** Send POST requests to `/api/convert`:

```bash
curl -X POST http://localhost:3000/api/convert \
  -H "Content-Type: application/json" \
  -d '{"markdown": "# Hello World", "natural_language": "Make headings blue"}' \
  --output document.docx
```

### Q: What's the API rate limit?

**A:** Default is 100 requests per minute per IP address. This can be configured when starting the server.

### Q: Can I integrate the API with my application?

**A:** Yes, the API is designed for integration. See the API documentation for complete details, including SDKs for Python and JavaScript.

### Q: Does the API support authentication?

**A:** Currently, no authentication is required. For production use, consider implementing API key authentication or placing behind a reverse proxy with authentication.

## Troubleshooting Questions

### Q: Why am I getting "File not found" errors?

**A:** Common causes:
- Incorrect file path
- File doesn't exist
- Permission issues
- Wrong current directory

**Solutions:**
- Use absolute paths: `/full/path/to/file.md`
- Check file exists: `ls -la input.md`
- Verify permissions: `ls -la input.md`

### Q: Why is conversion failing with "Invalid markdown"?

**A:** Possible issues:
- Malformed markdown syntax
- Unsupported markdown features
- File encoding problems

**Solutions:**
- Test with simple markdown: `echo "# Test" > test.md`
- Check file encoding: `file input.md`
- Validate markdown syntax online

### Q: Why are images not appearing in the output?

**A:** Common causes:
- Image files don't exist
- Incorrect image paths
- Unsupported image formats
- Permission issues

**Solutions:**
- Use absolute paths for images
- Ensure images exist: `ls -la image.png`
- Convert to supported formats (PNG, JPEG, GIF, BMP)
- Check image permissions

### Q: Why is the server not starting?

**A:** Common issues:
- Port already in use
- Permission denied
- Missing dependencies

**Solutions:**
- Check port usage: `lsof -i :3000`
- Use different port: `md2docx-server --port 8080`
- Check permissions and dependencies

### Q: Why is natural language processing not working?

**A:** Usually API key related:
- Missing API key: `export OPENAI_API_KEY=your_key`
- Invalid API key
- Network connectivity issues
- API service down

**Workaround:** Use YAML configuration directly instead of natural language.

## Performance Questions

### Q: How can I improve conversion speed?

**A:** Several strategies:

1. **Optimize images**: Compress before processing
2. **Simplify configuration**: Remove complex styling
3. **Use batch processing**: More efficient for multiple files
4. **Parallel processing**: Use tools like GNU parallel
5. **Increase memory**: More RAM helps with large files

### Q: Why is memory usage high?

**A:** Large files and images consume memory. Solutions:
- Process files individually instead of batch
- Reduce image sizes in configuration
- Split large markdown files
- Monitor with `--verbose` flag

### Q: Can I run this on a server?

**A:** Yes, the converter is designed for server deployment:
- Stateless design for horizontal scaling
- Docker support
- Health check endpoints
- Prometheus metrics
- Configurable resource limits

## Development Questions

### Q: How can I contribute to the project?

**A:** We welcome contributions:
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

See CONTRIBUTING.md for detailed guidelines.

### Q: How do I build from source?

**A:** 
```bash
git clone https://github.com/yourusername/md2docx-converter
cd md2docx-converter
cargo build --release
```

For development:
```bash
cargo build
cargo test
cargo run --bin md2docx-cli -- --help
```

### Q: How do I run tests?

**A:** 
```bash
# All tests
cargo test

# Specific test suite
cargo test --lib
cargo test --test integration_tests

# With coverage
cargo tarpaulin --out html
```

### Q: Can I extend the converter with new features?

**A:** Yes, the modular architecture supports extensions:
- Add new markdown elements
- Implement custom formatting rules
- Add new output formats
- Integrate with other APIs

## Deployment Questions

### Q: How do I deploy in production?

**A:** Several options:

1. **Docker:**
   ```bash
   docker run -p 3000:3000 -e OPENAI_API_KEY=your_key md2docx-converter
   ```

2. **Systemd service:**
   ```bash
   sudo systemctl enable md2docx-server
   sudo systemctl start md2docx-server
   ```

3. **Behind reverse proxy:**
   ```nginx
   location /api/ {
       proxy_pass http://localhost:3000/api/;
   }
   ```

### Q: What about monitoring and logging?

**A:** Built-in support for:
- Structured logging with tracing
- Prometheus metrics at `/metrics`
- Health checks at `/api/health`
- Request/response logging
- Error tracking

### Q: How do I handle high traffic?

**A:** Scale horizontally:
- Run multiple instances behind load balancer
- Use Redis for shared state (if needed)
- Implement caching for common conversions
- Monitor resource usage

## Licensing Questions

### Q: What license is used?

**A:** MIT License - you can use it freely for personal and commercial projects.

### Q: Can I use this in commercial products?

**A:** Yes, the MIT license allows commercial use. You can integrate it into commercial products, modify it, and distribute it.

### Q: Do I need to credit the project?

**A:** While not required by the MIT license, attribution is appreciated. You can include a notice in your documentation or about page.

## Integration Questions

### Q: Can I integrate with my CMS?

**A:** Yes, use the REST API to integrate with any system that can make HTTP requests. Common integrations:
- WordPress plugins
- Drupal modules
- Custom web applications
- Documentation systems

### Q: Does it work with CI/CD pipelines?

**A:** Yes, perfect for automated documentation generation:

```yaml
# GitHub Actions example
- name: Convert documentation
  run: |
    md2docx-cli convert -i docs/ -o dist/ --batch
```

### Q: Can I use it with other tools?

**A:** Yes, it works well with:
- **Pandoc**: For additional format conversions
- **GitBook**: For documentation workflows
- **Jekyll/Hugo**: For static site generation
- **Sphinx**: For technical documentation

## Still Have Questions?

If your question isn't answered here:

1. **Check the documentation**: Full docs in the `docs/` directory
2. **Search GitHub issues**: Someone might have asked already
3. **Create a new issue**: Describe your question or problem
4. **Join discussions**: GitHub Discussions for community help

We're always happy to help and improve the documentation based on your feedback!