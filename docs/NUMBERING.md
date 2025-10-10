# Heading Auto-Numbering Feature

This document provides comprehensive documentation for the heading auto-numbering feature in the Markdown to docx converter.

## Overview

The heading auto-numbering feature automatically adds numerical prefixes to headings based on their level and position in the document. This feature is configurable through YAML configuration and supports various numbering formats and patterns.

## Features

- **Configurable Numbering**: Enable numbering for specific heading levels (H1-H6)
- **Multiple Formats**: Support for various numbering patterns (1., 1.1., 1.1.1, etc.)
- **Custom Separators**: Use custom separators and text in numbering formats
- **Skip-Level Handling**: Proper numbering when heading levels are skipped
- **Mixed Scenarios**: Support documents with both numbered and non-numbered headings
- **Error Handling**: Graceful degradation when numbering configuration is invalid
- **Natural Language Config**: Update numbering through natural language descriptions

## Configuration

### Basic Configuration

Add the `numbering` field to heading styles in your YAML configuration:

```yaml
styles:
  headings:
    1:  # H1 headings
      font:
        family: "Times New Roman"
        size: 18.0
        bold: true
      numbering: "%1."  # Simple numbering: 1., 2., 3.
    2:  # H2 headings
      font:
        family: "Times New Roman"
        size: 16.0
        bold: true
      numbering: "%1.%2."  # Two-level: 1.1., 1.2., 2.1.
    3:  # H3 headings
      font:
        family: "Times New Roman"
        size: 14.0
        bold: true
      numbering: "%1.%2.%3"  # Three-level: 1.1.1, 1.1.2, 1.2.1
    # H4, H5, H6 without numbering (omit the numbering field)
```

### Numbering Format Patterns

The numbering format uses placeholder patterns where `%N` represents the counter for heading level N:

| Format | Example Output | Description |
|--------|----------------|-------------|
| `%1.` | 1., 2., 3. | Simple single-level numbering |
| `%1.%2.` | 1.1., 1.2., 2.1. | Two-level numbering with dots |
| `%1.%2.%3` | 1.1.1, 1.1.2, 1.2.1 | Three-level numbering |
| `%1-%2-%3` | 1-1-1, 1-1-2, 1-2-1 | Custom separator (dashes) |
| `Chapter %1` | Chapter 1, Chapter 2 | Text prefix |
| `%1.%2:` | 1.1:, 1.2:, 2.1: | Custom suffix |

### Format Rules

1. **Sequential Levels**: Placeholders must be sequential starting from %1 (e.g., %1.%2.%3, not %1.%3)
2. **Valid Levels**: Only levels 1-6 are supported (%1 through %6)
3. **No Empty Formats**: Format strings cannot be empty
4. **Custom Text**: Any text can be included around placeholders

### Advanced Configuration Examples

#### Academic Paper Style
```yaml
styles:
  headings:
    1:
      numbering: "%1."  # 1., 2., 3.
    2:
      numbering: "%1.%2"  # 1.1, 1.2, 2.1
    3:
      numbering: "%1.%2.%3"  # 1.1.1, 1.1.2
```

#### Book Chapter Style
```yaml
styles:
  headings:
    1:
      numbering: "Chapter %1:"  # Chapter 1:, Chapter 2:
    2:
      numbering: "%1.%2"  # 1.1, 1.2, 2.1
    3:
      numbering: "%1.%2.%3"  # 1.1.1, 1.1.2
```

#### Legal Document Style
```yaml
styles:
  headings:
    1:
      numbering: "%1."  # 1., 2., 3.
    2:
      numbering: "%1.%2."  # 1.1., 1.2., 2.1.
    3:
      numbering: "%1.%2.%3."  # 1.1.1., 1.1.2.
    4:
      numbering: "(%1.%2.%3.%4)"  # (1.1.1.1), (1.1.1.2)
```

## Usage Examples

### CLI Usage

```bash
# Convert with numbering configuration
md2docx-cli convert -i document.md -o document.docx -c numbering_config.yaml

# Use natural language to add numbering
md2docx-cli convert -i document.md -o document.docx \
  --config-prompt "Add numbering to H1 headings with format '1.' and H2 headings with format '1.1.'"
```

### API Usage

```bash
# Convert with numbering via API
curl -X POST http://localhost:3000/api/convert \
  -H "Content-Type: application/json" \
  -d '{
    "markdown": "# Chapter 1\n\n## Section 1.1\n\nContent here.",
    "config": "styles:\n  headings:\n    1:\n      numbering: \"%1.\"\n    2:\n      numbering: \"%1.%2.\""
  }' \
  --output document.docx
```

### Natural Language Configuration

You can configure numbering using natural language descriptions:

```bash
# English examples
"Add numbering to H1 headings with format 1."
"Add numbering to H2 headings with format 1.1."
"Remove numbering from H3 headings"

# Chinese examples (中文)
"为一级标题添加编号，格式为1."
"为二级标题添加编号，格式为1.1."
"取消三级标题的编号"
```

## Behavior and Logic

### Numbering State Management

The numbering system maintains counters for each heading level (H1-H6):

1. **Increment**: When encountering a heading at the same level, increment its counter
2. **Reset Lower Levels**: When encountering a higher-level heading, reset all lower-level counters
3. **Skip Levels**: When skipping heading levels, maintain proper counter relationships

### Example Document Flow

```markdown
# Introduction          → 1. Introduction
## Overview            → 1.1. Overview
### Details            → 1.1.1 Details
### More Details       → 1.1.2 More Details
## Methodology         → 1.2. Methodology
# Results              → 2. Results (resets H2, H3 counters)
## Findings            → 2.1. Findings
```

### Skip-Level Handling

When heading levels are skipped, the system handles it gracefully:

```markdown
# Chapter 1             → 1. Chapter 1
### Subsection          → 1.1.1 Subsection (H2 counter used implicitly)
## Section 1.1          → 1.1. Section 1.1 (H3 counter reset)
### Details             → 1.1.1 Details
```

### Mixed Numbering Scenarios

You can have some heading levels with numbering and others without:

```yaml
styles:
  headings:
    1:
      numbering: "%1."     # H1 has numbering
    2:
      # No numbering field - H2 has no numbering
    3:
      numbering: "%1.%2.%3"  # H3 has numbering (uses H1 and synthetic H2)
```

## Error Handling and Graceful Degradation

The numbering system implements comprehensive error handling:

### Invalid Format Handling

When an invalid numbering format is encountered:
- The system logs a warning
- Falls back to displaying the original heading text without numbering
- Continues processing other headings normally

### Counter Overflow

When counters reach their maximum value (u32::MAX):
- The counter resets to 1
- A warning is logged
- Processing continues normally

### Configuration Errors

When configuration validation fails:
- Invalid formats are ignored
- Valid formats continue to work
- Detailed error messages are provided

### Example Error Scenarios

```yaml
# Invalid format examples that will be handled gracefully
styles:
  headings:
    1:
      numbering: ""              # Empty format - ignored
    2:
      numbering: "no placeholders"  # No %N - ignored
    3:
      numbering: "%1.%3."        # Skip level - error logged, ignored
```

## Performance Considerations

### Optimization Features

- **Lazy Processing**: Numbering is only applied to levels that have it configured
- **Efficient State Management**: O(1) counter operations
- **Memory Efficient**: Minimal memory overhead per document
- **Batch Processing**: Optimized for processing multiple headings

### Performance Benchmarks

Based on testing with various document sizes:

| Document Size | Headings | Processing Time | Memory Usage |
|---------------|----------|-----------------|--------------|
| Small (1-10 headings) | 10 | < 1ms | < 1KB |
| Medium (50-100 headings) | 100 | < 10ms | < 5KB |
| Large (500+ headings) | 500 | < 50ms | < 20KB |

## Integration with Other Features

### Compatibility

The numbering feature is fully compatible with all other Markdown features:

- **Text Formatting**: Bold, italic, strikethrough work in numbered headings
- **Code Blocks**: Numbering works alongside code syntax highlighting
- **Tables**: Numbered headings can contain tables
- **Images**: Images in numbered sections work correctly
- **Links**: Links in numbered headings are preserved

### Web Interface Integration

The web interface provides:
- **Real-time Preview**: See numbering as you edit configuration
- **Error Feedback**: Visual indicators for invalid numbering formats
- **Format Suggestions**: Helpful examples and templates

## Troubleshooting

### Common Issues

#### Numbering Not Appearing

**Problem**: Headings don't show numbering
**Solutions**:
1. Check that `numbering` field is present in heading configuration
2. Verify the format string is valid (use validation endpoint)
3. Ensure heading level matches configuration (H1 = level 1, etc.)

#### Invalid Format Errors

**Problem**: "Invalid numbering format" errors
**Solutions**:
1. Ensure placeholders are sequential (%1, %1.%2, %1.%2.%3)
2. Use only levels 1-6 (%1 through %6)
3. Don't skip levels (%1.%3 is invalid)

#### Incorrect Numbering Sequence

**Problem**: Numbers don't increment correctly
**Solutions**:
1. Check document structure for proper heading hierarchy
2. Verify no duplicate heading levels without content between them
3. Reset numbering state if processing multiple documents

#### Performance Issues

**Problem**: Slow processing with numbering enabled
**Solutions**:
1. Reduce number of heading levels with numbering
2. Use simpler format patterns
3. Process documents in smaller batches

### Debugging Tools

#### Validation Endpoint

Test your numbering configuration:

```bash
curl -X POST http://localhost:3000/api/config/validate \
  -H "Content-Type: application/json" \
  -d '{
    "config": "styles:\n  headings:\n    1:\n      numbering: \"%1.\""
  }'
```

#### CLI Validation

```bash
md2docx-cli validate-config -c your_config.yaml
```

#### Logging

Enable debug logging to see numbering operations:

```bash
RUST_LOG=debug md2docx-cli convert -i document.md -o output.docx
```

### Error Messages Reference

| Error Message | Cause | Solution |
|---------------|-------|----------|
| "Format string cannot be empty" | Empty numbering field | Provide a valid format or remove the field |
| "No valid placeholders found" | Format without %N patterns | Add at least one %N placeholder |
| "Level X is out of range" | Using %0 or %7+ | Use only %1 through %6 |
| "Levels must be sequential" | Skipping levels like %1.%3 | Use sequential levels: %1.%2.%3 |
| "Counter overflow" | Too many headings at one level | This is handled automatically |

## Best Practices

### Configuration Design

1. **Start Simple**: Begin with basic formats like "%1." and "%1.%2."
2. **Be Consistent**: Use similar patterns across heading levels
3. **Test Thoroughly**: Validate configuration before production use
4. **Document Choices**: Comment your configuration files

### Document Structure

1. **Proper Hierarchy**: Use heading levels in logical order (H1 → H2 → H3)
2. **Avoid Deep Nesting**: Limit to 3-4 heading levels for readability
3. **Consistent Style**: Apply numbering consistently throughout document
4. **Clear Separation**: Use content between headings of the same level

### Performance Optimization

1. **Selective Numbering**: Only enable numbering for levels that need it
2. **Simple Formats**: Use simpler patterns for better performance
3. **Batch Processing**: Process multiple documents together when possible
4. **Monitor Metrics**: Use health endpoints to monitor performance

## API Reference

### Configuration Fields

```yaml
# Heading style with numbering
heading_style:
  font:
    family: string      # Font family name
    size: number        # Font size in points
    bold: boolean       # Bold formatting
    italic: boolean     # Italic formatting
    color: string       # Hex color code
  spacing_before: number  # Space before heading
  spacing_after: number   # Space after heading
  alignment: string       # Text alignment
  numbering: string       # Numbering format (optional)
```

### Natural Language Commands

#### English Commands
- "Add numbering to H{N} headings with format {format}"
- "Remove numbering from H{N} headings"
- "Change H{N} numbering format to {format}"

#### Chinese Commands (中文)
- "为{N}级标题添加编号，格式为{format}"
- "取消{N}级标题的编号"
- "修改{N}级标题编号格式为{format}"

### REST API Endpoints

#### Convert with Numbering
```http
POST /api/convert
Content-Type: application/json

{
  "markdown": "string",
  "config": "yaml_string_with_numbering",
  "natural_language": "optional_numbering_description"
}
```

#### Validate Numbering Configuration
```http
POST /api/config/validate
Content-Type: application/json

{
  "config": "yaml_string_with_numbering"
}
```

#### Update Configuration with Natural Language
```http
POST /api/config/update
Content-Type: application/json

{
  "config": "current_yaml_config",
  "natural_language": "add numbering to H1 headings with format 1."
}
```

## Examples and Templates

### Academic Paper Template

```yaml
document:
  page_size:
    width: 595.0
    height: 842.0
  margins:
    top: 72.0
    bottom: 72.0
    left: 72.0
    right: 72.0

styles:
  headings:
    1:
      font:
        family: "Times New Roman"
        size: 16.0
        bold: true
      numbering: "%1."
      spacing_before: 12.0
      spacing_after: 6.0
    2:
      font:
        family: "Times New Roman"
        size: 14.0
        bold: true
      numbering: "%1.%2"
      spacing_before: 10.0
      spacing_after: 5.0
    3:
      font:
        family: "Times New Roman"
        size: 12.0
        bold: true
      numbering: "%1.%2.%3"
      spacing_before: 8.0
      spacing_after: 4.0
```

### Technical Manual Template

```yaml
styles:
  headings:
    1:
      font:
        family: "Arial"
        size: 18.0
        bold: true
        color: "#2E4057"
      numbering: "Chapter %1:"
      spacing_before: 15.0
      spacing_after: 8.0
    2:
      font:
        family: "Arial"
        size: 16.0
        bold: true
        color: "#2E4057"
      numbering: "%1.%2"
      spacing_before: 12.0
      spacing_after: 6.0
    3:
      font:
        family: "Arial"
        size: 14.0
        bold: true
      numbering: "%1.%2.%3"
      spacing_before: 10.0
      spacing_after: 5.0
    4:
      font:
        family: "Arial"
        size: 12.0
        bold: true
      numbering: "%1.%2.%3.%4"
      spacing_before: 8.0
      spacing_after: 4.0
```

### Legal Document Template

```yaml
styles:
  headings:
    1:
      font:
        family: "Times New Roman"
        size: 14.0
        bold: true
      numbering: "%1."
      alignment: "center"
    2:
      font:
        family: "Times New Roman"
        size: 13.0
        bold: true
      numbering: "%1.%2."
    3:
      font:
        family: "Times New Roman"
        size: 12.0
        bold: true
      numbering: "%1.%2.%3."
    4:
      font:
        family: "Times New Roman"
        size: 12.0
        bold: false
      numbering: "(%1.%2.%3.%4)"
```

## Migration Guide

### Upgrading from Non-Numbered Documents

1. **Backup**: Always backup your existing configuration
2. **Gradual Introduction**: Start with H1 numbering only
3. **Test**: Validate with sample documents before full deployment
4. **Monitor**: Check output quality and performance

### Converting Existing Configurations

```bash
# Example migration script
# Add numbering to existing H1 and H2 styles

# Before (existing config)
styles:
  headings:
    1:
      font:
        family: "Arial"
        size: 18.0

# After (with numbering)
styles:
  headings:
    1:
      font:
        family: "Arial"
        size: 18.0
      numbering: "%1."  # Added numbering
```

## Support and Resources

### Documentation
- [API Documentation](API.md)
- [Configuration Guide](CONFIG.md)
- [Troubleshooting Guide](TROUBLESHOOTING.md)

### Community
- GitHub Issues: Report bugs and request features
- GitHub Discussions: Ask questions and share configurations
- Examples Repository: Community-contributed templates

### Professional Support
- Enterprise support available for production deployments
- Custom numbering format development
- Performance optimization consulting