# Troubleshooting Guide

This guide helps you diagnose and resolve common issues with the Markdown to docx Converter.

## Quick Diagnostics

### Check System Status

1. **Verify Installation**
   ```bash
   md2docx-cli --version
   ```

2. **Test Basic Functionality**
   ```bash
   echo "# Test" | md2docx-cli convert -i - -o test.docx
   ```

3. **Check Server Health**
   ```bash
   curl http://localhost:3000/api/health
   ```

### Enable Verbose Logging

Add `-v` or `--verbose` to any command for detailed logging:
```bash
md2docx-cli convert -i input.md -o output.docx -v
```

Set environment variable for server logging:
```bash
RUST_LOG=debug md2docx-server
```

## Common Issues

### Installation and Setup Issues

#### Issue: "Command not found: md2docx-cli"

**Symptoms:**
- `md2docx-cli: command not found`
- `No such file or directory`

**Causes:**
- Binary not in PATH
- Installation incomplete
- Wrong binary name

**Solutions:**
1. **Check installation location:**
   ```bash
   find / -name "md2docx-cli" 2>/dev/null
   ```

2. **Use full path:**
   ```bash
   ./target/release/md2docx-cli convert -i input.md -o output.docx
   ```

3. **Add to PATH:**
   ```bash
   export PATH=$PATH:./target/release
   ```

4. **Reinstall:**
   ```bash
   cargo build --release
   ```

#### Issue: "Permission denied"

**Symptoms:**
- `Permission denied (os error 13)`
- Cannot execute binary

**Solutions:**
1. **Make executable:**
   ```bash
   chmod +x ./target/release/md2docx-cli
   chmod +x ./target/release/md2docx-server
   ```

2. **Check file ownership:**
   ```bash
   ls -la ./target/release/md2docx-*
   ```

### File Processing Issues

#### Issue: "File not found" or "No such file or directory"

**Symptoms:**
- `No such file or directory (os error 2)`
- `Input file not found`

**Causes:**
- Incorrect file path
- File doesn't exist
- Permission issues

**Solutions:**
1. **Verify file exists:**
   ```bash
   ls -la input.md
   ```

2. **Use absolute path:**
   ```bash
   md2docx-cli convert -i /full/path/to/input.md -o output.docx
   ```

3. **Check current directory:**
   ```bash
   pwd
   ls -la
   ```

4. **Check file permissions:**
   ```bash
   ls -la input.md
   ```

#### Issue: "Invalid markdown syntax"

**Symptoms:**
- `Markdown parsing failed`
- `Invalid markdown syntax`
- Conversion stops with parsing error

**Causes:**
- Malformed markdown
- Unsupported syntax
- Encoding issues

**Solutions:**
1. **Validate markdown:**
   ```bash
   # Use online markdown validator or
   md2docx-cli convert -i input.md -o output.docx -v
   ```

2. **Check file encoding:**
   ```bash
   file input.md
   # Should show UTF-8 encoding
   ```

3. **Convert encoding if needed:**
   ```bash
   iconv -f ISO-8859-1 -t UTF-8 input.md > input_utf8.md
   ```

4. **Test with minimal example:**
   ```bash
   echo "# Test Header" > test.md
   md2docx-cli convert -i test.md -o test.docx
   ```

#### Issue: "Output file cannot be created"

**Symptoms:**
- `Permission denied` when writing output
- `No space left on device`
- `Directory not found`

**Solutions:**
1. **Check output directory exists:**
   ```bash
   mkdir -p $(dirname output.docx)
   ```

2. **Check write permissions:**
   ```bash
   touch output.docx
   rm output.docx
   ```

3. **Check disk space:**
   ```bash
   df -h .
   ```

4. **Use different output location:**
   ```bash
   md2docx-cli convert -i input.md -o /tmp/output.docx
   ```

### Configuration Issues

#### Issue: "Invalid YAML configuration"

**Symptoms:**
- `YAML parsing error`
- `Invalid configuration format`
- `Configuration validation failed`

**Causes:**
- Syntax errors in YAML
- Missing required fields
- Invalid values

**Solutions:**
1. **Validate YAML syntax:**
   ```bash
   # Use online YAML validator or
   python3 -c "import yaml; yaml.safe_load(open('config.yaml'))"
   ```

2. **Check indentation (use spaces, not tabs):**
   ```bash
   cat -A config.yaml  # Shows tabs as ^I
   ```

3. **Use example configuration:**
   ```bash
   cp examples/config.yaml my_config.yaml
   md2docx-cli convert -i input.md -o output.docx -c my_config.yaml
   ```

4. **Validate specific fields:**
   ```bash
   md2docx-cli validate-config -c config.yaml
   ```

#### Issue: "Font not found" or "Invalid font family"

**Symptoms:**
- Warning about missing fonts
- Default font used instead
- Inconsistent formatting

**Solutions:**
1. **List available fonts:**
   ```bash
   # On macOS
   fc-list : family
   
   # On Linux
   fc-list | grep -i arial
   
   # On Windows
   reg query "HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts"
   ```

2. **Use standard fonts:**
   ```yaml
   document:
     default_font:
       family: "Times New Roman"  # Available on most systems
   ```

3. **Install missing fonts:**
   ```bash
   # On Ubuntu/Debian
   sudo apt install fonts-liberation
   
   # On macOS
   # Install through Font Book application
   ```

### Server Issues

#### Issue: "Address already in use"

**Symptoms:**
- `Address already in use (os error 48)`
- Server fails to start
- Port binding error

**Solutions:**
1. **Check what's using the port:**
   ```bash
   lsof -i :3000
   netstat -tulpn | grep :3000
   ```

2. **Kill existing process:**
   ```bash
   pkill -f md2docx-server
   ```

3. **Use different port:**
   ```bash
   md2docx-server --port 8080
   ```

4. **Wait and retry:**
   ```bash
   # Sometimes need to wait for port to be released
   sleep 5
   md2docx-server
   ```

#### Issue: "Connection refused" or "Server not responding"

**Symptoms:**
- `Connection refused`
- API requests timeout
- Web interface not loading

**Solutions:**
1. **Check server is running:**
   ```bash
   ps aux | grep md2docx-server
   ```

2. **Check server logs:**
   ```bash
   RUST_LOG=debug md2docx-server
   ```

3. **Test local connection:**
   ```bash
   curl http://localhost:3000/api/health
   ```

4. **Check firewall settings:**
   ```bash
   # On Linux
   sudo ufw status
   
   # On macOS
   sudo pfctl -sr
   ```

### API Issues

#### Issue: "Rate limit exceeded"

**Symptoms:**
- `429 Too Many Requests`
- API calls being rejected
- Rate limit headers in response

**Solutions:**
1. **Check rate limit headers:**
   ```bash
   curl -I http://localhost:3000/api/health
   # Look for X-RateLimit-* headers
   ```

2. **Implement backoff:**
   ```bash
   # Wait before retrying
   sleep 60
   ```

3. **Reduce request frequency:**
   - Batch multiple conversions
   - Cache results when possible
   - Use exponential backoff

#### Issue: "LLM API Error" or Natural Language Processing Fails

**Symptoms:**
- `LLM API error`
- `Natural language processing failed`
- Configuration updates don't work

**Causes:**
- Missing API key
- Invalid API key
- Network connectivity issues
- API service down

**Solutions:**
1. **Check API key:**
   ```bash
   echo $OPENAI_API_KEY
   # Should show your API key
   ```

2. **Set API key:**
   ```bash
   export OPENAI_API_KEY=your_api_key_here
   ```

3. **Test API connectivity:**
   ```bash
   curl -H "Authorization: Bearer $OPENAI_API_KEY" \
        https://api.openai.com/v1/models
   ```

4. **Use configuration file instead:**
   ```bash
   # Skip natural language, use direct YAML config
   md2docx-cli convert -i input.md -o output.docx -c config.yaml
   ```

### Memory and Performance Issues

#### Issue: "Out of memory" or "Memory allocation failed"

**Symptoms:**
- `memory allocation of X bytes failed`
- Process killed by system
- Slow performance with large files

**Solutions:**
1. **Check available memory:**
   ```bash
   free -h  # Linux
   vm_stat  # macOS
   ```

2. **Process smaller files:**
   ```bash
   # Split large markdown files
   split -l 1000 large_file.md part_
   ```

3. **Increase system memory:**
   - Close other applications
   - Add more RAM
   - Use swap file

4. **Use streaming processing:**
   ```bash
   # Process files one at a time instead of batch
   for file in *.md; do
     md2docx-cli convert -i "$file" -o "${file%.md}.docx"
   done
   ```

#### Issue: "Conversion is very slow"

**Symptoms:**
- Long processing times
- High CPU usage
- Timeouts

**Solutions:**
1. **Check system resources:**
   ```bash
   top
   htop
   ```

2. **Optimize configuration:**
   - Reduce image sizes
   - Simplify formatting rules
   - Remove unnecessary styling

3. **Use parallel processing:**
   ```bash
   # For batch processing
   find . -name "*.md" | xargs -P 4 -I {} md2docx-cli convert -i {} -o {}.docx
   ```

### Image Processing Issues

#### Issue: "Image not found" or "Cannot load image"

**Symptoms:**
- Images missing from output
- `Image processing failed`
- Broken image references

**Solutions:**
1. **Check image paths:**
   ```bash
   # Ensure images exist
   ls -la images/
   ```

2. **Use absolute paths:**
   ```markdown
   ![Alt text](/full/path/to/image.png)
   ```

3. **Check image formats:**
   - Supported: PNG, JPEG, GIF, BMP
   - Convert if needed: `convert image.webp image.png`

4. **Check image permissions:**
   ```bash
   ls -la image.png
   ```

#### Issue: "Image too large" or "Image processing error"

**Solutions:**
1. **Resize images:**
   ```bash
   # Using ImageMagick
   convert large_image.png -resize 800x600 resized_image.png
   ```

2. **Adjust configuration:**
   ```yaml
   elements:
     image:
       max_width: 400.0
       max_height: 300.0
   ```

## Debugging Techniques

### Enable Debug Logging

```bash
# For CLI
RUST_LOG=debug md2docx-cli convert -i input.md -o output.docx

# For server
RUST_LOG=debug md2docx-server
```

### Test with Minimal Examples

1. **Simple markdown:**
   ```bash
   echo "# Test" > test.md
   md2docx-cli convert -i test.md -o test.docx
   ```

2. **Default configuration:**
   ```bash
   md2docx-cli convert -i input.md -o output.docx
   # Don't use custom config initially
   ```

### Check Dependencies

```bash
# Verify Rust installation
rustc --version
cargo --version

# Check system libraries
ldd target/release/md2docx-cli  # Linux
otool -L target/release/md2docx-cli  # macOS
```

### Validate Input Files

```bash
# Check file encoding
file input.md

# Check file size
ls -lh input.md

# Check content
head -20 input.md
```

## Getting Help

### Collect Diagnostic Information

When reporting issues, include:

1. **System information:**
   ```bash
   uname -a
   rustc --version
   ```

2. **Error messages:**
   ```bash
   md2docx-cli convert -i input.md -o output.docx -v 2>&1 | tee error.log
   ```

3. **Configuration:**
   ```bash
   cat config.yaml
   ```

4. **Sample input:**
   ```bash
   head -50 input.md
   ```

### Create Minimal Reproduction

1. Create the smallest possible markdown file that reproduces the issue
2. Use the simplest configuration possible
3. Document exact steps to reproduce
4. Include full error output

### Community Resources

- **GitHub Issues**: Report bugs and feature requests
- **Discussions**: Ask questions and share tips
- **Documentation**: Check latest documentation
- **Examples**: Review example configurations and usage

## Performance Optimization

### For Large Files

1. **Split processing:**
   ```bash
   # Process chapters separately
   md2docx-cli convert -i chapter1.md -o chapter1.docx
   md2docx-cli convert -i chapter2.md -o chapter2.docx
   ```

2. **Optimize images:**
   ```bash
   # Compress images before processing
   find . -name "*.png" -exec pngquant {} \;
   ```

3. **Simplify configuration:**
   - Remove complex styling
   - Use standard fonts
   - Reduce image sizes

### For Batch Processing

1. **Use parallel processing:**
   ```bash
   find . -name "*.md" | parallel md2docx-cli convert -i {} -o {.}.docx
   ```

2. **Monitor resources:**
   ```bash
   # Watch memory usage
   watch -n 1 'ps aux | grep md2docx'
   ```

### For Server Deployment

1. **Tune server settings:**
   ```bash
   # Increase file limits
   ulimit -n 4096
   
   # Set memory limits
   export RUST_MIN_STACK=8388608
   ```

2. **Use reverse proxy:**
   ```nginx
   # Nginx configuration
   location /api/ {
       proxy_pass http://localhost:3000/api/;
       proxy_timeout 300s;
       client_max_body_size 50M;
   }
   ```

3. **Monitor performance:**
   ```bash
   # Check metrics endpoint
   curl http://localhost:3000/metrics
   ```

## Security Considerations

### Input Validation

- Always validate markdown input
- Sanitize file paths
- Check file sizes
- Validate configuration files

### Network Security

- Use HTTPS in production
- Implement rate limiting
- Monitor for abuse
- Use API keys for authentication

### File System Security

- Restrict file access
- Use temporary directories
- Clean up generated files
- Monitor disk usage

This troubleshooting guide should help you resolve most common issues. If you encounter problems not covered here, please check the GitHub issues or create a new issue with detailed information about your problem.