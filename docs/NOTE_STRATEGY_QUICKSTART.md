# Note Strategy 快速入门

## 5 分钟上手指南

### 1. 基本使用

在你的 Markdown 文档中，使用 `note` 代码块：

````markdown
```note
重要提示
这是一个需要特别注意的内容。
```
````

### 2. 渲染效果

第一行"重要提示"会被渲染为：
- **加粗**
- *倾斜*
- 1.2 倍字体大小

同时，在内容区域的右上角会显示一个小提示图标。

### 3. 支持的别名

除了 `note`，你还可以使用：

````markdown
```tip
专业建议
使用快捷键提高效率。
```

```hint
小提示
记得保存你的工作。
```
````

### 4. 代码集成

#### 注册策略

```rust
use md2docx_converter::markdown::code_block::{
    StrategyRegistry, NoteStrategy
};
use std::sync::Arc;

let mut registry = StrategyRegistry::new();
registry.register_strategy(Arc::new(NoteStrategy::new()));
```

#### 处理内容

```rust
use md2docx_converter::markdown::code_block::{
    NoteStrategy, ProcessingConfig, CodeBlockStrategy
};

let strategy = NoteStrategy::new();
let config = ProcessingConfig::default().with_formatting(true);

let content = "重要\n这是内容。";
let result = strategy.process(content, &config)?;

if let Some(html) = result.processed_code {
    println!("{}", html);
}
```

### 5. 自定义图标

```rust
let strategy = NoteStrategy::with_icon_path("my-icon.svg".to_string());
```

或者动态修改：

```rust
let mut strategy = NoteStrategy::new();
strategy.set_icon_path("custom-icon.png".to_string());
```

## 完整示例

```rust
use md2docx_converter::markdown::code_block::{
    StrategyRegistry, NoteStrategy, ProcessingConfig, CodeBlockStrategy
};
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 创建并注册策略
    let mut registry = StrategyRegistry::new();
    let note_strategy = Arc::new(NoteStrategy::new());
    registry.register_strategy(note_strategy);
    
    // 2. 获取策略
    let strategy = registry.get_strategy("note");
    
    // 3. 配置处理选项
    let config = ProcessingConfig::default()
        .with_formatting(true);
    
    // 4. 处理内容
    let content = "安全警告\n请勿在代码中硬编码密码。";
    let result = strategy.process(content, &config)?;
    
    // 5. 使用结果
    if let Some(formatted) = result.processed_code {
        println!("格式化后的 HTML：\n{}", formatted);
    }
    
    Ok(())
}
```

## 输出示例

输入：
```
重要提示
这是一个重要的注意事项。
```

输出（内部格式）：
```
[NOTE_BLOCK_START]
[TITLE]重要提示[/TITLE]
[ICON]default-qrcode.png[/ICON]
[CONTENT]
这是一个重要的注意事项。
[/CONTENT]
[NOTE_BLOCK_END]
```

在 DOCX 中显示为：
- 标题：**重要提示**（加粗、倾斜、放大）
- 内容：这是一个重要的注意事项。
- 图标：💡（右上角）

## 常见问题

### Q: 如何更改图标？
A: 使用 `with_icon_path()` 或 `set_icon_path()` 方法。

### Q: 支持哪些语言别名？
A: `note`, `notes`, `tip`, `tips`, `hint`（不区分大小写）

### Q: 可以禁用格式化吗？
A: 可以，设置 `config.with_formatting(false)`

### Q: 第一行为空会怎样？
A: 会被跳过，不会显示空标题。

## 下一步

- 查看 [完整文档](NOTE_STRATEGY.md)
- 运行 [示例代码](../examples/note_strategy_example.rs)
- 查看 [测试用例](../tests/note_strategy_integration_test.rs)

## 运行测试

```bash
# 运行单元测试
cargo test note_strategy --lib

# 运行集成测试
cargo test --test note_strategy_integration_test

# 运行示例
cargo run --example note_strategy_example
```
