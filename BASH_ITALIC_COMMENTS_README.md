# Bash 注释斜体功能

## 概述

本功能为 Bash/Shell 代码块添加了注释斜体渲染支持。当处理 Bash 代码块时，所有以 `#` 开头的注释（包括 shebang 和行内注释）都会被自动标记为斜体格式。

## 功能特性

- ✅ 支持完整行注释（以 `#` 开头的行）
- ✅ 支持行内注释（代码后的 `#` 注释）
- ✅ 支持 shebang（`#!/bin/bash`）
- ✅ 支持多种 Shell 语言别名：`bash`, `sh`, `shell`, `zsh`
- ✅ 自动格式化处理
- ✅ 代码质量检查（如 `rm -rf` 警告）

## 实现细节

### 策略类

`BashStrategy` 实现了 `CodeBlockStrategy` 和 `LanguageStrategy` trait，提供以下功能：

1. **注释检测**：识别以 `#` 开头的注释
2. **斜体标记**：使用 `[ITALIC]...[/ITALIC]` 标记注释
3. **格式保持**：保持原始代码的缩进和结构
4. **质量检查**：检查常见的代码问题

### 支持的语言

- `bash`
- `sh`
- `shell`
- `zsh`

## 使用方法

### 1. 在 Markdown 中使用

```markdown
\`\`\`bash
#!/bin/bash
# 这是一个注释
echo "Hello World" # 行内注释
\`\`\`
```

### 2. 程序化使用

```rust
use md2docx_converter::markdown::code_block::{
    strategies::BashStrategy,
    CodeBlockStrategy,
    ProcessingConfig,
};

let strategy = BashStrategy::new();
let config = ProcessingConfig::default().with_formatting(true);

let code = r#"#!/bin/bash
# 这是一个注释
echo "Hello" # 行内注释
"#;

let processed = strategy.process(code, &config).unwrap();
```

### 3. 命令行使用

```bash
cargo run --release --bin md2docx-cli -- convert \
    --input input.md \
    --output output.docx \
    --config config_with_code_processing.yaml
```

## 配置

在 YAML 配置文件中启用代码块处理：

```yaml
code_blocks:
  enable_processing: true
  languages:
    bash:
      enable_formatting: true
      enable_syntax_validation: false
```

## 示例

### 输入

```bash
#!/bin/bash
# 脚本功能：备份文件
# 作者：测试

SOURCE_DIR="/home/user/documents"
BACKUP_DIR="/backup" # 备份目录

# 创建备份目录
mkdir -p "$BACKUP_DIR"

# 执行备份
tar -czf "$BACKUP_DIR/backup-$(date +%Y%m%d).tar.gz" "$SOURCE_DIR" # 压缩备份

echo "备份完成" # 输出消息
```

### 输出（带斜体标记）

```
[ITALIC]#!/bin/bash[/ITALIC]
[ITALIC]# 脚本功能：备份文件[/ITALIC]
[ITALIC]# 作者：测试[/ITALIC]

SOURCE_DIR="/home/user/documents"
BACKUP_DIR="/backup" [ITALIC]# 备份目录[/ITALIC]

[ITALIC]# 创建备份目录[/ITALIC]
mkdir -p "$BACKUP_DIR"

[ITALIC]# 执行备份[/ITALIC]
tar -czf "$BACKUP_DIR/backup-$(date +%Y%m%d).tar.gz" "$SOURCE_DIR" [ITALIC]# 压缩备份[/ITALIC]

echo "备份完成" [ITALIC]# 输出消息[/ITALIC]
```

## 测试

运行测试：

```bash
# 运行单元测试
cargo test bash_strategy --lib

# 运行示例程序
cargo run --release --example bash_italic_comments_example

# 测试文档转换
cargo run --release --bin md2docx-cli -- convert \
    --input test_bash_italic_comments.md \
    --output test_bash_italic_comments.docx \
    --config config_with_code_processing.yaml
```

## 文件结构

```
src/markdown/code_block/strategies/
├── mod.rs                    # 策略模块导出
├── rust_strategy.rs          # Rust 策略（参考实现）
├── bash_strategy.rs          # Bash 策略（新增）
└── note_strategy.rs          # Note 策略

examples/
└── bash_italic_comments_example.rs  # 使用示例

tests/
└── test_bash_italic_comments.md     # 测试文档
```

## 技术实现

### 注释检测算法

```rust
fn apply_comment_italic(&self, code: &str) -> String {
    let mut result = String::new();
    
    for line in code.lines() {
        // 检查是否包含注释
        if let Some(comment_pos) = line.find('#') {
            // 添加注释前的代码部分
            result.push_str(&line[..comment_pos]);
            // 添加带斜体标记的注释
            result.push_str("[ITALIC]");
            result.push_str(&line[comment_pos..]);
            result.push_str("[/ITALIC]");
        } else {
            // 无注释，直接添加
            result.push_str(line);
        }
        result.push('\n');
    }
    
    // 移除多余的尾部换行符
    if !code.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }
    
    result
}
```

## 与 Rust 策略的对比

| 特性 | Rust 策略 | Bash 策略 |
|------|----------|----------|
| 注释斜体 | ✅ (`//` 注释) | ✅ (`#` 注释) |
| 关键字加粗 | ✅ | ❌ |
| 语法验证 | ✅ (使用 syn) | ❌ |
| 代码格式化 | ✅ | ✅ (基础) |
| 质量检查 | ✅ | ✅ (基础) |

## 未来改进

- [ ] 添加 Bash 语法验证（使用 shellcheck）
- [ ] 支持更多 Shell 变体（fish, csh 等）
- [ ] 添加关键字高亮
- [ ] 改进代码质量检查规则
- [ ] 支持多行注释（heredoc 中的注释）

## 参考

- [Rust 策略实现](src/markdown/code_block/strategies/rust_strategy.rs)
- [代码块策略设计文档](.kiro/specs/code-block-strategy/design.md)
- [Bash 注释语法](https://www.gnu.org/software/bash/manual/bash.html#Comments)
