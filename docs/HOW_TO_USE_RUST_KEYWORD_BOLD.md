# 如何使用 Rust 关键词粗体功能

## 功能说明

Rust 关键词粗体功能会自动将 Rust 代码块中的关键词（如 `fn`, `let`, `if`, `struct` 等）以及常用类型（如 `i32`, `String`, `Vec` 等）用粗体显示在生成的 Word 文档中。

## 使用方法

### 方法 1: 使用代码配置（推荐）

如果你需要在代码中使用这个功能，可以通过配置 `CodeBlockConfig` 来启用：

```rust
use md2docx_converter::{
    config::ConversionConfig,
    markdown::{MarkdownParser, code_block::{CodeBlockConfig, LanguageConfig}},
    docx::DocxGenerator,
};

// 创建代码块处理配置
let mut code_config = CodeBlockConfig::default();
code_config.global.enable_processing = true;

// 为 Rust 启用格式化（这会自动应用关键词粗体）
let mut rust_config = LanguageConfig::default();
rust_config.enable_formatting = true;
rust_config.enable_syntax_validation = true;
code_config.languages.insert("rust".to_string(), rust_config);

// 创建带有代码块处理的解析器
let parser = MarkdownParser::with_code_block_config(code_config);

// 解析和转换
let doc = parser.parse(&markdown)?;
let mut generator = DocxGenerator::new(ConversionConfig::default());
let docx_bytes = generator.generate(&doc)?;
```

### 方法 2: 使用 YAML 配置文件

创建一个配置文件（例如 `config.yaml`）：

```yaml
# 其他配置...

# 代码块处理配置
code_block_processing:
  global:
    enable_processing: true
    default_timeout_ms: 5000
  
  languages:
    rust:
      enable_syntax_validation: true
      enable_formatting: true  # 启用格式化会自动应用关键词粗体
```

然后使用 CLI 工具：

```bash
./target/debug/md2docx-cli convert \
  -i input.md \
  -o output.docx \
  -c config.yaml
```

### 方法 3: 直接使用 CLI（使用默认配置）

**注意**: 默认配置目前不会启用代码块处理，所以需要使用方法 1 或 2。

## 示例

### 输入 Markdown

````markdown
# Rust 示例

```rust
fn main() {
    let x: i32 = 42;
    if x > 0 {
        println!("positive");
    }
}

pub struct Point {
    pub x: f64,
    pub y: f64,
}
```
````

### 处理后的代码（在 Word 文档中）

代码块中的关键词会以粗体显示：

- **fn** main()
- **let** x: **i32**
- **if** x > 0
- **pub** **struct** Point
- **pub** x: **f64**

## 运行示例

项目包含了完整的示例程序：

```bash
# 运行关键词粗体示例
cargo run --example rust_keyword_bold_example

# 运行完整转换示例（生成 Word 文档）
cargo run --example test_conversion_with_bold
```

## 支持的关键词

功能支持 70+ 个 Rust 关键词和类型，包括：

- **基本关键词**: fn, let, if, match, for, while, loop, etc.
- **类型关键词**: struct, enum, trait, impl, type, etc.
- **访问控制**: pub, crate, super, self, Self
- **异步关键词**: async, await
- **基本类型**: i32, u64, f64, bool, char, str, etc.
- **标准库类型**: String, Vec, Option, Result, Box, Arc, etc.

完整列表请参考 [RUST_KEYWORD_BOLD.md](./RUST_KEYWORD_BOLD.md)

## 注意事项

1. **必须启用格式化**: 关键词粗体功能只在启用 `enable_formatting` 时才会应用
2. **语法验证**: 如果代码语法无效且启用了语法验证，格式化将不会应用
3. **Markdown 兼容**: 功能使用 Markdown 粗体语法 (`**text**`)，在 Word 文档中会正确渲染为粗体
4. **性能**: 对于大型代码块，关键词替换可能会有轻微的性能影响

## 故障排除

### 问题: 关键词没有显示为粗体

**解决方案**:
1. 确认已启用代码块处理: `code_config.global.enable_processing = true`
2. 确认已为 Rust 启用格式化: `rust_config.enable_formatting = true`
3. 确认代码块的语言标识为 `rust` 或 `rs`
4. 检查代码语法是否有效（如果启用了语法验证）

### 问题: 转换失败

**解决方案**:
1. 检查 Rust 代码语法是否正确
2. 查看错误日志了解详细信息
3. 尝试禁用语法验证: `rust_config.enable_syntax_validation = false`

## 更多信息

- [功能详细文档](./RUST_KEYWORD_BOLD.md)
- [实现总结](../RUST_KEYWORD_BOLD_FEATURE.md)
- [代码块策略 API](./CODE_BLOCK_STRATEGY_API.md)
