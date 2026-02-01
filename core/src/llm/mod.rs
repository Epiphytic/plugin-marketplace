//! LLM Fallback Module
//!
//! Provides Claude SDK integration for complex conversions
//! that can't be handled by deterministic Rosetta mappings.

use crate::aisp::{ConversionResult, ConversionTier};
use anyhow::Result;
use async_trait::async_trait;

mod claude;

pub use claude::ClaudeFallback;

/// LLM provider trait for fallback conversions
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Convert prose to AISP using LLM
    async fn convert(
        &self,
        prose: &str,
        tier: ConversionTier,
        unmapped: &[String],
        partial_output: Option<&str>,
    ) -> Result<LlmResult>;

    /// Check if provider is available
    async fn is_available(&self) -> bool;
}

/// LLM conversion result
#[derive(Debug, Clone)]
pub struct LlmResult {
    pub output: String,
    pub provider: String,
    pub model: String,
    pub tokens_used: Option<usize>,
}

impl LlmResult {
    /// Convert to ConversionResult
    pub fn to_conversion_result(self, tier: ConversionTier, input_len: usize) -> ConversionResult {
        ConversionResult {
            output: self.output.clone(),
            confidence: 0.95, // LLM output assumed high confidence
            unmapped: vec![],
            tier,
            tokens: crate::aisp::TokenStats {
                input: input_len,
                output: self.output.len(),
                ratio: if input_len == 0 {
                    0.0
                } else {
                    (self.output.len() as f64 / input_len as f64 * 100.0).round() / 100.0
                },
            },
            used_fallback: true,
        }
    }
}
