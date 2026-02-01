//! Gear Core Library
//!
//! Core functionality for the Gear plugin marketplace including:
//! - AISP conversion (prose ↔ symbolic notation)
//! - LLM fallback via Claude SDK
//! - Configuration management
//! - Shared utilities

pub mod aisp;
pub mod config;
pub mod llm;

pub use aisp::{AispConverter, ConversionOptions, ConversionResult, ConversionTier, TokenStats};
pub use config::Config;
pub use llm::{ClaudeFallback, LlmProvider, LlmResult};

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::aisp::{
        AispConverter, ConversionOptions, ConversionResult, ConversionTier, TokenStats,
    };
    pub use crate::config::Config;
    pub use crate::llm::{ClaudeFallback, LlmProvider, LlmResult};
    pub use anyhow::Result;
}
