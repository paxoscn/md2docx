# Note Strategy 更新说明

## 问题

初始实现生成 HTML 标签，但在 DOCX 中被当作纯文本显示，而不是被渲染成格式化的内容。

## 解决方案

修改 Note Strategy 生成特殊的标记格式，由 DOCX 生成器解析并渲染为格式化的表格。

## 实现变更

### 1. Note Strategy 输出格式

**之前（HTML）：**
```html
<table style="...">
  <tr>
    <td><span style="font-weight: bold; ...">标题</span>内容</td>
    <td><img src="icon.png" /></td>
  </tr>
</table>
```

**现在（标记格式）：**
```
[NOTE_BLOCK_START]
[TITLE]标题[/TITLE]
[ICON]icon_path[/ICON]
[CONTENT]
内容行1
内容行2
[/CONTENT]
[NOTE_BLOCK_END]
```

### 2. DOCX 生成器增强

在 `src/docx/generator.rs` 中添加了两个新方法：

#### `add_note_block()`
- 检测并解析 note 块标记
- 创建两列表格布局
- 左列：标题（加粗、倾斜、放大）+ 内容
- 右列：图标（当前使用 💡 emoji）

#### `extract_marker_content()`
- 辅助方法，用于提取标记之间的内容
- 支持 `[TITLE]...[/TITLE]`、`[ICON]...[/ICON]`、`[CONTENT]...[/CONTENT]`

### 3. 修改的文件

1. **src/markdown/code_block/strategies/note_strategy.rs**
   - 修改 `format_note_content()` 方法
   - 更新相关测试

2. **src/docx/generator.rs**
   - 修改 `add_code_block()` 方法，添加 note 块检测
   - 新增 `add_note_block()` 方法
   - 新增 `extract_marker_content()` 方法

3. **tests/note_strategy_integration_test.rs**
   - 更新测试断言，匹配新的输出格式

4. **tests/note_block_docx_test.rs**（新增）
   - 5 个 DOCX 生成测试
   - 验证 note 块正确渲染到 DOCX

5. **examples/note_to_docx_example.rs**（新增）
   - 完整的端到端示例
   - 生成包含多个 note 块的 DOCX 文件

6. **docs/NOTE_STRATEGY.md**
   - 更新输出示例
   - 更新注意事项

7. **docs/NOTE_STRATEGY_QUICKSTART.md**
   - 更新输出示例

## DOCX 渲染效果

在生成的 DOCX 文件中，note 块显示为：

```
┌─────────────────────────────────────────┬────┐
│ 重要提示（加粗、倾斜、14pt）              │ 💡 │
│                                         │    │
│ 这是内容的第一行。                       │    │
│ 这是内容的第二行。                       │    │
└─────────────────────────────────────────┴────┘
```

特点：
- 使用表格布局，带边框
- 标题：加粗 + 倾斜 + 1.2 倍字体（约 14pt）
- 图标：右上角对齐，使用 emoji 💡
- 内容：正常字体，保留换行

## 测试结果

所有测试通过：
- ✅ 单元测试：11/11
- ✅ 集成测试：18/18
- ✅ DOCX 测试：5/5
- ✅ 总计：34/34

## 使用示例

### 在 Markdown 中

````markdown
```note
重要提示
这是需要注意的内容。
```
````

### 在代码中

```rust
use md2docx_converter::markdown::code_block::CodeBlockProcessor;

let processor = CodeBlockProcessor::new();
let result = processor.process_code_block(
    "提示\n内容",
    Some("note")
)?;

// result.processed_code 包含标记格式
// DOCX 生成器会自动识别并渲染
```

### 生成 DOCX

```rust
use md2docx_converter::docx::DocxGenerator;
use md2docx_converter::markdown::ast::{MarkdownDocument, MarkdownElement};

let mut document = MarkdownDocument::new();
document.add_element(MarkdownElement::CodeBlock {
    language: Some("note".to_string()),
    code: "标题\n内容".to_string(),
    processed: Some(result),
});

let mut generator = DocxGenerator::new(config);
let docx = generator.generate(&document)?;
```

## 未来改进

1. **实际图片支持**
   - 当前使用 emoji 💡 作为占位符
   - 未来版本将支持嵌入实际图片文件

2. **可配置样式**
   - 标题字体大小
   - 表格边框样式
   - 背景颜色

3. **更多图标选项**
   - 不同类型的 note 使用不同图标
   - 支持自定义图标库

4. **主题支持**
   - 预定义的颜色主题
   - 信息、警告、错误等不同级别

## 兼容性

- ✅ 向后兼容：旧的 note 块仍然可以处理
- ✅ 混合内容：note 块和普通代码块可以共存
- ✅ 多语言别名：note、tip、hint 等都支持

## 总结

通过使用特殊标记格式而不是 HTML，成功解决了 DOCX 渲染问题。Note 块现在可以在 DOCX 中正确显示为格式化的表格，包含加粗倾斜的标题和图标。

---

**更新时间**: 2024-11-24  
**状态**: ✅ 已完成并测试
