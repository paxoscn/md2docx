# Note Strategy 使用指南

## 概述

Note Strategy 是一个专门用于处理 `note` 类型代码块的策略。它会自动将第一行文字放大、加粗、倾斜，并在表格右上角插入一个小提示图标。

## 功能特性

- **第一行特殊格式化**：第一行文字会被设置为 1.2 倍字体大小、加粗、倾斜
- **提示图标**：在内容区域右上角显示一个小图标（默认使用 `default-qrcode.png`）
- **表格布局**：使用 HTML 表格结构，带有圆角边框和浅色背景
- **多语言别名支持**：支持 `note`、`notes`、`tip`、`tips`、`hint` 等别名

## 使用方法

### 基本用法

在 Markdown 文档中使用 `note` 代码块：

\`\`\`note
重要提示
这是一个重要的注意事项，需要特别关注。
可以包含多行内容。
\`\`\`

### 使用别名

也可以使用其他支持的别名：

\`\`\`tip
专业建议
始终在提交代码前进行测试。
\`\`\`

\`\`\`hint
小提示
使用快捷键可以提高工作效率。
\`\`\`

## 代码集成

### 注册策略

```rust
use markdown_converter::markdown::code_block::{
    StrategyRegistry, NoteStrategy
};
use std::sync::Arc;

// 创建注册表
let mut registry = StrategyRegistry::new();

// 注册 Note 策略（使用默认图标）
let note_strategy = Arc::new(NoteStrategy::new());
registry.register_strategy(note_strategy);

// 或者使用自定义图标
let custom_note_strategy = Arc::new(
    NoteStrategy::with_icon_path("path/to/custom-icon.png".to_string())
);
registry.register_strategy(custom_note_strategy);
```

### 处理代码块

```rust
use markdown_converter::markdown::code_block::{
    ProcessingConfig, NoteStrategy
};

let strategy = NoteStrategy::new();
let config = ProcessingConfig::default()
    .with_formatting(true);

let note_content = "重要提示\n这是注意事项的内容。";
let result = strategy.process(note_content, &config)?;

if let Some(formatted) = result.processed_code {
    println!("格式化后的内容：\n{}", formatted);
}
```

## 输出示例

输入：
```
重要提示
这是一个需要注意的事项。
请仔细阅读。
```

输出（内部格式）：
```
[NOTE_BLOCK_START]
[TITLE]重要提示[/TITLE]
[ICON]default-qrcode.png[/ICON]
[CONTENT]
这是一个需要注意的事项。
请仔细阅读。
[/CONTENT]
[NOTE_BLOCK_END]
```

在 DOCX 中的渲染效果：
- 第一行"重要提示"显示为：**加粗**、*倾斜*、1.2 倍字体大小（约 14pt）
- 右上角显示一个小图标（💡 emoji 作为占位符）
- 整体使用两列表格布局，带有边框和间距
- 左列包含标题和内容，右列包含图标

## 配置选项

### 自定义图标路径

```rust
let mut strategy = NoteStrategy::new();
strategy.set_icon_path("custom-icon.svg".to_string());
```

### 处理配置

```rust
let config = ProcessingConfig::default()
    .with_formatting(true);  // 启用格式化
```

## 元数据

处理后的代码块包含以下元数据：

- `language`: "note"
- `formatter`: "note_formatter"
- `icon_path`: 图标文件路径
- `is_formatted`: 是否已格式化
- `processing_time`: 处理耗时

## 注意事项

1. **DOCX 输出**：此策略生成特殊标记格式，由 DOCX 生成器解析并渲染为格式化的表格
2. **图标显示**：当前版本使用 emoji 💡 作为图标占位符，未来版本将支持实际图片嵌入
3. **格式化要求**：必须启用格式化（`with_formatting(true)`）才能生成特殊格式
4. **第一行为空**：如果第一行为空，将被跳过，不会显示空的标题

## 测试

运行测试：

```bash
cargo test note_strategy
```

## 优先级

Note Strategy 的优先级为 120（中高优先级），确保它能正确处理 note 类型的代码块。

## 版本

当前版本：1.0.0
