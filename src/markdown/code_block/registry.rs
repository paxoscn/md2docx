//! Strategy registry for managing code block processing strategies

use std::collections::HashMap;
use std::sync::Arc;
use crate::markdown::code_block::{CodeBlockStrategy, DefaultStrategy};

/// Registry that manages all available code block processing strategies
pub struct StrategyRegistry {
    strategies: HashMap<String, Arc<dyn CodeBlockStrategy>>,
    default_strategy: Arc<dyn CodeBlockStrategy>,
    language_aliases: HashMap<String, String>, // Maps aliases to canonical language names
}

impl StrategyRegistry {
    /// Create a new strategy registry with the default strategy
    pub fn new() -> Self {
        let default_strategy = Arc::new(DefaultStrategy::new());
        
        Self {
            strategies: HashMap::new(),
            default_strategy,
            language_aliases: HashMap::new(),
        }
    }

    /// Register a new strategy
    pub fn register_strategy(&mut self, strategy: Arc<dyn CodeBlockStrategy>) {
        let language = strategy.get_language_name().to_lowercase();
        
        // Check if we already have a strategy for this language
        if let Some(existing) = self.strategies.get(&language) {
            // Only replace if the new strategy has higher priority
            if strategy.get_priority() > existing.get_priority() {
                self.strategies.insert(language, strategy);
            }
        } else {
            self.strategies.insert(language, strategy);
        }
    }

    /// Register a strategy with a boxed trait object
    pub fn register_boxed_strategy(&mut self, strategy: Box<dyn CodeBlockStrategy>) {
        self.register_strategy(Arc::from(strategy));
    }

    /// Get a strategy for the given language
    pub fn get_strategy(&self, language: &str) -> Arc<dyn CodeBlockStrategy> {
        let normalized_lang = language.to_lowercase();
        
        // First, check if it's an alias
        let canonical_lang = self.language_aliases
            .get(&normalized_lang)
            .unwrap_or(&normalized_lang);
        
        // Then look up the strategy
        self.strategies
            .get(canonical_lang)
            .cloned()
            .unwrap_or_else(|| {
                // If no specific strategy found, check if any strategy supports this language
                for strategy in self.strategies.values() {
                    if strategy.supports_language(language) {
                        return strategy.clone();
                    }
                }
                // Fall back to default strategy
                self.default_strategy.clone()
            })
    }

    /// Get the default strategy
    pub fn get_default_strategy(&self) -> Arc<dyn CodeBlockStrategy> {
        self.default_strategy.clone()
    }

    /// Register a language alias
    pub fn register_alias(&mut self, alias: &str, canonical_language: &str) {
        self.language_aliases.insert(
            alias.to_lowercase(),
            canonical_language.to_lowercase(),
        );
    }

    /// Get all registered language names
    pub fn get_registered_languages(&self) -> Vec<String> {
        let mut languages: Vec<String> = self.strategies.keys().cloned().collect();
        // Always include the default strategy
        languages.push(self.default_strategy.get_language_name().to_string());
        languages.sort();
        languages.dedup();
        languages
    }

    /// Get all registered aliases
    pub fn get_registered_aliases(&self) -> Vec<String> {
        self.language_aliases.keys().cloned().collect()
    }

    /// Check if a language has a registered strategy
    pub fn has_strategy_for_language(&self, language: &str) -> bool {
        let normalized_lang = language.to_lowercase();
        
        // Check direct registration
        if self.strategies.contains_key(&normalized_lang) {
            return true;
        }
        
        // Check aliases
        if let Some(canonical) = self.language_aliases.get(&normalized_lang) {
            return self.strategies.contains_key(canonical);
        }
        
        // Check if any strategy supports this language
        self.strategies.values().any(|strategy| strategy.supports_language(language))
    }

    /// Get strategy information for debugging/introspection
    pub fn get_strategy_info(&self, language: &str) -> StrategyInfo {
        let strategy = self.get_strategy(language);
        let is_default = Arc::ptr_eq(&strategy, &self.default_strategy);
        
        StrategyInfo {
            language_name: strategy.get_language_name().to_string(),
            version: strategy.get_version().to_string(),
            description: strategy.get_description().to_string(),
            priority: strategy.get_priority(),
            is_default_strategy: is_default,
        }
    }

    /// Get information about all registered strategies
    pub fn list_all_strategies(&self) -> Vec<StrategyInfo> {
        let mut strategies = Vec::new();
        
        // Add default strategy
        strategies.push(StrategyInfo {
            language_name: self.default_strategy.get_language_name().to_string(),
            version: self.default_strategy.get_version().to_string(),
            description: self.default_strategy.get_description().to_string(),
            priority: self.default_strategy.get_priority(),
            is_default_strategy: true,
        });
        
        // Add registered strategies
        for strategy in self.strategies.values() {
            strategies.push(StrategyInfo {
                language_name: strategy.get_language_name().to_string(),
                version: strategy.get_version().to_string(),
                description: strategy.get_description().to_string(),
                priority: strategy.get_priority(),
                is_default_strategy: false,
            });
        }
        
        // Sort by priority (highest first)
        strategies.sort_by(|a, b| b.priority.cmp(&a.priority));
        strategies
    }

    /// Clear all registered strategies (except default)
    pub fn clear(&mut self) {
        self.strategies.clear();
        self.language_aliases.clear();
    }

    /// Get the number of registered strategies (excluding default)
    pub fn strategy_count(&self) -> usize {
        self.strategies.len()
    }

    /// Register common language aliases
    pub fn register_common_aliases(&mut self) {
        // Rust aliases
        self.register_alias("rs", "rust");
        
        // JavaScript aliases
        self.register_alias("js", "javascript");
        self.register_alias("jsx", "javascript");
        self.register_alias("ts", "typescript");
        self.register_alias("tsx", "typescript");
        
        // Python aliases
        self.register_alias("py", "python");
        self.register_alias("python3", "python");
        
        // C/C++ aliases
        self.register_alias("c++", "cpp");
        self.register_alias("cxx", "cpp");
        
        // Shell aliases
        self.register_alias("sh", "shell");
        self.register_alias("bash", "shell");
        self.register_alias("zsh", "shell");
        
        // Markup aliases
        self.register_alias("md", "markdown");
        self.register_alias("yml", "yaml");
    }
}

impl Default for StrategyRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        registry.register_common_aliases();
        registry
    }
}

/// Information about a registered strategy
#[derive(Debug, Clone)]
pub struct StrategyInfo {
    pub language_name: String,
    pub version: String,
    pub description: String,
    pub priority: u8,
    pub is_default_strategy: bool,
}

impl StrategyInfo {
    /// Get a formatted string representation of the strategy info
    pub fn format(&self) -> String {
        format!(
            "{} v{} (priority: {}) - {}{}",
            self.language_name,
            self.version,
            self.priority,
            self.description,
            if self.is_default_strategy { " [DEFAULT]" } else { "" }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::code_block::{ProcessingConfig, ProcessedCodeBlock, ProcessingError};
    use std::sync::Arc;

    // Mock strategy for testing
    #[derive(Debug)]
    struct MockStrategy {
        language: &'static str,
        priority: u8,
    }

    impl MockStrategy {
        fn new(language: &'static str, priority: u8) -> Self {
            Self { language, priority }
        }
    }

    impl CodeBlockStrategy for MockStrategy {
        fn process(&self, code: &str, _config: &ProcessingConfig) -> Result<ProcessedCodeBlock, ProcessingError> {
            Ok(ProcessedCodeBlock::new(code.to_string(), Some(self.language.to_string())))
        }

        fn supports_language(&self, language: &str) -> bool {
            language.to_lowercase() == self.language.to_lowercase()
        }

        fn get_language_name(&self) -> &'static str {
            self.language
        }

        fn get_priority(&self) -> u8 {
            self.priority
        }
    }

    #[test]
    fn test_registry_creation() {
        let registry = StrategyRegistry::new();
        assert_eq!(registry.strategy_count(), 0);
        assert!(registry.get_registered_languages().is_empty());
    }

    #[test]
    fn test_register_and_get_strategy() {
        let mut registry = StrategyRegistry::new();
        let rust_strategy = Arc::new(MockStrategy::new("rust", 100));
        
        registry.register_strategy(rust_strategy);
        
        let retrieved = registry.get_strategy("rust");
        assert_eq!(retrieved.get_language_name(), "rust");
        assert_eq!(retrieved.get_priority(), 100);
    }

    #[test]
    fn test_strategy_priority_replacement() {
        let mut registry = StrategyRegistry::new();
        
        // Register low priority strategy first
        let low_priority = Arc::new(MockStrategy::new("rust", 50));
        registry.register_strategy(low_priority);
        
        // Register high priority strategy
        let high_priority = Arc::new(MockStrategy::new("rust", 150));
        registry.register_strategy(high_priority);
        
        let retrieved = registry.get_strategy("rust");
        assert_eq!(retrieved.get_priority(), 150);
    }

    #[test]
    fn test_strategy_priority_no_replacement() {
        let mut registry = StrategyRegistry::new();
        
        // Register high priority strategy first
        let high_priority = Arc::new(MockStrategy::new("rust", 150));
        registry.register_strategy(high_priority);
        
        // Try to register low priority strategy
        let low_priority = Arc::new(MockStrategy::new("rust", 50));
        registry.register_strategy(low_priority);
        
        let retrieved = registry.get_strategy("rust");
        assert_eq!(retrieved.get_priority(), 150);
    }

    #[test]
    fn test_default_strategy_fallback() {
        let registry = StrategyRegistry::new();
        
        let strategy = registry.get_strategy("unknown_language");
        assert_eq!(strategy.get_language_name(), "default");
    }

    #[test]
    fn test_language_aliases() {
        let mut registry = StrategyRegistry::new();
        let rust_strategy = Arc::new(MockStrategy::new("rust", 100));
        
        registry.register_strategy(rust_strategy);
        registry.register_alias("rs", "rust");
        
        let retrieved = registry.get_strategy("rs");
        assert_eq!(retrieved.get_language_name(), "rust");
    }

    #[test]
    fn test_case_insensitive_lookup() {
        let mut registry = StrategyRegistry::new();
        let rust_strategy = Arc::new(MockStrategy::new("rust", 100));
        
        registry.register_strategy(rust_strategy);
        
        let retrieved1 = registry.get_strategy("RUST");
        let retrieved2 = registry.get_strategy("Rust");
        let retrieved3 = registry.get_strategy("rust");
        
        assert_eq!(retrieved1.get_language_name(), "rust");
        assert_eq!(retrieved2.get_language_name(), "rust");
        assert_eq!(retrieved3.get_language_name(), "rust");
    }

    #[test]
    fn test_has_strategy_for_language() {
        let mut registry = StrategyRegistry::new();
        let rust_strategy = Arc::new(MockStrategy::new("rust", 100));
        
        registry.register_strategy(rust_strategy);
        registry.register_alias("rs", "rust");
        
        assert!(registry.has_strategy_for_language("rust"));
        assert!(registry.has_strategy_for_language("RUST"));
        assert!(registry.has_strategy_for_language("rs"));
        assert!(!registry.has_strategy_for_language("python"));
    }

    #[test]
    fn test_get_registered_languages() {
        let mut registry = StrategyRegistry::new();
        
        registry.register_strategy(Arc::new(MockStrategy::new("rust", 100)));
        registry.register_strategy(Arc::new(MockStrategy::new("javascript", 100)));
        
        let languages = registry.get_registered_languages();
        assert_eq!(languages.len(), 2);
        assert!(languages.contains(&"rust".to_string()));
        assert!(languages.contains(&"javascript".to_string()));
    }

    #[test]
    fn test_strategy_info() {
        let mut registry = StrategyRegistry::new();
        let rust_strategy = Arc::new(MockStrategy::new("rust", 100));
        
        registry.register_strategy(rust_strategy);
        
        let info = registry.get_strategy_info("rust");
        assert_eq!(info.language_name, "rust");
        assert_eq!(info.priority, 100);
        assert!(!info.is_default_strategy);
        
        let default_info = registry.get_strategy_info("unknown");
        assert!(default_info.is_default_strategy);
    }

    #[test]
    fn test_list_all_strategies() {
        let mut registry = StrategyRegistry::new();
        
        registry.register_strategy(Arc::new(MockStrategy::new("rust", 150)));
        registry.register_strategy(Arc::new(MockStrategy::new("javascript", 100)));
        
        let strategies = registry.list_all_strategies();
        assert_eq!(strategies.len(), 3); // 2 registered + 1 default
        
        // Should be sorted by priority (highest first)
        assert_eq!(strategies[0].language_name, "rust");
        assert_eq!(strategies[1].language_name, "javascript");
        assert!(strategies[2].is_default_strategy);
    }

    #[test]
    fn test_clear_registry() {
        let mut registry = StrategyRegistry::new();
        
        registry.register_strategy(Arc::new(MockStrategy::new("rust", 100)));
        registry.register_alias("rs", "rust");
        
        assert_eq!(registry.strategy_count(), 1);
        assert!(!registry.get_registered_aliases().is_empty());
        
        registry.clear();
        
        assert_eq!(registry.strategy_count(), 0);
        assert!(registry.get_registered_aliases().is_empty());
        
        // Default strategy should still work
        let strategy = registry.get_strategy("rust");
        assert_eq!(strategy.get_language_name(), "default");
    }

    #[test]
    fn test_common_aliases() {
        let registry = StrategyRegistry::default();
        
        // Test some common aliases are registered
        let aliases = registry.get_registered_aliases();
        assert!(aliases.contains(&"rs".to_string()));
        assert!(aliases.contains(&"js".to_string()));
        assert!(aliases.contains(&"py".to_string()));
    }

    #[test]
    fn test_boxed_strategy_registration() {
        let mut registry = StrategyRegistry::new();
        let boxed_strategy: Box<dyn CodeBlockStrategy> = Box::new(MockStrategy::new("rust", 100));
        
        registry.register_boxed_strategy(boxed_strategy);
        
        let retrieved = registry.get_strategy("rust");
        assert_eq!(retrieved.get_language_name(), "rust");
    }
}