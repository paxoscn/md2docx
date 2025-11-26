# Rust Comment Italic Formatting Feature

## Overview

This feature adds automatic italic formatting for Rust comments in code blocks, complementing the existing bold formatting for Rust keywords.

## Changes Made

### 1. `src/markdown/code_block/strategies/rust_strategy.rs`

#### New Method: `apply_comment_italic`
- Detects Rust line comments (`//`) in code
- Wraps comments with `[ITALIC]...[/ITALIC]` tags
- Handles both standalone comments and inline comments
- Preserves code structure and line breaks

#### Updated Method: `format_rust_code`
- Now applies comment italic formatting before keyword bold formatting
- Ensures proper ordering to avoid tag conflicts

#### New Tests
- `test_comment_italic_formatting`: Tests basic comment detection and tagging
- `test_format_code_includes_italic_comments`: Tests integration with formatting
- `test_process_with_formatting_includes_italic_comments`: Tests end-to-end processing

### 2. `src/docx/generator.rs`

#### Updated Method: `parse_inline_formatting_with_tags`
- Extended to handle both `[BOLD]` and `[ITALIC]` tags
- Processes tags in order of appearance
- Supports mixed bold and italic formatting
- Handles unclosed tags gracefully

#### New Tests
- `test_parse_inline_formatting_with_italic_tags`: Tests italic tag parsing
- `test_parse_inline_formatting_with_mixed_tags`: Tests mixed bold/italic
- `test_parse_inline_formatting_with_nested_order`: Tests tag ordering

## Usage

When processing Rust code blocks with the `RustStrategy`:

```rust
// This is a comment
fn main() {
    let x = 42; // inline comment
    println!("Hello");
}
```

The output will have:
- Keywords (`fn`, `let`) rendered in **bold** using `[BOLD]...[/BOLD]` tags
- Comments (`// This is a comment`, `// inline comment`) rendered in *italic* using `[ITALIC]...[/ITALIC]` tags

## Configuration

Enable Rust code processing in your config:

```yaml
code_processing:
  enabled: true
  strategies:
    - language: rust
      enable_syntax_validation: true
      enable_formatting: true
```

## Testing

All tests pass successfully:
- 20 tests in `rust_strategy.rs` (including 3 new tests for italic comments)
- 5 tests in `generator.rs` for tag parsing (including 3 new tests for italic tags)

## Example Output

Test files created:
- `test_rust_italic_comments.md` → `test_rust_italic_comments.docx` (25KB)
- `test_rust_formatting_demo.md` → `test_rust_formatting_demo.docx` (62KB)

Both files demonstrate the feature with various Rust code examples showing bold keywords and italic comments.

## Technical Details

### Tag Processing Order
1. Comments are tagged with `[ITALIC]` first
2. Keywords are then tagged with `[BOLD]`
3. The generator processes tags in order of appearance
4. This ensures no conflicts between bold and italic formatting

### Comment Detection
- Uses simple string search for `//` to detect line comments
- Preserves everything after `//` as part of the comment
- Works with both standalone and inline comments
- Does not interfere with `//` in string literals (handled by syntax context)

### Rendering
- `[BOLD]` tags → Bold runs in docx
- `[ITALIC]` tags → Italic runs in docx
- Both can coexist in the same code block
- Proper font styling is applied based on code block configuration
