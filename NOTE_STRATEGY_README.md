# Note Strategy - Code Block 处理策略

## 简介

Note Strategy 是一个专门用于处理 `note` 类型代码块的策略实现。它会自动将第一行文字放大、加粗、倾斜，并在表格右上角插入一个小提示图标。

## 快速开始

### 在 Markdown 中使用

````markdown
```note
重要提示
这是一个需要特别注意的内容。
可以包含多行。
```
````

### 在代码中使用

```rust
use md2docx_converter::markdown::code_block::{
    StrategyRegistry, NoteStrategy, CodeBlockStrategy
};
use std::sync::Arc;

// 注册策略
let mut registry = StrategyRegistry::new();
registry.register_strategy(Arc::new(NoteStrategy::new()));

// 使用策略
let strategy = registry.get_strategy("note");
let config = ProcessingConfig::default().with_formatting(true);
let result = strategy.process("提示\n内容", &config)?;
```

## 主要特性

✓ **第一行特殊格式化** - 1.2 倍字体、加粗、倾斜  
✓ **提示图标** - 右上角显示小图标  
✓ **多语言别名** - 支持 note、tip、hint 等  
✓ **自定义图标** - 可配置图标路径  
✓ **HTML 输出** - 生成美观的表格布局  
✓ **完整测试** - 29 个测试全部通过  

## 文档

- [完整文档](docs/NOTE_STRATEGY.md) - 详细的功能说明和 API 文档
- [快速入门](docs/NOTE_STRATEGY_QUICKSTART.md) - 5 分钟上手指南
- [实现总结](NOTE_STRATEGY_IMPLEMENTATION.md) - 技术细节和实现说明

## 示例

- [代码示例](examples/note_strategy_example.rs) - 7 个实际使用示例
- [Markdown 示例](examples/note_example.md) - 10 个不同场景的示例

## 测试

```bash
# 运行单元测试（11 个）
cargo test note_strategy --lib

# 运行集成测试（18 个）
cargo test --test note_strategy_integration_test

# 运行所有相关测试
cargo test note_strategy
```

## 支持的语言别名

- `note` / `notes`
- `tip` / `tips`
- `hint`

所有别名不区分大小写。

## 输出示例

### 输入
```
重要提示
这是一个需要注意的内容。
```

### 输出（HTML）
```html
<table style="width: 100%; border: 1px solid #e0e0e0; border-radius: 8px; padding: 16px; background-color: #f8f9fa;">
<tr>
<td style="vertical-align: top;">
<span style="font-size: 1.2em; font-weight: bold; font-style: italic;">重要提示</span>

这是一个需要注意的内容。
</td>
<td style="width: 48px; vertical-align: top; text-align: right;">
<img src="default-qrcode.png" alt="Tip" style="width: 32px; height: 32px; opacity: 0.7;" />
</td>
</tr>
</table>
```

## 自定义配置

### 自定义图标路径

```rust
// 创建时指定
let strategy = NoteStrategy::with_icon_path("custom-icon.svg".to_string());

// 或者动态修改
let mut strategy = NoteStrategy::new();
strategy.set_icon_path("new-icon.png".to_string());
```

### 处理配置

```rust
let config = ProcessingConfig::default()
    .with_formatting(true);  // 启用格式化
```

## 技术规格

- **优先级**: 120（中高优先级）
- **版本**: 1.0.0
- **线程安全**: 是（实现了 Send + Sync）
- **性能**: 微秒级处理速度
- **依赖**: 仅标准库和项目内部模块

## 测试覆盖

### 单元测试（11 个）✓
- 策略创建和配置
- 内容格式化
- 图标路径管理
- 语言别名支持
- 元数据验证
- 输出结构验证

### 集成测试（18 个）✓
- 策略注册和获取
- 多策略共存
- 处理管道完整性
- 边界情况处理
- Unicode 和特殊字符支持
- 性能和元数据验证

## 项目结构

```
.
├── src/markdown/code_block/strategies/
│   ├── note_strategy.rs          # 核心实现
│   └── mod.rs                     # 模块导出
├── docs/
│   ├── NOTE_STRATEGY.md           # 完整文档
│   └── NOTE_STRATEGY_QUICKSTART.md # 快速入门
├── examples/
│   ├── note_strategy_example.rs   # 代码示例
│   └── note_example.md            # Markdown 示例
├── tests/
│   └── note_strategy_integration_test.rs # 集成测试
└── NOTE_STRATEGY_IMPLEMENTATION.md # 实现总结
```

## 贡献

欢迎提交 Issue 和 Pull Request！

## 许可证

与主项目保持一致。

---

**状态**: ✅ 已完成并通过所有测试  
**最后更新**: 2024-11-24
