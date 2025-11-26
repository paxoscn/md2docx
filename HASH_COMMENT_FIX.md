# 代码块中 # 注释符号显示修复

## 问题描述

在代码块中，`#` 注释符号（如 bash、python、shell 等语言的注释）在生成的 Word 文档中无法显示。

**问题示例：**
```bash
# 调试构建
cargo build
# 生产构建
cargo build --release
```

在生成的 docx 文件中，`#` 符号及其后的注释文本会消失，只显示 `cargo build` 等命令。

## 问题原因

在 `src/docx/generator.rs` 的 `create_code_paragraph_with_markdown` 函数中，代码会将代码块的每一行都使用完整的 Markdown 解析器进行解析。这导致 `#` 被识别为 Markdown 标题标记（heading），而不是普通文本。

原有实现：
```rust
// Parse the text as Markdown
let parser = crate::markdown::MarkdownParser::new();
let document = parser.parse(text_without_leading_spaces).map_err(|e| {
    ConversionError::DocxGeneration(format!("Failed to parse Markdown in code block: {}", e))
})?;
```

## 解决方案

修改 `create_code_paragraph_with_markdown` 函数，使用自定义的内联格式解析器，只处理特定的内联格式（如 `**bold**` 和 `*italic*`），而不是完整的 Markdown 解析。这样 `#` 等特殊字符在代码块中会被保持原样。

新增了 `parse_inline_formatting` 函数：
```rust
/// Parse inline formatting (bold, italic) without treating # as heading
/// This is a simple parser that only handles ** and * for bold/italic
fn parse_inline_formatting(&self, text: &str) -> Vec<crate::markdown::InlineElement> {
    // 只解析 ** 和 * 标记，不解析 # 等其他 Markdown 语法
    // ...
}
```

## 修改的文件

- `src/docx/generator.rs`
  - 修改 `create_code_paragraph_with_markdown` 函数
  - 新增 `parse_inline_formatting` 函数

## 测试验证

### 1. 功能测试

创建了测试文件 `test_hash_fix.md`，包含多种语言的 `#` 注释：
- Bash 脚本注释
- Python 注释
- Shell 脚本 shebang 和注释
- 混合格式（注释 + 粗体/斜体）

测试结果：
```bash
cargo run --release --bin md2docx-cli -- convert --input test_hash_fix.md --output test_hash_fix.docx
# ✓ Conversion completed successfully

# 验证 # 符号正确显示
unzip -p test_hash_fix.docx word/document.xml | grep -o "# [^<]*" | head -10
# 输出：
# # 符号修复
# # 这是注释
# # 调试构建
# # 生产构建
# ...
```

### 2. 单元测试

新增了 6 个单元测试：
- `test_parse_inline_formatting_plain_text` - 测试纯文本（包含 #）
- `test_parse_inline_formatting_bold` - 测试粗体格式
- `test_parse_inline_formatting_italic` - 测试斜体格式
- `test_parse_inline_formatting_mixed` - 测试混合格式
- `test_parse_inline_formatting_unclosed_markers` - 测试未闭合的标记
- `test_code_block_with_hash_comments` - 测试包含 # 注释的代码块

所有测试通过：
```bash
cargo test --lib test_parse_inline_formatting --release
# test result: ok. 5 passed; 0 failed; 0 ignored

cargo test --lib test_code_block_with_hash_comments --release
# test result: ok. 1 passed; 0 failed; 0 ignored
```

### 3. 回归测试

所有现有测试通过，无回归问题：
```bash
cargo test --lib docx::generator::tests --release
# test result: ok. 68 passed; 0 failed; 0 ignored
```

## 影响范围

- 修复了代码块中 `#` 注释符号无法显示的问题
- 保留了代码块中 `**bold**` 和 `*italic*` 格式的支持
- 不影响其他 Markdown 元素的解析
- 所有现有测试通过，无回归问题

## 使用示例

修复后，以下代码块可以正确显示：

```bash
# 调试构建
cargo build

# 生产构建
cargo build --release
```

```python
# 这是 Python 注释
def hello():
    # 函数内注释
    print("Hello")
```

```sh
#!/bin/bash
# 脚本注释
echo "Hello"
```
