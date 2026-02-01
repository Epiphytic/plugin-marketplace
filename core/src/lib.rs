//! Gear Core Library
//!
//! Core functionality for the Gear plugin marketplace including:
//! - AISP conversion (prose ↔ symbolic notation) via rosetta-aisp
//! - LLM fallback via Claude SDK
//! - Configuration management
//! - Shared utilities

pub mod config;
pub mod convert;
pub mod llm;

// Re-export rosetta-aisp types
pub use rosetta_aisp::{
    AispConverter, ConversionOptions, ConversionResult, ConversionTier, RosettaStone, TokenStats,
};

// Re-export rosetta module for direct access
pub use rosetta_aisp as rosetta;

// Re-export extended conversion with LLM fallback
pub use convert::{convert_with_fallback, ConversionOptionsExt};

pub use config::Config;
pub use llm::{ClaudeFallback, LlmProvider, LlmResult};

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::config::Config;
    pub use crate::convert::{convert_with_fallback, ConversionOptionsExt};
    pub use crate::llm::{ClaudeFallback, LlmProvider, LlmResult};
    pub use anyhow::Result;
    pub use rosetta_aisp::{
        AispConverter, ConversionOptions, ConversionResult, ConversionTier, RosettaStone,
        TokenStats,
    };
}
