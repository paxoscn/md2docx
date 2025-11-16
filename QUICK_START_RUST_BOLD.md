# Rust 关键词粗体功能 - 快速开始

## 功能已实现 ✓

Rust 代码块中的关键词现在会自动以粗体显示在生成的 Word 文档中！

## 快速测试

### 1. 运行示例程序

```bash
# 查看关键词粗体效果
cargo run --example rust_keyword_bold_example

# 生成包含粗体关键词的 Word 文档
cargo run --example test_conversion_with_bold
```

生成的文档位于: `~/Downloads/test_rust_bold.docx`

### 2. 在你的代码中使用

```rust
use md2docx_converter::markdown::{
    MarkdownParser,
    code_block::{CodeBlockConfig, LanguageConfig}
};

// 配置代码块处理
let mut code_config = CodeBlockConfig::default();
code_config.global.enable_processing = true;

// 为 Rust 启用格式化（自动应用关键词粗体）
let mut rust_config = LanguageConfig::default();
rust_config.enable_formatting = true;
rust_config.enable_syntax_validation = true;
code_config.languages.insert("rust".to_string(), rust_config);

// 创建解析器
let parser = MarkdownParser::with_code_block_config(code_config);

// 解析 Markdown
let doc = parser.parse(markdown)?;

// 生成 Word 文档
let mut generator = DocxGenerator::new(ConversionConfig::default());
let docx_bytes = generator.generate(&doc)?;
```

### 3. 效果展示

**输入的 Markdown:**
````markdown
```rust
fn main() {
    let x: i32 = 42;
    if x > 0 {
        println!("positive");
    }
}
```
````

**在 Word 文档中显示为:**
- **fn** main() {
- **let** x: **i32** = 42;
- **if** x > 0 {

所有关键词（fn, let, i32, if 等）都会以粗体显示！

## 验证功能

运行测试确认一切正常：

```bash
# 运行所有 Rust 策略测试
cargo test --lib rust_strategy::tests

# 运行关键词粗体测试
cargo test --lib rust_strategy::tests::test_keyword_bold
```

所有测试应该都通过 ✓

## 支持的关键词

- 基本关键词: fn, let, if, match, for, while, etc.
- 类型: i32, String, Vec, Option, Result, etc.
- 访问控制: pub, crate, super, self, Self
- 异步: async, await
- 更多... (70+ 个关键词)

## 文档

- [详细使用指南](docs/HOW_TO_USE_RUST_KEYWORD_BOLD.md)
- [功能文档](docs/RUST_KEYWORD_BOLD.md)
- [实现总结](RUST_KEYWORD_BOLD_FEATURE.md)

## 问题排查

如果关键词没有显示为粗体：

1. ✓ 确认启用了代码块处理: `enable_processing = true`
2. ✓ 确认启用了格式化: `enable_formatting = true`
3. ✓ 确认代码块语言标识为 `rust` 或 `rs`
4. ✓ 检查代码语法是否正确

## 下一步

- 查看 `examples/rust_keyword_bold_example.rs` 了解更多用法
- 查看 `examples/test_conversion_with_bold.rs` 了解完整转换流程
- 阅读详细文档了解所有支持的关键词和配置选项
