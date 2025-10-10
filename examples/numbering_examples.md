# Heading Numbering Examples

This document demonstrates various heading numbering formats and their output when using the auto-numbering feature.

## Basic Numbering Formats

### Single Level Numbering (%1.)

Configuration:
```yaml
styles:
  headings:
    1:
      numbering: "%1."
```

Output:
- 1. First Chapter
- 2. Second Chapter  
- 3. Third Chapter

### Two Level Numbering (%1.%2.)

Configuration:
```yaml
styles:
  headings:
    1:
      numbering: "%1."
    2:
      numbering: "%1.%2."
```

Output:
- 1. First Chapter
  - 1.1. First Section
  - 1.2. Second Section
- 2. Second Chapter
  - 2.1. First Section
  - 2.2. Second Section

### Three Level Numbering (%1.%2.%3)

Configuration:
```yaml
styles:
  headings:
    1:
      numbering: "%1."
    2:
      numbering: "%1.%2."
    3:
      numbering: "%1.%2.%3"
```

Output:
- 1. First Chapter
  - 1.1. First Section
    - 1.1.1 First Subsection
    - 1.1.2 Second Subsection
  - 1.2. Second Section
    - 1.2.1 First Subsection
- 2. Second Chapter
  - 2.1. First Section
    - 2.1.1 First Subsection

## Custom Separator Examples

### Dash Separators (%1-%2-%3)

Configuration:
```yaml
styles:
  headings:
    1:
      numbering: "%1"
    2:
      numbering: "%1-%2"
    3:
      numbering: "%1-%2-%3"
```

Output:
- 1 First Chapter
  - 1-1 First Section
    - 1-1-1 First Subsection
    - 1-1-2 Second Subsection
  - 1-2 Second Section
- 2 Second Chapter

### Custom Text with Numbers

Configuration:
```yaml
styles:
  headings:
    1:
      numbering: "Chapter %1:"
    2:
      numbering: "Section %1.%2"
    3:
      numbering: "%1.%2.%3 -"
```

Output:
- Chapter 1: Introduction
  - Section 1.1 Overview
    - 1.1.1 - Background
    - 1.1.2 - Objectives
  - Section 1.2 Methodology
- Chapter 2: Results

## Advanced Examples

### Academic Paper Style

Configuration:
```yaml
styles:
  headings:
    1:
      numbering: "%1."
      font:
        size: 16.0
        bold: true
    2:
      numbering: "%1.%2"
      font:
        size: 14.0
        bold: true
    3:
      numbering: "%1.%2.%3"
      font:
        size: 12.0
        bold: true
```

Sample Document:
```markdown
# Introduction
## Literature Review
### Previous Studies
### Research Gaps
## Methodology
### Data Collection
### Analysis Methods
# Results
## Findings
### Primary Results
### Secondary Results
# Discussion
# Conclusion
```

Output:
- 1. Introduction
  - 1.1 Literature Review
    - 1.1.1 Previous Studies
    - 1.1.2 Research Gaps
  - 1.2 Methodology
    - 1.2.1 Data Collection
    - 1.2.2 Analysis Methods
- 2. Results
  - 2.1 Findings
    - 2.1.1 Primary Results
    - 2.1.2 Secondary Results
- 3. Discussion
- 4. Conclusion

### Legal Document Style

Configuration:
```yaml
styles:
  headings:
    1:
      numbering: "%1."
    2:
      numbering: "%1.%2."
    3:
      numbering: "%1.%2.%3."
    4:
      numbering: "(%1.%2.%3.%4)"
```

Sample Document:
```markdown
# Definitions
## General Terms
### Interpretation
#### Specific Clauses
# Obligations
## Party A Obligations
### Financial Obligations
#### Payment Terms
### Performance Obligations
## Party B Obligations
```

Output:
- 1. Definitions
  - 1.1. General Terms
    - 1.1.1. Interpretation
      - (1.1.1.1) Specific Clauses
- 2. Obligations
  - 2.1. Party A Obligations
    - 2.1.1. Financial Obligations
      - (2.1.1.1) Payment Terms
    - 2.1.2. Performance Obligations
  - 2.2. Party B Obligations

### Technical Manual Style

Configuration:
```yaml
styles:
  headings:
    1:
      numbering: "Chapter %1:"
      font:
        size: 18.0
        bold: true
        color: "#2E4057"
    2:
      numbering: "%1.%2"
      font:
        size: 16.0
        bold: true
        color: "#2E4057"
    3:
      numbering: "%1.%2.%3"
      font:
        size: 14.0
        bold: true
```

Sample Document:
```markdown
# Getting Started
## Installation
### System Requirements
### Download and Install
## Configuration
### Basic Setup
### Advanced Options
# User Guide
## Basic Operations
### Creating Documents
### Editing Documents
## Advanced Features
```

Output:
- Chapter 1: Getting Started
  - 1.1 Installation
    - 1.1.1 System Requirements
    - 1.1.2 Download and Install
  - 1.2 Configuration
    - 1.2.1 Basic Setup
    - 1.2.2 Advanced Options
- Chapter 2: User Guide
  - 2.1 Basic Operations
    - 2.1.1 Creating Documents
    - 2.1.2 Editing Documents
  - 2.2 Advanced Features

## Mixed Numbering Scenarios

### Selective Numbering

Configuration (only H1 and H3 have numbering):
```yaml
styles:
  headings:
    1:
      numbering: "%1."
    2:
      # No numbering field - H2 will not be numbered
    3:
      numbering: "%1.%2.%3"  # Uses H1 counter and synthetic H2
```

Sample Document:
```markdown
# Chapter One
## Overview Section
### First Subsection
### Second Subsection
## Details Section
### Another Subsection
# Chapter Two
## Summary Section
### Final Subsection
```

Output:
- 1. Chapter One
  - Overview Section
    - 1.1.1 First Subsection
    - 1.1.2 Second Subsection
  - Details Section
    - 1.2.1 Another Subsection
- 2. Chapter Two
  - Summary Section
    - 2.1.1 Final Subsection

## Skip-Level Handling

When heading levels are skipped, the numbering system handles it gracefully:

Sample Document:
```markdown
# Introduction
### Quick Start (skipping H2)
## Detailed Guide (back to H2)
### Step by Step
##### Advanced Tips (skipping H4)
# Conclusion
```

With standard numbering configuration:
```yaml
styles:
  headings:
    1:
      numbering: "%1."
    2:
      numbering: "%1.%2."
    3:
      numbering: "%1.%2.%3"
    5:
      numbering: "%1.%2.%3.%4.%5"
```

Output:
- 1. Introduction
  - 1.1.1 Quick Start (uses synthetic H2 counter)
  - 1.1. Detailed Guide
    - 1.1.1 Step by Step
      - 1.1.1.1.1 Advanced Tips (uses synthetic H4 counter)
- 2. Conclusion

## Natural Language Configuration Examples

You can configure numbering using natural language descriptions:

### English Examples
- "Add numbering to H1 headings with format 1."
- "Add numbering to H2 headings with format 1.1."
- "Add numbering to H3 headings with format 1.1.1"
- "Remove numbering from H2 headings"
- "Change H1 numbering format to Chapter 1:"

### Chinese Examples (中文)
- "为一级标题添加编号，格式为1."
- "为二级标题添加编号，格式为1.1."
- "为三级标题添加编号，格式为1.1.1"
- "取消二级标题的编号"
- "修改一级标题编号格式为第1章："

## Error Handling Examples

### Invalid Format Handling

Invalid configurations are handled gracefully:

```yaml
styles:
  headings:
    1:
      numbering: ""  # Empty format - will be ignored
    2:
      numbering: "no placeholders"  # No %N - will be ignored
    3:
      numbering: "%1.%3."  # Skip level - will be ignored
```

Result: Headings will display without numbering, and warnings will be logged.

### Graceful Degradation

When numbering fails, the system falls back to original heading text:

```markdown
# This heading will work
## This heading might fail but won't crash the system
### This heading will continue working
```

Even with invalid numbering configuration, the document will still be converted successfully, just without the problematic numbering.

## Best Practices

1. **Start Simple**: Begin with basic formats like "%1." and "%1.%2."
2. **Be Consistent**: Use similar patterns across heading levels
3. **Test Thoroughly**: Validate configuration before production use
4. **Use Sequential Levels**: Always use %1, %1.%2, %1.%2.%3 (not %1.%3)
5. **Consider Readability**: Don't make numbering formats too complex
6. **Document Structure**: Use proper heading hierarchy in your Markdown

## Troubleshooting

### Common Issues

1. **Numbering not appearing**: Check that the `numbering` field is present and valid
2. **Invalid format errors**: Ensure placeholders are sequential (%1, %1.%2, etc.)
3. **Incorrect sequence**: Verify document heading structure is logical
4. **Performance issues**: Use simpler formats for large documents

### Validation

Always validate your configuration:

```bash
# CLI validation
md2docx-cli validate-config -c your_config.yaml

# API validation
curl -X POST http://localhost:3000/api/config/validate \
  -H "Content-Type: application/json" \
  -d '{"config": "your_yaml_config_here"}'
```