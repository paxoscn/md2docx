//! Integration tests for Note Strategy

use md2docx_converter::markdown::code_block::{
    StrategyRegistry, NoteStrategy, ProcessingConfig, RustStrategy, CodeBlockStrategy
};
use std::sync::Arc;

#[test]
fn test_note_strategy_registration() {
    let mut registry = StrategyRegistry::new();
    let note_strategy = Arc::new(NoteStrategy::new());
    
    registry.register_strategy(note_strategy);
    
    // Verify the strategy is registered
    assert!(registry.has_strategy_for_language("note"));
    assert!(registry.has_strategy_for_language("tip"));
    assert!(registry.has_strategy_for_language("hint"));
}

#[test]
fn test_note_strategy_with_multiple_strategies() {
    let mut registry = StrategyRegistry::new();
    
    // Register multiple strategies
    registry.register_strategy(Arc::new(NoteStrategy::new()));
    registry.register_strategy(Arc::new(RustStrategy::new()));
    
    // Verify each strategy handles its own language
    let note_strategy = registry.get_strategy("note");
    assert_eq!(note_strategy.get_language_name(), "note");
    
    let rust_strategy = registry.get_strategy("rust");
    assert_eq!(rust_strategy.get_language_name(), "rust");
}

#[test]
fn test_note_strategy_priority() {
    let mut registry = StrategyRegistry::new();
    
    registry.register_strategy(Arc::new(NoteStrategy::new()));
    
    let info = registry.get_strategy_info("note");
    assert_eq!(info.priority, 120);
    assert!(!info.is_default_strategy);
}

#[test]
fn test_note_strategy_processing_pipeline() {
    let strategy = NoteStrategy::new();
    let config = ProcessingConfig::default()
        .with_formatting(true);
    
    let content = "Important Notice\nThis is a test note.\nWith multiple lines.";
    
    let result = strategy.process(content, &config).unwrap();
    
    // Verify processing results
    assert!(result.is_successful());
    assert_eq!(result.error_count(), 0);
    assert_eq!(result.warning_count(), 0);
    assert!(result.metadata.is_formatted);
    assert!(result.metadata.syntax_valid);
    
    // Verify formatted output
    let formatted = result.processed_code.unwrap();
    assert!(formatted.contains("<table"));
    assert!(formatted.contains("Important Notice"));
    assert!(formatted.contains("font-weight: bold"));
    assert!(formatted.contains("font-style: italic"));
    assert!(formatted.contains("<img"));
}

#[test]
fn test_note_strategy_with_custom_icon() {
    let strategy = NoteStrategy::with_icon_path("custom-icon.png".to_string());
    let config = ProcessingConfig::default()
        .with_formatting(true);
    
    let content = "Test Note\nContent here.";
    let result = strategy.process(content, &config).unwrap();
    
    let formatted = result.processed_code.unwrap();
    assert!(formatted.contains("custom-icon.png"));
}

#[test]
fn test_note_strategy_metadata_attributes() {
    let strategy = NoteStrategy::new();
    let config = ProcessingConfig::default()
        .with_formatting(true);
    
    let content = "Test\nContent";
    let result = strategy.process(content, &config).unwrap();
    
    // Verify custom attributes
    assert_eq!(
        result.metadata.get_custom_attribute("language"),
        Some(&"note".to_string())
    );
    assert_eq!(
        result.metadata.get_custom_attribute("formatter"),
        Some(&"note_formatter".to_string())
    );
    assert_eq!(
        result.metadata.get_custom_attribute("icon_path"),
        Some(&"default-qrcode.png".to_string())
    );
}

#[test]
fn test_note_strategy_without_formatting() {
    let strategy = NoteStrategy::new();
    let config = ProcessingConfig::default()
        .with_formatting(false);
    
    let content = "Test Note\nNo formatting.";
    let result = strategy.process(content, &config).unwrap();
    
    assert!(!result.metadata.is_formatted);
    assert!(result.processed_code.is_none());
    assert_eq!(result.get_final_code(), content);
}

#[test]
fn test_note_strategy_empty_content() {
    let strategy = NoteStrategy::new();
    let config = ProcessingConfig::default()
        .with_formatting(true);
    
    let result = strategy.process("", &config).unwrap();
    
    assert!(result.is_successful());
    assert!(result.processed_code.is_some());
}

#[test]
fn test_note_strategy_single_line() {
    let strategy = NoteStrategy::new();
    let config = ProcessingConfig::default()
        .with_formatting(true);
    
    let content = "Single line note";
    let result = strategy.process(content, &config).unwrap();
    
    let formatted = result.processed_code.unwrap();
    assert!(formatted.contains("Single line note"));
    assert!(formatted.contains("font-weight: bold"));
}

#[test]
fn test_note_strategy_multiline_content() {
    let strategy = NoteStrategy::new();
    let config = ProcessingConfig::default()
        .with_formatting(true);
    
    let content = "Title\nLine 1\nLine 2\nLine 3\nLine 4";
    let result = strategy.process(content, &config).unwrap();
    
    let formatted = result.processed_code.unwrap();
    
    // First line should be styled
    assert!(formatted.contains("Title"));
    assert!(formatted.contains("font-weight: bold"));
    
    // Other lines should be present
    assert!(formatted.contains("Line 1"));
    assert!(formatted.contains("Line 2"));
    assert!(formatted.contains("Line 3"));
    assert!(formatted.contains("Line 4"));
}

#[test]
fn test_note_strategy_language_aliases() {
    let strategy = NoteStrategy::new();
    
    let aliases = vec!["note", "notes", "tip", "tips", "hint"];
    
    for alias in aliases {
        assert!(
            strategy.supports_language(alias),
            "Should support alias: {}",
            alias
        );
    }
}

#[test]
fn test_note_strategy_case_insensitive() {
    let strategy = NoteStrategy::new();
    
    assert!(strategy.supports_language("note"));
    assert!(strategy.supports_language("NOTE"));
    assert!(strategy.supports_language("Note"));
    assert!(strategy.supports_language("tip"));
    assert!(strategy.supports_language("TIP"));
    assert!(strategy.supports_language("Tip"));
}

#[test]
fn test_note_strategy_does_not_support_other_languages() {
    let strategy = NoteStrategy::new();
    
    assert!(!strategy.supports_language("rust"));
    assert!(!strategy.supports_language("python"));
    assert!(!strategy.supports_language("javascript"));
    assert!(!strategy.supports_language("markdown"));
}

#[test]
fn test_note_strategy_processing_time() {
    let strategy = NoteStrategy::new();
    let config = ProcessingConfig::default()
        .with_formatting(true);
    
    let content = "Test\nContent";
    let result = strategy.process(content, &config).unwrap();
    
    // Processing time should be recorded
    assert!(result.metadata.processing_time.as_nanos() > 0);
}

#[test]
fn test_note_strategy_version_and_description() {
    let strategy = NoteStrategy::new();
    
    assert_eq!(strategy.get_version(), "1.0.0");
    assert!(!strategy.get_description().is_empty());
    assert!(strategy.get_description().contains("Note"));
}

#[test]
fn test_note_strategy_with_special_characters() {
    let strategy = NoteStrategy::new();
    let config = ProcessingConfig::default()
        .with_formatting(true);
    
    let content = "Special Characters: <>&\"'\nContent with symbols: @#$%^&*()";
    let result = strategy.process(content, &config).unwrap();
    
    assert!(result.is_successful());
    let formatted = result.processed_code.unwrap();
    assert!(formatted.contains("Special Characters"));
}

#[test]
fn test_note_strategy_with_unicode() {
    let strategy = NoteStrategy::new();
    let config = ProcessingConfig::default()
        .with_formatting(true);
    
    let content = "中文标题\n这是中文内容。\n日本語、한국어、Español";
    let result = strategy.process(content, &config).unwrap();
    
    assert!(result.is_successful());
    let formatted = result.processed_code.unwrap();
    assert!(formatted.contains("中文标题"));
    assert!(formatted.contains("这是中文内容"));
}

#[test]
fn test_note_strategy_summary() {
    let strategy = NoteStrategy::new();
    let config = ProcessingConfig::default()
        .with_formatting(true);
    
    let content = "Test\nContent";
    let result = strategy.process(content, &config).unwrap();
    
    let summary = result.get_summary();
    assert!(summary.is_successful());
    assert!(!summary.has_issues());
    assert_eq!(summary.get_status(), "success");
}
