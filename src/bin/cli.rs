//! CLI binary for Markdown to docx conversion

use clap::{Parser, Subcommand};
use md2docx_converter::{
    config::{ConversionConfig, service::ConfigurationService},
    conversion::ConversionEngine,
    error::{ConversionError, ConfigError},
};
use std::path::{Path, PathBuf};
use std::fs;
use std::time::Instant;
use tracing::{info, error, warn, debug};
use tracing_subscriber;

#[derive(Parser)]
#[command(name = "md2docx")]
#[command(about = "A configurable Markdown to docx converter with natural language configuration support")]
#[command(version)]
#[command(long_about = "Convert Markdown files to Microsoft docx format with flexible YAML configuration and natural language configuration updates via LLM integration.")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
    
    /// Enable debug logging
    #[arg(short, long, global = true)]
    debug: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert a single Markdown file to docx
    Convert {
        /// Input Markdown file path
        #[arg(short, long, value_name = "FILE")]
        input: PathBuf,
        
        /// Output docx file path (optional, defaults to input filename with .docx extension)
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
        
        /// Configuration file path (YAML format)
        #[arg(short, long, value_name = "FILE")]
        config: Option<PathBuf>,
        
        /// Natural language prompt to modify configuration before conversion
        #[arg(long, value_name = "PROMPT")]
        config_prompt: Option<String>,
        
        /// Show conversion statistics
        #[arg(long)]
        stats: bool,
    },
    
    /// Convert multiple Markdown files (batch processing)
    Batch {
        /// Input directory containing Markdown files
        #[arg(short, long, value_name = "DIR")]
        input_dir: PathBuf,
        
        /// Output directory for docx files (optional, defaults to input directory)
        #[arg(short, long, value_name = "DIR")]
        output_dir: Option<PathBuf>,
        
        /// Configuration file path (YAML format)
        #[arg(short, long, value_name = "FILE")]
        config: Option<PathBuf>,
        
        /// Natural language prompt to modify configuration before conversion
        #[arg(long, value_name = "PROMPT")]
        config_prompt: Option<String>,
        
        /// Process files recursively in subdirectories
        #[arg(short, long)]
        recursive: bool,
        
        /// Maximum number of parallel conversions
        #[arg(long, default_value = "4")]
        parallel: usize,
        
        /// Show detailed progress information
        #[arg(long)]
        progress: bool,
    },
    
    /// Start the web server
    Server {
        /// Port to listen on
        #[arg(short, long, default_value = "3000")]
        port: u16,
        
        /// Host address to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },
    
    /// Validate a configuration file
    Config {
        /// Configuration file path to validate
        #[arg(short, long, value_name = "FILE")]
        file: PathBuf,
        
        /// Natural language prompt to modify configuration
        #[arg(long, value_name = "PROMPT")]
        update_prompt: Option<String>,
        
        /// Preview the updated configuration without saving
        #[arg(long)]
        preview: bool,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    // Initialize logging
    init_logging(cli.verbose, cli.debug);
    
    info!("Starting md2docx CLI tool");
    
    let result = match cli.command {
        Commands::Convert { 
            input, 
            output, 
            config, 
            config_prompt, 
            stats 
        } => {
            handle_convert(input, output, config, config_prompt, stats).await
        }
        
        Commands::Batch { 
            input_dir, 
            output_dir, 
            config, 
            config_prompt, 
            recursive, 
            parallel, 
            progress 
        } => {
            handle_batch(input_dir, output_dir, config, config_prompt, recursive, parallel, progress).await
        }
        
        Commands::Server { port, host } => {
            handle_server(host, port).await
        }
        
        Commands::Config { file, update_prompt, preview } => {
            handle_config(file, update_prompt, preview).await
        }
    };
    
    match result {
        Ok(_) => {
            info!("CLI operation completed successfully");
            Ok(())
        }
        Err(e) => {
            error!("CLI operation failed: {}", e);
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

/// Initialize logging based on verbosity flags
fn init_logging(verbose: bool, debug: bool) {
    let level = if debug {
        tracing::Level::DEBUG
    } else if verbose {
        tracing::Level::INFO
    } else {
        tracing::Level::WARN
    };
    
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .init();
}

/// Handle single file conversion
async fn handle_convert(
    input: PathBuf,
    output: Option<PathBuf>,
    config_path: Option<PathBuf>,
    config_prompt: Option<String>,
    show_stats: bool,
) -> Result<(), ConversionError> {
    info!("Starting single file conversion");
    
    // Validate input file
    if !input.exists() {
        return Err(ConversionError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Input file not found: {}", input.display()),
        )));
    }
    
    if !input.extension().map_or(false, |ext| ext == "md" || ext == "markdown") {
        warn!("Input file does not have .md or .markdown extension: {}", input.display());
    }
    
    // Determine output path
    let output_path = output.unwrap_or_else(|| {
        input.with_extension("docx")
    });
    
    println!("Converting: {} -> {}", input.display(), output_path.display());
    
    // Load and process configuration
    let mut config = load_config(config_path.as_deref()).await?;
    
    if let Some(prompt) = config_prompt {
        println!("Updating configuration with natural language prompt...");
        config = update_config_with_prompt(config, &prompt).await?;
        println!("Configuration updated successfully");
    }
    
    // Create conversion engine
    let mut engine = ConversionEngine::new(config);
    
    // Show conversion statistics if requested
    if show_stats {
        let markdown_content = fs::read_to_string(&input)
            .map_err(ConversionError::Io)?;
        
        let stats = engine.get_conversion_stats(&markdown_content)?;
        println!("Conversion Statistics:");
        println!("  {}", stats.summary());
    }
    
    // Perform conversion
    let start_time = Instant::now();
    
    engine.convert_file(
        input.to_str().unwrap(),
        output_path.to_str().unwrap(),
    ).await?;
    
    let duration = start_time.elapsed();
    
    println!("✓ Conversion completed successfully in {:.2}s", duration.as_secs_f64());
    println!("  Output: {}", output_path.display());
    
    // Show file size information
    if let Ok(metadata) = fs::metadata(&output_path) {
        println!("  Size: {} bytes", metadata.len());
    }
    
    Ok(())
}

/// Handle batch conversion
async fn handle_batch(
    input_dir: PathBuf,
    output_dir: Option<PathBuf>,
    config_path: Option<PathBuf>,
    config_prompt: Option<String>,
    recursive: bool,
    parallel: usize,
    show_progress: bool,
) -> Result<(), ConversionError> {
    info!("Starting batch conversion");
    
    // Validate input directory
    if !input_dir.exists() || !input_dir.is_dir() {
        return Err(ConversionError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Input directory not found: {}", input_dir.display()),
        )));
    }
    
    // Determine output directory
    let output_dir = output_dir.unwrap_or_else(|| input_dir.clone());
    
    // Create output directory if it doesn't exist
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir)
            .map_err(ConversionError::Io)?;
    }
    
    println!("Batch conversion: {} -> {}", input_dir.display(), output_dir.display());
    
    // Find all Markdown files
    let markdown_files = find_markdown_files(&input_dir, recursive)?;
    
    if markdown_files.is_empty() {
        println!("No Markdown files found in {}", input_dir.display());
        return Ok(());
    }
    
    println!("Found {} Markdown files", markdown_files.len());
    
    // Load and process configuration
    let mut config = load_config(config_path.as_deref()).await?;
    
    if let Some(prompt) = config_prompt {
        println!("Updating configuration with natural language prompt...");
        config = update_config_with_prompt(config, &prompt).await?;
        println!("Configuration updated successfully");
    }
    
    // Create conversion engine
    let mut engine = ConversionEngine::new(config);
    
    // Prepare file pairs for batch conversion
    let file_pairs: Vec<(String, String)> = markdown_files
        .iter()
        .map(|input_path| {
            let relative_path = input_path.strip_prefix(&input_dir).unwrap();
            let output_path = output_dir.join(relative_path).with_extension("docx");
            
            // Create output subdirectory if needed
            if let Some(parent) = output_path.parent() {
                if !parent.exists() {
                    let _ = fs::create_dir_all(parent);
                }
            }
            
            (input_path.to_string_lossy().to_string(), output_path.to_string_lossy().to_string())
        })
        .collect();
    
    // Perform batch conversion
    let start_time = Instant::now();
    
    if show_progress {
        println!("Starting conversion of {} files with {} parallel workers...", file_pairs.len(), parallel);
    }
    
    let results = engine.convert_batch(&file_pairs).await?;
    
    let duration = start_time.elapsed();
    
    // Process results
    let successful = results.iter().filter(|r| r.is_ok()).count();
    let failed = results.len() - successful;
    
    println!("\n✓ Batch conversion completed in {:.2}s", duration.as_secs_f64());
    println!("  Successful: {}", successful);
    
    if failed > 0 {
        println!("  Failed: {}", failed);
        
        // Show failed files
        for (i, result) in results.iter().enumerate() {
            if let Err(e) = result {
                let (input_path, _) = &file_pairs[i];
                println!("    ✗ {}: {}", input_path, e);
            }
        }
    }
    
    Ok(())
}

/// Handle web server startup
async fn handle_server(host: String, port: u16) -> Result<(), ConversionError> {
    println!("Starting web server on {}:{}", host, port);
    
    // This will be implemented when the web server is ready
    println!("Web server functionality will be available when task 7 is completed");
    
    Ok(())
}

/// Handle configuration validation and updates
async fn handle_config(
    config_path: PathBuf,
    update_prompt: Option<String>,
    preview: bool,
) -> Result<(), ConversionError> {
    info!("Handling configuration operations");
    
    // Load existing configuration
    let config = load_config(Some(&config_path)).await?;
    
    println!("✓ Configuration file is valid: {}", config_path.display());
    
    if let Some(prompt) = update_prompt {
        println!("Updating configuration with natural language prompt...");
        
        let updated_config = update_config_with_prompt(config.clone(), &prompt).await?;
        
        if preview {
            // Show preview of updated configuration
            println!("\nUpdated configuration preview:");
            let yaml_content = serde_yaml::to_string(&updated_config)
                .map_err(|e| ConversionError::Configuration(
                    ConfigError::Validation(format!("Failed to serialize config: {}", e))
                ))?;
            println!("{}", yaml_content);
        } else {
            // Save updated configuration
            let yaml_content = serde_yaml::to_string(&updated_config)
                .map_err(|e| ConversionError::Configuration(
                    ConfigError::Validation(format!("Failed to serialize config: {}", e))
                ))?;
            
            fs::write(&config_path, yaml_content)
                .map_err(ConversionError::Io)?;
            
            println!("✓ Configuration updated and saved to {}", config_path.display());
        }
    }
    
    Ok(())
}

/// Load configuration from file or use default
async fn load_config(config_path: Option<&Path>) -> Result<ConversionConfig, ConversionError> {
    match config_path {
        Some(path) => {
            if !path.exists() {
                return Err(ConversionError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Configuration file not found: {}", path.display()),
                )));
            }
            
            debug!("Loading configuration from: {}", path.display());
            
            let yaml_content = fs::read_to_string(path)
                .map_err(ConversionError::Io)?;
            
            let config: ConversionConfig = serde_yaml::from_str(&yaml_content)
                .map_err(|e| ConversionError::Configuration(
                    ConfigError::InvalidYaml(format!("Failed to parse YAML: {}", e))
                ))?;
            
            // Validate configuration
            config.validate()
                .map_err(|e| ConversionError::Configuration(
                    ConfigError::Validation(e.to_string())
                ))?;
            
            info!("Configuration loaded successfully from {}", path.display());
            Ok(config)
        }
        None => {
            debug!("Using default configuration");
            Ok(ConversionConfig::default())
        }
    }
}

/// Update configuration using natural language prompt
async fn update_config_with_prompt(
    config: ConversionConfig,
    prompt: &str,
) -> Result<ConversionConfig, ConversionError> {
    debug!("Updating configuration with prompt: {}", prompt);
    
    let config_service = ConfigurationService::new();
    
    config_service.update_with_natural_language(&config, prompt)
        .await
        .map_err(ConversionError::Configuration)
}

/// Find all Markdown files in a directory
fn find_markdown_files(dir: &Path, recursive: bool) -> Result<Vec<PathBuf>, ConversionError> {
    let mut markdown_files = Vec::new();
    
    fn collect_files(
        dir: &Path,
        files: &mut Vec<PathBuf>,
        recursive: bool,
    ) -> Result<(), std::io::Error> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "md" || extension == "markdown" {
                        files.push(path);
                    }
                }
            } else if path.is_dir() && recursive {
                collect_files(&path, files, recursive)?;
            }
        }
        Ok(())
    }
    
    collect_files(dir, &mut markdown_files, recursive)
        .map_err(ConversionError::Io)?;
    
    // Sort files for consistent processing order
    markdown_files.sort();
    
    Ok(markdown_files)
}