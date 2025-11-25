# H1 标题分页功能

## 功能概述

现在，所有一级标题（`# 标题`）都会自动在新页面的开头显示。这个功能对于创建结构化文档（如书籍章节、报告等）非常有用。

## 实现细节

### 修改位置
- 文件：`src/docx/generator.rs`
- 方法：`add_heading()`

### 实现方式
在 `add_heading` 方法中，当检测到一级标题（level == 1）时，会在标题前插入一个分页符：

```rust
// For level 1 headings, add a page break before the heading
if level == 1 {
    let page_break = Paragraph::new().add_run(Run::new().add_break(BreakType::Page));
    docx = docx.add_paragraph(page_break);
}
```

## 使用示例

### Markdown 输入
```markdown
# 第一章

这是第一章的内容。

## 1.1 小节

这是第一章第一小节的内容。

# 第二章

这是第二章的内容。这个标题会出现在新的一页。

## 2.1 小节

这是第二章第一小节的内容。
```

### 效果
- **第一章**：出现在新页面开头
- **第二章**：出现在新页面开头
- 二级标题（##）和更低级别的标题不会触发分页

## 行为说明

1. **仅对一级标题生效**：只有使用单个 `#` 的一级标题会触发分页
2. **自动分页**：无需手动插入分页符
3. **保持格式**：标题的其他格式设置（字体、大小、对齐等）保持不变
4. **智能处理第一个标题**：文档中的第一个一级标题不会触发分页，从第二个一级标题开始才会在新页面显示

## 测试文件

已创建以下测试文件来验证功能：
- `test_h1_page_break.md` - 基本测试
- `test_h1_page_break_detailed.md` - 详细测试，包含多个章节

## 转换命令

```bash
./target/release/md2docx-cli convert --input test_h1_page_break.md --output test_h1_page_break.docx
```

## 注意事项

1. **第一个标题处理**：文档中的第一个一级标题不会触发分页，这样可以避免第一页为空的情况。

2. **配置选项**：目前这个功能是硬编码的。如果需要可配置（例如，允许用户选择是否启用此功能），可以在配置文件中添加相应选项。

## 可能的改进

1. **可配置性**：在 YAML 配置文件中添加选项来控制是否启用一级标题分页
2. **其他级别支持**：允许用户配置哪些级别的标题需要分页
3. **条件分页**：允许用户通过特殊标记来控制某些一级标题是否需要分页

## 相关代码

### 结构体定义
```rust
pub struct DocxGenerator {
    config: ConversionConfig,
    heading_processor: Option<HeadingProcessor>,
    /// Track if we've encountered the first H1 heading to avoid page break before it
    first_h1_encountered: bool,
}
```

### 分页逻辑
```rust
/// Add a heading to the document
fn add_heading(
    &mut self,
    mut docx: Docx,
    level: u8,
    text: &str,
) -> Result<Docx, ConversionError> {
    // ... 省略其他代码 ...
    
    // For level 1 headings, add a page break before the heading (except for the first H1)
    if level == 1 {
        if self.first_h1_encountered {
            // Add page break for subsequent H1 headings
            let page_break = Paragraph::new().add_run(Run::new().add_break(BreakType::Page));
            docx = docx.add_paragraph(page_break);
        } else {
            // Mark that we've encountered the first H1
            self.first_h1_encountered = true;
        }
    }
    
    // ... 省略其他代码 ...
}
```

### 状态重置
```rust
pub fn generate(&mut self, document: &MarkdownDocument) -> Result<Vec<u8>, ConversionError> {
    // Reset state at the beginning of document generation
    self.first_h1_encountered = false;
    
    // ... 省略其他代码 ...
}
```
