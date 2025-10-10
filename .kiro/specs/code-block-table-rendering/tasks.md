# Implementation Plan

- [x] 1. Update configuration model to use border_width instead of border
  - Modify `CodeBlockStyle` struct in `src/config/models.rs` to replace `border: bool` with `border_width: f32`
  - Add validation for `border_width` to ensure it's non-negative
  - Update default configuration to use `border_width: 1.0`
  - _Requirements: 3.1, 3.2, 3.4_

- [x] 2. Implement backward compatibility for configuration deserialization
  - Create custom deserializer for `CodeBlockStyle` that handles both old and new formats
  - Map `border: true` to `border_width: 1.0` and `border: false` to `border_width: 0.0`
  - Ensure new `border_width` property takes precedence when both are present
  - _Requirements: 4.1, 4.2, 4.3, 4.4_

- [x] 3. Create table-based code block rendering infrastructure
  - Add helper method `create_code_block_cell` to generate table cells with code content
  - Add helper method `apply_table_borders` to apply border styling based on `border_width`
  - Add helper method `create_code_paragraph` for consistent code paragraph creation within cells
  - _Requirements: 1.1, 2.1, 2.2, 2.3_

- [x] 4. Implement single-row table generation for code blocks
  - Modify `add_code_block` method in `src/docx/generator.rs` to create tables instead of paragraphs
  - Create single-row, single-column table structure for each code block
  - Handle empty code blocks by creating tables with appropriate minimum content
  - _Requirements: 1.1, 1.3_

- [x] 5. Implement line break preservation within table cells
  - Handle multi-line code blocks by creating separate paragraphs for each line within the table cell
  - Preserve empty lines using non-breaking spaces when `preserve_line_breaks` is enabled
  - Apply line spacing configuration within table cell paragraphs
  - _Requirements: 1.2, 5.4_

- [x] 6. Apply border styling to code block tables
  - Implement border application using docx-rs table border API
  - Convert `border_width` from points to docx-rs border units (eighths of a point)
  - Handle `border_width: 0.0` by not applying any borders
  - _Requirements: 2.1, 2.2, 2.3_

- [x] 7. Maintain existing font and styling options for table-rendered code blocks
  - Apply configured font family, size, bold, and italic settings to table cell content
  - Apply background color to table cells when configured
  - Ensure all existing code block styling options work with table rendering
  - _Requirements: 5.1, 5.2, 5.3_

- [x] 8. Add comprehensive unit tests for configuration changes
  - Test `CodeBlockStyle` validation with new `border_width` property
  - Test backward compatibility deserialization for both `border: true` and `border: false`
  - Test default configuration generation with new property
  - Test error handling for invalid `border_width` values
  - _Requirements: 3.2, 4.1, 4.2, 4.3, 4.4_

- [x] 9. Add unit tests for table-based code block generation
  - Test single-line code block table creation
  - Test multi-line code block with line break preservation
  - Test empty code block table handling
  - Test border application with various `border_width` values
  - Test font and styling application within table cells
  - _Requirements: 1.1, 1.2, 1.3, 2.1, 2.2, 2.3, 5.1, 5.2_

- [x] 10. Add integration tests for complete document generation
  - Test document generation with table-based code blocks
  - Test configuration migration scenarios in real document conversion
  - Test error handling and graceful degradation when table creation fails
  - Verify generated DOCX structure contains proper table elements
  - _Requirements: 1.4, 4.1, 4.2, 4.3, 4.4_

- [x] 11. Update test utilities and example configurations
  - Update `create_test_config` function in `src/test_utils.rs` to use new `border_width` property
  - Update example configuration files to demonstrate new `border_width` usage
  - Ensure all existing tests pass with the new configuration structure
  - _Requirements: 3.4, 4.1, 4.2_