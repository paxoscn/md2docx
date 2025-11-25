# BR 标签支持功能

## 功能概述

现在 md2docx 支持在 Markdown 文档中使用 `<br />` 标签来创建空行。当遇到 `<br />` 标签时，会在生成的 Word 文档中显示一个空行。

## 实现原理

1. **解析阶段**: pulldown-cmark 解析器会将 `<br />` 标签识别为 `HardBreak` 事件
2. **转换阶段**: `HardBreak` 事件被转换为换行符 `\n`
3. **生成阶段**: 在生成 Word 文档时，检测段落内容中的换行符，并将其拆分为多个段落，在换行符处插入空段落

## 使用示例

### 基本用法

```markdown
这是第一行<br />这是第二行
```

生成的 Word 文档中会显示：
```
这是第一行

这是第二行
```

### 多个 BR 标签

```markdown
第一行<br />第二行<br />第三行
```

生成的 Word 文档中会显示：
```
第一行

第二行

第三行
```

### 与格式化文本结合

```markdown
这是**粗体文本**<br />这是*斜体文本*<br />这是`代码文本`
```

生成的 Word 文档中会保留格式化，并在每个 BR 标签处插入空行。

### 连续的 BR 标签

```markdown
第一行<br /><br />第三行（中间有空行）
```

生成的 Word 文档中会显示：
```
第一行


第三行（中间有空行）
```

## 技术细节

### 修改的文件

- `src/docx/generator.rs`: 修改了 `add_paragraph` 方法，添加了 `add_single_paragraph` 和 `add_empty_paragraph` 辅助方法

### 核心逻辑

```rust
fn add_paragraph(&self, mut docx: Docx, content: &[InlineElement]) -> Result<Docx, ConversionError> {
    // 检查内容是否包含硬换行符（来自 <br /> 标签）
    let has_hard_breaks = content.iter().any(|inline| {
        matches!(inline, InlineElement::Text(text) if text.contains('\n'))
    });

    if has_hard_breaks {
        // 按硬换行符拆分内容并创建单独的段落
        // 在每个换行符处插入空段落
        // ...
    } else {
        // 没有硬换行符，作为单个段落添加
        docx = self.add_single_paragraph(docx, content)?;
    }
    
    Ok(docx)
}
```

### 空段落的实现

空段落使用不间断空格（`\u{00A0}`）和小字号来实现，以确保在 Word 文档中正确显示：

```rust
fn add_empty_paragraph(&self, mut docx: Docx) -> Result<Docx, ConversionError> {
    let paragraph = Paragraph::new()
        .add_run(Run::new().add_text("\u{00A0}")) // 不间断空格
        .size(1); // 小字号
    
    docx = docx.add_paragraph(paragraph);
    Ok(docx)
}
```

## 测试

添加了以下测试用例来验证功能：

1. `test_paragraph_with_hard_breaks`: 测试基本的硬换行符功能
2. `test_paragraph_with_hard_breaks_and_formatting`: 测试硬换行符与格式化文本的结合
3. `test_paragraph_with_multiple_consecutive_hard_breaks`: 测试多个连续的硬换行符
4. `test_paragraph_without_hard_breaks`: 测试正常段落（无硬换行符）

所有测试都通过，确保功能正常工作且不影响现有功能。

## 兼容性

- 该功能与现有的 Markdown 解析和 Word 文档生成功能完全兼容
- 不会影响正常段落的处理
- 支持与其他内联元素（粗体、斜体、代码、链接等）的组合使用

## 示例文件

- `test_br_tag.md`: 基本测试文件
- `test_br_comprehensive.md`: 全面测试文件，包含各种使用场景
