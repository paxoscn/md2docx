// 测试完整的转换流程，包括关键词粗体

use md2docx_converter::{
    config::ConversionConfig,
    markdown::{MarkdownParser, code_block::{CodeBlockConfig, LanguageConfig}},
    docx::DocxGenerator,
};
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 读取测试文档
    let markdown = r#"# Rust 代码测试

这是一个测试文档，用于验证 Rust 关键词粗体功能。

```rust
fn main() {
    let x: i32 = 42;
    let s: String = String::from("hello");
    
    if x > 0 {
        println!("x is positive");
    }
}

pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}
```

上面的代码展示了 Rust 的基本语法。
"#;
    
    // 创建代码块处理配置
    let mut code_config = CodeBlockConfig::default();
    code_config.global.enable_processing = true;
    
    // 为 Rust 启用格式化
    let mut rust_config = LanguageConfig::default();
    rust_config.enable_formatting = true;
    rust_config.enable_syntax_validation = true;
    code_config.languages.insert("rust".to_string(), rust_config);
    
    // 创建带有代码块处理的解析器
    let parser = MarkdownParser::with_code_block_config(code_config);
    
    // 解析文档
    println!("解析 Markdown 文档...");
    let doc = parser.parse(&markdown)?;
    
    println!("解析成功！元素数量: {}", doc.elements.len());
    
    // 检查代码块
    for (i, element) in doc.elements.iter().enumerate() {
        if let Some(processed) = element.get_code_block_processed() {
            println!("\n=== 代码块 {} ===", i);
            println!("语言: {:?}", processed.language);
            println!("是否格式化: {}", processed.metadata.is_formatted);
            
            if let Some(formatted) = &processed.processed_code {
                println!("\n格式化后的代码片段:");
                let lines: Vec<&str> = formatted.lines().take(5).collect();
                for line in lines {
                    println!("  {}", line);
                }
                
                // 检查关键词
                let keywords = vec!["**fn**", "**let**", "**i32**", "**String**", "**if**", "**pub**", "**struct**", "**impl**"];
                let mut found_keywords = Vec::new();
                for keyword in keywords {
                    if formatted.contains(keyword) {
                        found_keywords.push(keyword);
                    }
                }
                
                println!("\n找到的粗体关键词: {:?}", found_keywords);
            }
        }
    }
    
    // 生成 Word 文档
    println!("\n生成 Word 文档...");
    let config = ConversionConfig::default();
    let mut generator = DocxGenerator::new(config);
    let docx_bytes = generator.generate(&doc)?;
    
    // 保存文件
    let output_path = "/Users/lindagao/Downloads/test_rust_bold.docx";
    fs::write(output_path, docx_bytes)?;
    
    println!("✓ 成功生成文档: {}", output_path);
    println!("  文件大小: {} 字节", fs::metadata(output_path)?.len());
    
    Ok(())
}
