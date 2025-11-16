//! Example demonstrating Rust keyword bold formatting
//! 
//! This example shows how the RustStrategy automatically applies bold formatting
//! to Rust keywords when processing code blocks.

use md2docx_converter::markdown::code_block::{
    RustStrategy, CodeBlockStrategy, ProcessingConfig,
};

fn main() {
    // Create a Rust strategy instance
    let strategy = RustStrategy::new();
    
    // Sample Rust code
    let rust_code = r#"
fn main() {
    let x: i32 = 42;
    let s: String = String::from("hello");
    
    if x > 0 {
        println!("x is positive");
    } else {
        println!("x is not positive");
    }
    
    for i in 0..10 {
        println!("i = {}", i);
    }
    
    match x {
        0 => println!("zero"),
        _ => println!("non-zero"),
    }
}

pub struct MyStruct {
    pub field: bool,
}

impl MyStruct {
    pub fn new() -> Self {
        Self { field: true }
    }
}
"#;
    
    // Configure processing with formatting enabled
    let config = ProcessingConfig::default()
        .with_syntax_validation(true)
        .with_formatting(true);
    
    // Process the code
    match strategy.process(rust_code, &config) {
        Ok(processed) => {
            println!("=== Original Code ===");
            println!("{}", processed.original_code);
            println!();
            
            if let Some(formatted) = &processed.processed_code {
                println!("=== Formatted Code (with bold keywords) ===");
                println!("{}", formatted);
                println!();
            }
            
            println!("=== Processing Summary ===");
            let summary = processed.get_summary();
            println!("Language: {:?}", summary.language);
            println!("Was processed: {}", summary.was_processed);
            println!("Was modified: {}", summary.was_modified);
            println!("Is valid: {}", summary.is_valid);
            println!("Status: {}", summary.get_status());
            println!("Processing time: {:?}", summary.processing_time);
            
            if processed.has_warnings() {
                println!();
                println!("=== Warnings ===");
                for warning in &processed.warnings {
                    println!("- [{}] {}", warning.warning_type, warning.message);
                }
            }
        }
        Err(e) => {
            eprintln!("Error processing code: {:?}", e);
        }
    }
}
