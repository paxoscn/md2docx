# 代码块空格保留修复

## 问题描述

在格式化代码块时，每一行最前面的四个空格（或其他前导空格）被丢失了。

## 根本原因

问题出在 `src/docx/generator.rs` 文件中的 `create_code_paragraph_with_markdown` 函数。该函数会：

1. 将代码块的每一行作为 Markdown 文本进行解析
2. 在解析过程中，`src/markdown/parser.rs` 中的 `normalize_whitespace` 函数会被调用
3. `normalize_whitespace` 函数会将所有连续的空白字符（包括多个空格）压缩成单个空格
4. 这导致代码块中的缩进（通常是4个空格）被压缩成1个空格

## 解决方案

修改 `create_code_paragraph_with_markdown` 函数，使其不再将代码块内容作为 Markdown 解析，而是直接调用 `create_code_paragraph` 函数来保留原始的空白字符。

### 修改的文件

- `src/docx/generator.rs`

### 具体修改

```rust
// 修改前：
fn create_code_paragraph_with_markdown(
    &self,
    text: &str,
    style: &crate::config::CodeBlockStyle,
) -> Result<Paragraph, ConversionError> {
    // Parse the text as Markdown
    let parser = crate::markdown::MarkdownParser::new();
    let document = parser.parse(text).map_err(|e| {
        ConversionError::DocxGeneration(format!("Failed to parse Markdown in code block: {}", e))
    })?;
    // ... 复杂的 Markdown 解析逻辑 ...
}

// 修改后：
fn create_code_paragraph_with_markdown(
    &self,
    text: &str,
    style: &crate::config::CodeBlockStyle,
) -> Result<Paragraph, ConversionError> {
    // For code blocks, we should preserve whitespace exactly as-is
    // Don't parse as Markdown to avoid whitespace normalization
    // Just create a simple code paragraph with the text
    self.create_code_paragraph(text, style)
}
```

### 删除的代码

同时删除了以下不再使用的辅助函数：
- `create_code_run_from_inline`
- `create_code_run`
- `extract_text_from_element`
- `extract_text_from_inline`

以及相关的测试函数：
- `test_create_code_run_from_inline`
- `test_extract_text_from_element`

## 测试验证

创建了测试文件来验证修复：

1. `test_whitespace_preservation.md` - 基本的空格保留测试
2. `test_indentation_detailed.md` - 详细的缩进测试，包括：
   - 标准缩进（4个空格）
   - 不同级别的缩进
   - 混合空格和制表符
   - 保留空行

所有现有的单元测试都通过了：
```
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured
```

## 影响范围

这个修复只影响代码块的渲染，不会影响其他 Markdown 元素（如段落、标题等）的处理。

## 向后兼容性

这个修复提高了代码块的保真度，使其更准确地保留原始格式。对于依赖旧行为的用户来说，这是一个改进而不是破坏性变更。
