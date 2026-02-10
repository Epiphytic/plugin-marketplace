//! Gear Core CLI
//!
//! Command-line interface for AISP conversion and gear utilities.

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use gear_core::prelude::*;
use std::io::{self, Read};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "gear-core")]
#[command(about = "Gear plugin marketplace core utilities")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Copy, ValueEnum)]
enum TierArg {
    Minimal,
    Standard,
    Full,
    Auto,
}

impl TierArg {
    fn to_conversion_tier(self) -> Option<gear_core::ConversionTier> {
        match self {
            TierArg::Minimal => Some(gear_core::ConversionTier::Minimal),
            TierArg::Standard => Some(gear_core::ConversionTier::Standard),
            TierArg::Full => Some(gear_core::ConversionTier::Full),
            TierArg::Auto => None,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum OutputFormat {
    /// Plain text output
    Text,
    /// JSON output with metadata
    Json,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert prose to AISP
    Convert {
        /// Input prose (reads from stdin if not provided)
        #[arg(short, long)]
        input: Option<String>,

        /// Input file
        #[arg(short = 'f', long)]
        file: Option<PathBuf>,

        /// Output file (stdout if not provided)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Conversion tier
        #[arg(short, long, value_enum)]
        tier: Option<TierArg>,

        /// Output format
        #[arg(short = 'r', long, value_enum, default_value = "text")]
        format: OutputFormat,

        /// Enable LLM fallback for low-confidence conversions
        #[arg(long)]
        llm_fallback: bool,

        /// Confidence threshold for LLM fallback (0.0-1.0)
        #[arg(long)]
        confidence_threshold: Option<f64>,

        /// LLM model to use (haiku, sonnet, opus)
        #[arg(long)]
        model: Option<String>,

        /// Use AISP symbolic prompt instead of English prompt
        #[arg(long)]
        aisp_prompt: bool,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Convert AISP back to prose
    ToProse {
        /// Input AISP
        #[arg(short, long)]
        input: Option<String>,

        /// Input file
        #[arg(short = 'f', long)]
        file: Option<PathBuf>,
    },

    /// Validate AISP document
    Validate {
        /// Input AISP
        #[arg(short, long)]
        input: Option<String>,

        /// Input file
        #[arg(short = 'f', long)]
        file: Option<PathBuf>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Detect appropriate conversion tier
    Triage {
        /// Input prose (reads from stdin if not provided)
        #[arg(short, long)]
        input: Option<String>,
    },

    /// Look up a symbol for a prose pattern
    Lookup {
        /// Prose pattern to look up
        pattern: String,
    },

    /// Look up prose for a symbol
    Reverse {
        /// AISP symbol to look up
        symbol: String,
    },

    /// List all available symbols
    Symbols {
        /// Filter by category
        #[arg(short, long)]
        category: Option<String>,
    },

    /// Show all available categories
    Categories,

    /// Perform round-trip conversion to test semantic preservation
    RoundTrip {
        /// Prose text to test (reads from stdin if not provided)
        #[arg(short, long)]
        input: Option<String>,

        /// Number of round-trips to perform
        #[arg(short = 'n', long, default_value = "5")]
        rounds: usize,
    },

    /// Configuration management
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Get a configuration value
    Get {
        /// Configuration key (e.g., aisp.confidence_threshold)
        key: String,
    },

    /// Set a configuration value
    Set {
        /// Configuration key
        key: String,
        /// Value to set
        value: String,
        /// Set globally
        #[arg(long)]
        global: bool,
    },

    /// Show all configuration
    Show,

    /// Show configuration file path
    Path {
        /// Show global path
        #[arg(long)]
        global: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Convert {
            input,
            file,
            output,
            tier,
            format,
            llm_fallback,
            confidence_threshold,
            model,
            aisp_prompt,
            verbose,
        } => {
            let config = Config::load()?;
            let prose = get_input(input, file)?;

            // Determine effective values (CLI > Config > Default)
            let effective_threshold = confidence_threshold.unwrap_or(config.aisp.confidence_threshold);
            let effective_fallback = llm_fallback || config.aisp.enable_llm_fallback;
            let effective_model = model.or(Some(config.llm.default_model));

            let tier_opt = match tier {
                Some(t) => t.to_conversion_tier(),
                None => match config.aisp.default_tier.as_str() {
                    "minimal" => Some(gear_core::ConversionTier::Minimal),
                    "standard" => Some(gear_core::ConversionTier::Standard),
                    "full" => Some(gear_core::ConversionTier::Full),
                    _ => None, // auto-detect
                },
            };

            let options = gear_core::ConversionOptionsExt {
                tier: tier_opt,
                confidence_threshold: Some(effective_threshold),
                enable_llm_fallback: effective_fallback,
                llm_model: effective_model,
                use_aisp_prompt: aisp_prompt,
            };

            let result = gear_core::convert_with_fallback(&prose, Some(options)).await;

            match format {
                OutputFormat::Text => {
                    if verbose {
                        eprintln!("Tier: {}", result.tier);
                        eprintln!("Confidence: {:.1}%", result.confidence * 100.0);
                        eprintln!(
                            "Tokens: {} → {} ({:.2}x)",
                            result.tokens.input, result.tokens.output, result.tokens.ratio
                        );
                        if result.used_fallback {
                            eprintln!("LLM fallback: used");
                        }
                        if !result.unmapped.is_empty() {
                            eprintln!("Unmapped: {:?}", result.unmapped);
                        }
                        eprintln!("---");
                    }
                    write_output(&result.output, output)?;
                }
                OutputFormat::Json => {
                    let json_output = serde_json::to_string_pretty(&result)?;
                    write_output(&json_output, output)?;
                }
            }
        }

        Commands::ToProse { input, file } => {
            let aisp = get_input(input, file)?;
            let prose = gear_core::AispConverter::to_prose(&aisp);
            println!("{}", prose);
        }

        Commands::Validate { input, file, json } => {
            let aisp = get_input(input, file)?;
            let result = gear_core::AispConverter::validate(&aisp);

            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "valid": result.valid,
                        "tier": format!("{:?}", result.tier),
                        "delta": result.delta,
                        "pure_density": result.pure_density,
                        "ambiguity": result.ambiguity,
                    })
                );
            } else if result.valid {
                println!("Valid AISP document");
                println!("Tier: {:?}", result.tier);
                println!("Delta: {:.2}", result.delta);
                println!("Pure Density: {:.2}", result.pure_density);
            } else {
                eprintln!("Invalid AISP document");
                std::process::exit(1);
            }
        }

        Commands::Triage { input } => {
            let prose = match input {
                Some(text) => text,
                None => {
                    let mut buf = String::new();
                    io::stdin().read_to_string(&mut buf)?;
                    buf
                }
            };

            let tier = gear_core::AispConverter::detect_tier(&prose);
            println!("Recommended tier: {}", tier);
        }

        Commands::Lookup { pattern } => {
            match gear_core::prose_to_symbol(&pattern) {
                Some(symbol) => println!("{}", symbol),
                None => {
                    eprintln!("No symbol found for pattern: {}", pattern);
                    std::process::exit(1);
                }
            }
        }

        Commands::Reverse { symbol } => {
            match gear_core::symbol_to_prose(&symbol) {
                Some(prose) => println!("{}", prose),
                None => {
                    eprintln!("No prose found for symbol: {}", symbol);
                    std::process::exit(1);
                }
            }
        }

        Commands::Symbols { category } => {
            match category {
                Some(cat) => {
                    let symbols = gear_core::symbols_by_category(&cat);
                    if symbols.is_empty() {
                        eprintln!("No symbols found for category: {}", cat);
                        eprintln!("Available categories: {:?}", gear_core::get_all_categories());
                        std::process::exit(1);
                    }
                    for symbol in symbols {
                        if let Some(prose) = gear_core::symbol_to_prose(symbol) {
                            println!("{} → {}", symbol, prose);
                        } else {
                            println!("{}", symbol);
                        }
                    }
                }
                None => {
                    for category in gear_core::get_all_categories() {
                        println!("\n=== {} ===", category);
                        for symbol in gear_core::symbols_by_category(category) {
                            if let Some(prose) = gear_core::symbol_to_prose(symbol) {
                                println!("  {} → {}", symbol, prose);
                            } else {
                                println!("  {}", symbol);
                            }
                        }
                    }
                }
            }
        }

        Commands::Categories => {
            for category in gear_core::get_all_categories() {
                println!("{}", category);
            }
        }

        Commands::RoundTrip { input, rounds } => {
            let original = match input {
                Some(text) => text,
                None => {
                    let mut buf = String::new();
                    io::stdin().read_to_string(&mut buf)?;
                    buf.trim().to_string()
                }
            };
            let mut current = original.clone();

            println!("Original: {}", original);
            println!();

            for i in 1..=rounds {
                let (aisp, mapped_chars, _) = gear_core::RosettaStone::convert(&current);
                let prose = gear_core::RosettaStone::to_prose(&aisp);
                let similarity = gear_core::RosettaStone::semantic_similarity(&original, &prose);
                let confidence = gear_core::RosettaStone::confidence(current.len(), mapped_chars);

                println!(
                    "Round {} (confidence: {:.1}%, similarity: {:.1}%):",
                    i,
                    confidence * 100.0,
                    similarity * 100.0
                );
                println!("  AISP: {}", aisp);
                println!("  Prose: {}", prose);
                println!();

                current = prose;
            }

            let final_similarity = gear_core::RosettaStone::semantic_similarity(&original, &current);
            println!("Final semantic similarity: {:.1}%", final_similarity * 100.0);

            if final_similarity < 0.30 {
                eprintln!("Warning: Semantic drift exceeded acceptable threshold");
                std::process::exit(1);
            }
        }

        Commands::Config { action } => match action {
            ConfigAction::Get { key } => {
                let config = Config::load()?;
                match config.get(&key) {
                    Some(value) => println!("{}", value),
                    None => {
                        eprintln!("Unknown key: {}", key);
                        std::process::exit(1);
                    }
                }
            }

            ConfigAction::Set { key, value, global } => {
                eprintln!(
                    "Config set not yet implemented: {} = {} (global: {})",
                    key, value, global
                );
            }

            ConfigAction::Show => {
                let config = Config::load()?;
                println!("{}", toml::to_string_pretty(&config)?);
            }

            ConfigAction::Path { global } => {
                if global {
                    match Config::global_config_path() {
                        Some(path) => println!("{}", path.display()),
                        None => eprintln!("Could not determine global config path"),
                    }
                } else {
                    println!(".gear/config.toml");
                }
            }
        },
    }

    Ok(())
}

/// Get input from argument, file, or stdin
fn get_input(input: Option<String>, file: Option<PathBuf>) -> Result<String> {
    if let Some(text) = input {
        return Ok(text);
    }

    if let Some(path) = file {
        return Ok(std::fs::read_to_string(path)?);
    }

    // Read from stdin
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    Ok(buf)
}

/// Write output to file or stdout
fn write_output(content: &str, output: Option<PathBuf>) -> Result<()> {
    match output {
        Some(path) => {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, content)?;
        }
        None => println!("{}", content),
    }
    Ok(())
}
