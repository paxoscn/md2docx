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

输出（HTML）：
```html
<table style="width: 100%; border: 1px solid #e0e0e0; border-radius: 8px; padding: 16px; background-color: #f8f9fa;">
<tr>
<td style="vertical-align: top;">

<span style="font-size: 1.2em; font-weight: bold; font-style: italic;">重要提示</span>

这是一个需要注意的事项。
请仔细阅读。
</td>
<td style="width: 48px; vertical-align: top; text-align: right;">
<img src="default-qrcode.png" alt="Tip" style="width: 32px; height: 32px; opacity: 0.7;" />
</td>
</tr>
</table>
```

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

1. **HTML 输出**：此策略生成 HTML 格式的输出，确保你的 Markdown 渲染器支持内联 HTML
2. **图标路径**：确保图标文件路径正确，相对于最终 HTML 文档的位置
3. **样式兼容性**：生成的 HTML 使用内联样式，在大多数环境中都能正常显示
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
