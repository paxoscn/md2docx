# Design Document

## Overview

This design document outlines the implementation of table-based rendering for code blocks in the Markdown to DOCX converter. The feature will replace the current paragraph-based code block rendering with single-row, single-column tables, providing better visual separation and formatting consistency. Additionally, the configuration model will be updated to use `border_width` instead of the boolean `border` property for more granular control.

## Architecture

The implementation involves modifications to three main components:

1. **Configuration Model** (`src/config/models.rs`): Update `CodeBlockStyle` to use `border_width` instead of `border`
2. **DOCX Generator** (`src/docx/generator.rs`): Modify `add_code_block` method to render tables instead of paragraphs
3. **Backward Compatibility**: Ensure existing configurations continue to work

## Components and Interfaces

### 1. Configuration Model Changes

#### CodeBlockStyle Structure Update

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeBlockStyle {
    pub font: FontConfig,
    pub background_color: Option<String>,
    pub border_width: f32,  // Changed from `border: bool`
    pub preserve_line_breaks: bool,
    pub line_spacing: f32,
    pub paragraph_spacing: f32,
}
```

#### Backward Compatibility Handler

A custom deserializer will handle the migration from `border: bool` to `border_width: f32`:

```rust
impl<'de> Deserialize<'de> for CodeBlockStyle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Custom deserialization logic to handle both old and new formats
    }
}
```

### 2. DOCX Generator Changes

#### Table-Based Code Block Rendering

The `add_code_block` method will be refactored to create tables instead of paragraphs:

```rust
fn add_code_block_as_table(&self, mut docx: Docx, code: &str) -> Result<Docx, ConversionError> {
    let code_style = &self.config.styles.code_block;
    
    // Create table cell with code content
    let cell = self.create_code_block_cell(code, code_style)?;
    
    // Create single-row table
    let row = TableRow::new(vec![cell]);
    let mut table = Table::new(vec![row]);
    
    // Apply border styling
    if code_style.border_width > 0.0 {
        table = self.apply_table_borders(table, code_style.border_width)?;
    }
    
    docx.add_table(table)
}
```

#### Border Application

Based on docx-rs 0.4 API, borders will be applied using the table border methods:

```rust
fn apply_table_borders(&self, mut table: Table, border_width: f32) -> Result<Table, ConversionError> {
    // Convert border_width to docx-rs border units
    let border_size = (border_width * 8.0) as usize; // Convert to eighths of a point
    
    // Apply borders to all sides
    table = table
        .set_border_all(BorderType::Single, border_size, 0, "000000")
        .map_err(|e| ConversionError::DocxGeneration(format!("Failed to set borders: {}", e)))?;
    
    Ok(table)
}
```

### 3. Cell Content Handling

#### Line Break Preservation

When `preserve_line_breaks` is enabled, each line will be added as a separate paragraph within the cell:

```rust
fn create_code_block_cell(&self, code: &str, style: &CodeBlockStyle) -> Result<TableCell, ConversionError> {
    let mut cell = TableCell::new();
    
    if style.preserve_line_breaks {
        let lines: Vec<&str> = code.split('\n').collect();
        for line in lines {
            let paragraph = self.create_code_paragraph(line, style)?;
            cell = cell.add_paragraph(paragraph);
        }
    } else {
        let paragraph = self.create_code_paragraph(code, style)?;
        cell = cell.add_paragraph(paragraph);
    }
    
    // Apply background color if specified
    if let Some(bg_color) = &style.background_color {
        cell = self.apply_cell_background(cell, bg_color)?;
    }
    
    Ok(cell)
}
```

## Data Models

### Configuration Migration

The system will support both old and new configuration formats:

**Old Format:**
```yaml
styles:
  code_block:
    border: true  # or false
```

**New Format:**
```yaml
styles:
  code_block:
    border_width: 1.0  # in points
```

### Default Values

- `border_width: 1.0` (equivalent to old `border: true`)
- When migrating from `border: false`, use `border_width: 0.0`

## Error Handling

### Configuration Validation

```rust
impl CodeBlockStyle {
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.font.validate()?;
        if let Some(color) = &self.background_color {
            validate_color(color)?;
        }
        if self.line_spacing <= 0.0 || self.paragraph_spacing < 0.0 {
            return Err(ValidationError::InvalidSpacing);
        }
        if self.border_width < 0.0 {
            return Err(ValidationError::InvalidBorderWidth);
        }
        Ok(())
    }
}
```

### Table Generation Errors

- Handle docx-rs table creation failures gracefully
- Fallback to paragraph rendering if table creation fails
- Log warnings for border application failures

## Testing Strategy

### Unit Tests

1. **Configuration Tests**
   - Test backward compatibility deserialization
   - Test validation of new `border_width` property
   - Test default value assignment

2. **Table Generation Tests**
   - Test single-line code block table creation
   - Test multi-line code block with line break preservation
   - Test empty code block handling
   - Test border application with various widths

3. **Integration Tests**
   - Test complete document generation with table-based code blocks
   - Test configuration migration scenarios
   - Test error handling and fallback behavior

### Test Cases

```rust
#[test]
fn test_code_block_table_generation() {
    let config = ConversionConfig::default();
    let mut generator = DocxGenerator::new(config);
    
    let code = "fn main() {\n    println!(\"Hello, world!\");\n}";
    let document = create_test_document_with_code_block(code);
    
    let result = generator.generate(&document);
    assert!(result.is_ok());
    
    // Verify table structure in generated document
    // (This would require additional docx parsing utilities)
}

#[test]
fn test_border_width_migration() {
    // Test old format with border: true
    let old_config_yaml = r#"
        styles:
          code_block:
            border: true
    "#;
    
    let config: CodeBlockStyle = serde_yaml::from_str(old_config_yaml).unwrap();
    assert_eq!(config.border_width, 1.0);
    
    // Test old format with border: false
    let old_config_yaml = r#"
        styles:
          code_block:
            border: false
    "#;
    
    let config: CodeBlockStyle = serde_yaml::from_str(old_config_yaml).unwrap();
    assert_eq!(config.border_width, 0.0);
}
```

### Performance Considerations

- Table creation may have slightly higher overhead than paragraph creation
- Border application adds processing time but should be minimal
- Memory usage should remain similar as we're still creating similar content structures

### Compatibility Notes

- Requires docx-rs 0.4+ for proper table border support
- Backward compatibility maintained for existing configuration files
- Generated documents will have different internal structure but same visual appearance