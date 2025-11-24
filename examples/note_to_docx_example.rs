//! Example demonstrating Note Strategy with DOCX generation

use md2docx_converter::config::ConversionConfig;
use md2docx_converter::docx::DocxGenerator;
use md2docx_converter::markdown::ast::{MarkdownDocument, MarkdownElement, InlineElement};
use md2docx_converter::markdown::code_block::{
    StrategyRegistry, NoteStrategy, ProcessingConfig, CodeBlockProcessor
};
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Note Strategy DOCX Generation Example ===\n");

    // Create a code block processor with Note strategy
    let mut processor = CodeBlockProcessor::new();
    
    // Verify Note strategy is registered
    let stats = processor.get_processing_stats();
    println!("Processing Stats:");
    println!("  - Strategies: {}", stats.registered_strategies);
    println!("  - Languages: {}", stats.registered_languages.join(", "));
    println!();

    // Create a markdown document with various elements
    let mut document = MarkdownDocument::new();

    // Add title
    document.add_element(MarkdownElement::Heading {
        level: 1,
        text: "Note Strategy 示例文档".to_string(),
    });

    // Add introduction paragraph
    document.add_element(MarkdownElement::Paragraph {
        content: vec![
            InlineElement::Text("本文档展示了 Note Strategy 的使用效果。".to_string()),
        ],
    });

    // Example 1: Basic note
    println!("Example 1: Basic Note");
    let note1_content = "重要提示\n这是一个非常重要的注意事项。\n请务必仔细阅读。";
    let note1_result = processor.process_code_block(note1_content, Some("note"))?;
    
    document.add_element(MarkdownElement::Heading {
        level: 2,
        text: "示例 1：基本 Note".to_string(),
    });
    
    document.add_element(MarkdownElement::CodeBlock {
        language: Some("note".to_string()),
        code: note1_content.to_string(),
        processed: Some(note1_result),
    });

    // Example 2: Tip
    println!("Example 2: Pro Tip");
    let tip_content = "专业建议\n在提交代码之前：\n1. 运行所有测试\n2. 检查代码风格\n3. 更新文档";
    let tip_result = processor.process_code_block(tip_content, Some("tip"))?;
    
    document.add_element(MarkdownElement::Heading {
        level: 2,
        text: "示例 2：专业建议".to_string(),
    });
    
    document.add_element(MarkdownElement::CodeBlock {
        language: Some("tip".to_string()),
        code: tip_content.to_string(),
        processed: Some(tip_result),
    });

    // Example 3: Hint
    println!("Example 3: Quick Hint");
    let hint_content = "小提示\n使用 Ctrl+Shift+P 打开命令面板。";
    let hint_result = processor.process_code_block(hint_content, Some("hint"))?;
    
    document.add_element(MarkdownElement::Heading {
        level: 2,
        text: "示例 3：快速提示".to_string(),
    });
    
    document.add_element(MarkdownElement::CodeBlock {
        language: Some("hint".to_string()),
        code: hint_content.to_string(),
        processed: Some(hint_result),
    });

    // Example 4: Mixed with regular code
    println!("Example 4: Mixed Content");
    
    document.add_element(MarkdownElement::Heading {
        level: 2,
        text: "示例 4：混合内容".to_string(),
    });
    
    document.add_element(MarkdownElement::Paragraph {
        content: vec![
            InlineElement::Text("下面是一个 Rust 代码示例：".to_string()),
        ],
    });
    
    // Regular code block
    document.add_element(MarkdownElement::CodeBlock {
        language: Some("rust".to_string()),
        code: "fn main() {\n    println!(\"Hello, World!\");\n}".to_string(),
        processed: None,
    });
    
    // Note about the code
    let code_note = "代码说明\n这是一个简单的 Rust 程序，打印 \"Hello, World!\"。";
    let code_note_result = processor.process_code_block(code_note, Some("note"))?;
    
    document.add_element(MarkdownElement::CodeBlock {
        language: Some("note".to_string()),
        code: code_note.to_string(),
        processed: Some(code_note_result),
    });

    // Example 5: Security warning
    println!("Example 5: Security Warning");
    let security_content = "安全警告\n永远不要在代码中硬编码敏感信息：\n- API 密钥\n- 数据库密码\n- 私钥文件\n\n请使用环境变量或密钥管理服务。";
    let security_result = processor.process_code_block(security_content, Some("note"))?;
    
    document.add_element(MarkdownElement::Heading {
        level: 2,
        text: "示例 5：安全警告".to_string(),
    });
    
    document.add_element(MarkdownElement::CodeBlock {
        language: Some("note".to_string()),
        code: security_content.to_string(),
        processed: Some(security_result),
    });

    // Generate DOCX
    println!("\nGenerating DOCX...");
    let config = ConversionConfig::default();
    let mut generator = DocxGenerator::new(config);
    
    let docx = generator.generate(&document)?;
    
    // Save to file
    let output_path = "note_strategy_example.docx";
    std::fs::write(output_path, docx.build())?;
    
    println!("✓ DOCX generated successfully: {}", output_path);
    println!("\n打开 {} 查看渲染效果。", output_path);

    // Print statistics
    println!("\n=== 统计信息 ===");
    println!("总元素数: {}", document.element_count());
    
    let (processed, unprocessed) = document.count_code_blocks_by_status();
    println!("代码块:");
    println!("  - 已处理: {}", processed);
    println!("  - 未处理: {}", unprocessed);
    println!("  - 总计: {}", processed + unprocessed);

    Ok(())
}
