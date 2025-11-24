//! Example demonstrating the Note Strategy usage

use md2docx_converter::markdown::code_block::{
    StrategyRegistry, NoteStrategy, ProcessingConfig
};
use md2docx_converter::markdown::CodeBlockStrategy;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Note Strategy Example ===\n");

    // Create a strategy registry
    let mut registry = StrategyRegistry::new();

    // Register the Note strategy
    let note_strategy = Arc::new(NoteStrategy::new());
    registry.register_strategy(note_strategy.clone());

    println!("Registered strategies:");
    for info in registry.list_all_strategies() {
        println!("  - {}", info.format());
    }
    println!();

    // Example 1: Basic note
    println!("Example 1: Basic Note");
    println!("---");
    let note_content = "Important Notice\nThis is a critical piece of information.\nPlease read carefully.";
    
    let config = ProcessingConfig::default()
        .with_formatting(true);
    
    let result = note_strategy.process(note_content, &config)?;
    
    println!("Original content:");
    println!("{}\n", result.original_code);
    
    if let Some(formatted) = &result.processed_code {
        println!("Formatted output:");
        println!("{}\n", formatted);
    }
    
    println!("Metadata:");
    println!("  - Language: {:?}", result.language);
    println!("  - Is formatted: {}", result.metadata.is_formatted);
    println!("  - Processing time: {:?}", result.metadata.processing_time);
    println!("  - Icon path: {:?}", result.metadata.get_custom_attribute("icon_path"));
    println!();

    // Example 2: Using alias "tip"
    println!("Example 2: Using 'tip' Alias");
    println!("---");
    let tip_content = "Pro Tip\nAlways test your code before committing to production.";
    
    let strategy_for_tip = registry.get_strategy("tip");
    println!("Strategy for 'tip': {}", strategy_for_tip.get_language_name());
    
    let result = strategy_for_tip.process(tip_content, &config)?;
    
    if let Some(formatted) = &result.processed_code {
        println!("Formatted tip:");
        println!("{}\n", formatted);
    }

    // Example 3: Custom icon path
    println!("Example 3: Custom Icon Path");
    println!("---");
    let custom_strategy = NoteStrategy::with_icon_path("assets/custom-tip-icon.svg".to_string());
    
    let hint_content = "Quick Hint\nUse keyboard shortcuts to boost productivity.";
    let result = custom_strategy.process(hint_content, &config)?;
    
    println!("Custom icon path: {}", custom_strategy.get_icon_path());
    if let Some(formatted) = &result.processed_code {
        println!("Formatted hint:");
        println!("{}\n", formatted);
    }

    // Example 4: Without formatting
    println!("Example 4: Without Formatting");
    println!("---");
    let no_format_config = ProcessingConfig::default()
        .with_formatting(false);
    
    let result = note_strategy.process(note_content, &no_format_config)?;
    
    println!("Is formatted: {}", result.metadata.is_formatted);
    println!("Processed code: {:?}\n", result.processed_code);

    // Example 5: Single line note
    println!("Example 5: Single Line Note");
    println!("---");
    let single_line = "Remember to save your work!";
    let result = note_strategy.process(single_line, &config)?;
    
    if let Some(formatted) = &result.processed_code {
        println!("Formatted single line:");
        println!("{}\n", formatted);
    }

    // Example 6: Strategy information
    println!("Example 6: Strategy Information");
    println!("---");
    let info = registry.get_strategy_info("note");
    println!("Strategy Info:");
    println!("  - Name: {}", info.language_name);
    println!("  - Version: {}", info.version);
    println!("  - Priority: {}", info.priority);
    println!("  - Description: {}", info.description);
    println!("  - Is default: {}", info.is_default_strategy);
    println!();

    // Example 7: Supported languages
    println!("Example 7: Supported Languages");
    println!("---");
    let test_languages = vec!["note", "notes", "tip", "tips", "hint", "NOTE", "TIP", "rust"];
    
    for lang in test_languages {
        let supports = note_strategy.supports_language(lang);
        println!("  - '{}': {}", lang, if supports { "✓" } else { "✗" });
    }

    Ok(())
}
