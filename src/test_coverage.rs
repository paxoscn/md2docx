//! Test coverage utilities and benchmarks

#[cfg(test)]
mod coverage_tests {
    use crate::test_utils::*;

    /// Test coverage for all core modules
    #[test]
    fn test_module_coverage() {
        // This test ensures all major modules have basic functionality working
        
        // Test config module
        let config = create_test_config();
        assert!(config.validate().is_ok());
        
        // Test markdown parsing
        let parser = crate::MarkdownParser::new();
        let markdown = create_test_markdown();
        let doc = parser.parse(&markdown).unwrap();
        assert!(!doc.elements.is_empty());
        
        // Test docx generation
        let mut generator = crate::DocxGenerator::new(config.clone());
        let docx_bytes = generator.generate(&doc).unwrap();
        assert!(!docx_bytes.is_empty());
        
        // Test conversion engine
        let engine = crate::ConversionEngine::new(config);
        let stats = engine.get_conversion_stats(&markdown).unwrap();
        assert!(stats.total_elements > 0);
    }

    /// Performance benchmark test
    #[test]
    fn test_performance_baseline() {
        use std::time::Instant;
        
        let config = create_test_config();
        let engine = crate::ConversionEngine::new(config);
        let markdown = create_test_markdown();
        
        let start = Instant::now();
        let result = engine.get_conversion_stats(&markdown);
        let duration = start.elapsed();
        
        assert!(result.is_ok());
        // Should complete within reasonable time (adjust as needed)
        assert!(duration.as_millis() < 100);
    }

    /// Memory usage test
    #[test]
    fn test_memory_usage() {
        let config = create_test_config();
        let engine = crate::ConversionEngine::new(config);
        
        // Create a large markdown document
        let mut large_markdown = String::new();
        for i in 0..1000 {
            large_markdown.push_str(&format!("# Heading {}\n\nParagraph {} with some content.\n\n", i, i));
        }
        
        let stats = engine.get_conversion_stats(&large_markdown).unwrap();
        assert!(stats.headings == 1000);
        assert!(stats.paragraphs == 1000);
    }

    /// Error handling coverage test
    #[test]
    fn test_error_handling_coverage() {
        use crate::error::*;
        
        // Test all error types can be created and displayed
        let conversion_errors = vec![
            ConversionError::markdown_parsing("test"),
            ConversionError::docx_generation("test"),
            ConversionError::file_processing("test"),
            ConversionError::timeout("test"),
        ];
        
        for error in conversion_errors {
            assert!(!error.to_string().is_empty());
            let _category = error.category();
            let _recoverable = error.is_recoverable();
        }
        
        let config_errors = vec![
            ConfigError::invalid_yaml("test"),
            ConfigError::validation("test"),
            ConfigError::llm_api("test"),
        ];
        
        for error in config_errors {
            assert!(!error.to_string().is_empty());
        }
    }

    /// Integration test for full conversion pipeline
    #[tokio::test]
    async fn test_full_pipeline_integration() {
        let config = create_test_config();
        let mut engine = crate::ConversionEngine::new(config);
        let markdown = create_test_markdown();
        
        // Test the full conversion pipeline
        let result = engine.convert(&markdown).await;
        assert!(result.is_ok());
        
        let docx_bytes = result.unwrap();
        assert!(!docx_bytes.is_empty());
        assert!(docx_bytes.len() > 1000); // Should be substantial
    }

    /// Test configuration validation coverage
    #[test]
    fn test_config_validation_coverage() {
        // Test valid config
        let valid_config = create_test_config();
        assert!(valid_config.validate().is_ok());
        
        // Test invalid configs
        let invalid_configs = vec![
            {
                let mut config = create_test_config();
                config.document.page_size.width = -100.0;
                config
            },
            {
                let mut config = create_test_config();
                config.document.margins.top = -10.0;
                config
            },
            {
                let mut config = create_test_config();
                config.document.default_font.size = 0.0;
                config
            },
            {
                let mut config = create_test_config();
                config.document.default_font.family = "".to_string();
                config
            },
        ];
        
        for invalid_config in invalid_configs {
            assert!(invalid_config.validate().is_err());
        }
    }

    /// Test markdown parsing edge cases
    #[test]
    fn test_markdown_parsing_edge_cases() {
        let parser = crate::MarkdownParser::new();
        
        let edge_cases = vec![
            ("", 0), // Empty document
            ("# ", 1), // Empty heading
            ("**", 1), // Incomplete formatting
            ("![]()", 1), // Empty image
            ("[]", 1), // Empty link
            ("```\n```", 1), // Empty code block
            ("| |\n|---|", 1), // Minimal table
        ];
        
        for (markdown, expected_elements) in edge_cases {
            let result = parser.parse(markdown);
            if expected_elements == 0 {
                assert!(result.unwrap().elements.is_empty());
            } else {
                assert!(result.is_ok());
                let doc = result.unwrap();
                assert!(doc.elements.len() >= expected_elements);
            }
        }
    }
}

#[cfg(test)]
mod benchmark_tests {
    use crate::test_utils::*;
    use std::time::Instant;

    #[test]
    fn benchmark_markdown_parsing() {
        let parser = crate::MarkdownParser::new();
        let markdown = create_test_markdown();
        
        let iterations = 100;
        let start = Instant::now();
        
        for _ in 0..iterations {
            let _ = parser.parse(&markdown).unwrap();
        }
        
        let duration = start.elapsed();
        let avg_duration = duration / iterations;
        
        println!("Average parsing time: {:?}", avg_duration);
        // Should be reasonably fast
        assert!(avg_duration.as_millis() < 10);
    }

    #[tokio::test]
    async fn benchmark_docx_generation() {
        let config = create_test_config();
        let mut generator = crate::DocxGenerator::new(config);
        let document = create_test_document();
        
        let iterations = 10;
        let start = Instant::now();
        
        for _ in 0..iterations {
            let _ = generator.generate(&document).unwrap();
        }
        
        let duration = start.elapsed();
        let avg_duration = duration / iterations;
        
        println!("Average generation time: {:?}", avg_duration);
        // Should complete within reasonable time
        assert!(avg_duration.as_millis() < 1000);
    }

    #[tokio::test]
    async fn benchmark_full_conversion() {
        let config = create_test_config();
        let mut engine = crate::ConversionEngine::new(config);
        let markdown = create_test_markdown();
        
        let iterations = 10;
        let start = Instant::now();
        
        for _ in 0..iterations {
            let _ = engine.convert(&markdown).await.unwrap();
        }
        
        let duration = start.elapsed();
        let avg_duration = duration / iterations;
        
        println!("Average full conversion time: {:?}", avg_duration);
        // Should complete within reasonable time
        assert!(avg_duration.as_millis() < 1000);
    }
}