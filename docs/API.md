# API Documentation

This document provides comprehensive documentation for the Markdown to docx Converter REST API.

## Base URL

```
http://localhost:3000/api
```

## Authentication

Currently, the API does not require authentication. In production environments, consider implementing API key authentication.

## Rate Limiting

- **Rate Limit**: 100 requests per minute per IP address
- **Headers**: Rate limit information is included in response headers:
  - `X-RateLimit-Limit`: Maximum requests per window
  - `X-RateLimit-Remaining`: Remaining requests in current window
  - `X-RateLimit-Reset`: Time when the rate limit resets

## Content Types

- **Request**: `application/json`
- **Response**: `application/json` or `application/vnd.openxmlformats-officedocument.wordprocessingml.document`

## Endpoints

### Convert Markdown to docx

Convert Markdown content to a docx document.

```http
POST /api/convert
```

#### Request Body

```json
{
  "markdown": "string (required)",
  "config": "string (optional)",
  "natural_language": "string (optional)"
}
```

**Parameters:**

- `markdown` (string, required): The Markdown content to convert
- `config` (string, optional): YAML configuration string for formatting rules
- `natural_language` (string, optional): Natural language description to modify the configuration

#### Example Request

```bash
curl -X POST http://localhost:3000/api/convert \
  -H "Content-Type: application/json" \
  -d '{
    "markdown": "# Hello World\n\nThis is **bold** text and this is *italic* text.\n\n## Code Example\n\n```rust\nfn main() {\n    println!(\"Hello, world!\");\n}\n```",
    "natural_language": "Make headings blue and use Arial font"
  }' \
  --output document.docx
```

#### Response

**Success (200 OK):**
- **Content-Type**: `application/vnd.openxmlformats-officedocument.wordprocessingml.document`
- **Content-Disposition**: `attachment; filename="document.docx"`
- **Body**: Binary docx file data

**Error (400 Bad Request):**
```json
{
  "error": "Invalid markdown syntax",
  "details": "Specific error message"
}
```

**Error (500 Internal Server Error):**
```json
{
  "error": "Conversion failed",
  "details": "Internal error message"
}
```

### Update Configuration

Update a YAML configuration using natural language.

```http
POST /api/config/update
```

#### Request Body

```json
{
  "config": "string (required)",
  "natural_language": "string (required)"
}
```

**Parameters:**

- `config` (string, required): Current YAML configuration string
- `natural_language` (string, required): Natural language description of desired changes

#### Example Request

```bash
curl -X POST http://localhost:3000/api/config/update \
  -H "Content-Type: application/json" \
  -d '{
    "config": "document:\n  default_font:\n    family: \"Times New Roman\"\n    size: 12.0",
    "natural_language": "Change the font to Arial and make it 14pt"
  }'
```

#### Response

**Success (200 OK):**
```json
{
  "success": true,
  "updated_config": "document:\n  default_font:\n    family: \"Arial\"\n    size: 14.0",
  "changes_made": [
    "Changed font family from 'Times New Roman' to 'Arial'",
    "Changed font size from 12.0 to 14.0"
  ]
}
```

**Error (400 Bad Request):**
```json
{
  "error": "Invalid configuration",
  "details": "YAML parsing error at line 3"
}
```

**Error (422 Unprocessable Entity):**
```json
{
  "error": "Natural language processing failed",
  "details": "Could not understand the requested changes"
}
```

### Health Check

Check the health status of the API.

```http
GET /api/health
```

#### Response

**Success (200 OK):**
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime": 3600,
  "memory_usage": {
    "used": "45.2 MB",
    "total": "512 MB"
  }
}
```

### Configuration Validation

Validate a YAML configuration without processing.

```http
POST /api/config/validate
```

#### Request Body

```json
{
  "config": "string (required)"
}
```

#### Response

**Success (200 OK):**
```json
{
  "valid": true,
  "message": "Configuration is valid"
}
```

**Error (400 Bad Request):**
```json
{
  "valid": false,
  "errors": [
    "Invalid font size: must be positive number",
    "Unknown font family: 'InvalidFont'"
  ]
}
```

## Error Handling

### Error Response Format

All error responses follow this format:

```json
{
  "error": "Error type",
  "details": "Detailed error message",
  "code": "ERROR_CODE",
  "timestamp": "2024-01-01T12:00:00Z"
}
```

### Common Error Codes

| Code | Description |
|------|-------------|
| `INVALID_MARKDOWN` | Markdown syntax is invalid |
| `INVALID_CONFIG` | YAML configuration is invalid |
| `CONVERSION_FAILED` | Document conversion failed |
| `LLM_API_ERROR` | Natural language processing failed |
| `RATE_LIMIT_EXCEEDED` | Too many requests |
| `FILE_TOO_LARGE` | Input file exceeds size limit |
| `UNSUPPORTED_FORMAT` | Unsupported input format |

### HTTP Status Codes

- `200 OK`: Request successful
- `400 Bad Request`: Invalid request parameters
- `422 Unprocessable Entity`: Valid request but processing failed
- `429 Too Many Requests`: Rate limit exceeded
- `500 Internal Server Error`: Server error
- `503 Service Unavailable`: Service temporarily unavailable

## Configuration Schema

### Complete Configuration Structure

```yaml
document:
  page_size:
    width: number      # Page width in points
    height: number     # Page height in points
  margins:
    top: number        # Top margin in points
    bottom: number     # Bottom margin in points
    left: number       # Left margin in points
    right: number      # Right margin in points
  default_font:
    family: string     # Font family name
    size: number       # Font size in points
    bold: boolean      # Bold formatting
    italic: boolean    # Italic formatting
    color: string      # Hex color code

styles:
  headings:
    1:                 # H1 through H6
      font:
        family: string
        size: number
        bold: boolean
        italic: boolean
        color: string
      spacing_before: number
      spacing_after: number
      alignment: string  # left, center, right, justify
      numbering: string  # Optional: numbering format (e.g., "%1.", "%1.%2.")
  
  paragraph:
    font:
      family: string
      size: number
      bold: boolean
      italic: boolean
      color: string
    line_spacing: number
    spacing_after: number
    alignment: string
  
  code_block:
    font:
      family: string
      size: number
    background_color: string
    border: boolean
    padding: number
  
  table:
    header_font:
      family: string
      size: number
      bold: boolean
      italic: boolean
      color: string
    cell_font:
      family: string
      size: number
      bold: boolean
      italic: boolean
      color: string
    border_width: number
    header_background: string

elements:
  image:
    max_width: number
    max_height: number
    alignment: string
  
  list:
    indent: number
    spacing: number
  
  link:
    color: string
    underline: boolean
```

## Natural Language Processing

### Supported Commands

The natural language processor can understand various types of formatting commands:

#### Font Changes
- "Change font to Arial"
- "Make text larger" / "Make text smaller"
- "Use bold headings"
- "Make everything italic"

#### Color Changes
- "Make headings blue"
- "Change text color to red"
- "Use dark gray for code blocks"

#### Spacing and Layout
- "Increase margins"
- "Add more space between paragraphs"
- "Center all images"
- "Justify paragraph text"

#### Element-Specific Changes
- "Make tables have borders"
- "Remove underlines from links"
- "Use monospace font for code"

#### Numbering Changes
- "Add numbering to H1 headings with format 1."
- "Add numbering to H2 headings with format 1.1."
- "Remove numbering from H3 headings"
- "为一级标题添加编号，格式为1." (Chinese)
- "取消二级标题的编号" (Chinese)

### Best Practices

1. **Be Specific**: "Make H1 headings blue" is better than "make headings blue"
2. **Use Standard Terms**: Use common font names and color names
3. **One Change at a Time**: Multiple changes in one request may be less accurate
4. **Test Changes**: Always review the updated configuration before using it

## Examples

### Complete Conversion Example

```javascript
// Convert markdown with custom styling
const response = await fetch('http://localhost:3000/api/convert', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    markdown: `
# Project Report

## Executive Summary

This report provides an **overview** of the project status.

### Key Metrics

| Metric | Value | Status |
|--------|-------|--------|
| Completion | 85% | ✅ On Track |
| Budget | $45,000 | ⚠️ Monitor |

### Code Example

\`\`\`python
def calculate_progress(completed, total):
    return (completed / total) * 100
\`\`\`

For more information, visit [our website](https://example.com).
    `,
    natural_language: "Use Arial font, make headings blue, and center all tables"
  })
});

if (response.ok) {
  const blob = await response.blob();
  // Save or process the docx file
} else {
  const error = await response.json();
  console.error('Conversion failed:', error);
}
```

### Configuration Update Example

```javascript
// Update configuration with natural language
const configResponse = await fetch('http://localhost:3000/api/config/update', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    config: `
document:
  default_font:
    family: "Times New Roman"
    size: 12.0
styles:
  headings:
    1:
      font:
        size: 18.0
        bold: true
    `,
    natural_language: "Change the default font to Calibri and make H1 headings 24pt"
  })
});

const result = await configResponse.json();
console.log('Updated config:', result.updated_config);
```

## SDK Examples

### Python

```python
import requests
import json

class MarkdownConverter:
    def __init__(self, base_url="http://localhost:3000"):
        self.base_url = base_url
    
    def convert(self, markdown, config=None, natural_language=None):
        payload = {"markdown": markdown}
        if config:
            payload["config"] = config
        if natural_language:
            payload["natural_language"] = natural_language
        
        response = requests.post(
            f"{self.base_url}/api/convert",
            json=payload,
            headers={"Content-Type": "application/json"}
        )
        
        if response.status_code == 200:
            return response.content  # Binary docx data
        else:
            raise Exception(f"Conversion failed: {response.json()}")
    
    def update_config(self, config, natural_language):
        response = requests.post(
            f"{self.base_url}/api/config/update",
            json={
                "config": config,
                "natural_language": natural_language
            }
        )
        return response.json()

# Usage
converter = MarkdownConverter()
docx_data = converter.convert(
    "# Hello World\n\nThis is a test.",
    natural_language="Use Arial font and make headings blue"
)

with open("output.docx", "wb") as f:
    f.write(docx_data)
```

### JavaScript/Node.js

```javascript
class MarkdownConverter {
  constructor(baseUrl = 'http://localhost:3000') {
    this.baseUrl = baseUrl;
  }

  async convert(markdown, options = {}) {
    const payload = { markdown };
    if (options.config) payload.config = options.config;
    if (options.naturalLanguage) payload.natural_language = options.naturalLanguage;

    const response = await fetch(`${this.baseUrl}/api/convert`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(`Conversion failed: ${error.details}`);
    }

    return response.blob();
  }

  async updateConfig(config, naturalLanguage) {
    const response = await fetch(`${this.baseUrl}/api/config/update`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        config,
        natural_language: naturalLanguage
      })
    });

    return response.json();
  }
}

// Usage
const converter = new MarkdownConverter();
const docxBlob = await converter.convert(
  '# Hello World\n\nThis is a test.',
  { naturalLanguage: 'Use Arial font and make headings blue' }
);
```

## Monitoring and Logging

### Request Logging

All API requests are logged with the following information:
- Request method and path
- Response status code
- Response time
- Client IP address
- User agent
- Request size
- Response size

### Metrics

The API exposes metrics at `/metrics` endpoint (Prometheus format):
- Request count by endpoint
- Request duration histogram
- Active connections
- Memory usage
- Conversion success/failure rates

### Health Monitoring

Monitor the `/api/health` endpoint for:
- Service availability
- Memory usage
- Uptime
- Version information

## Security Considerations

### Input Validation

- All input is validated and sanitized
- File size limits are enforced
- Markdown content is parsed safely
- YAML configuration is validated

### Rate Limiting

- Implement rate limiting to prevent abuse
- Consider API key authentication for production
- Monitor for unusual usage patterns

### Content Security

- Markdown content is processed in a sandboxed environment
- No external resources are fetched automatically
- Image processing is limited to prevent resource exhaustion

## Deployment

### Environment Variables

- `PORT`: Server port (default: 3000)
- `HOST`: Bind address (default: 0.0.0.0)
- `RUST_LOG`: Log level (debug, info, warn, error)
- `OPENAI_API_KEY`: OpenAI API key for natural language processing
- `MAX_FILE_SIZE`: Maximum file size in bytes
- `RATE_LIMIT`: Requests per minute per IP

### Docker Deployment

```bash
docker run -p 3000:3000 -e OPENAI_API_KEY=your_key md2docx-converter
```

### Load Balancing

The API is stateless and can be horizontally scaled behind a load balancer.