//! Extended conversion with LLM fallback support
//!
//! Wraps rosetta-aisp conversion with Claude SDK fallback for low-confidence results.

use crate::llm::{ClaudeFallback, LlmProvider};
use rosetta_aisp::{AispConverter as BaseConverter, ConversionResult, ConversionTier};

/// Extended conversion options with LLM fallback support
#[derive(Debug, Clone, Default)]
pub struct ConversionOptionsExt {
    /// Force specific tier (auto-detect if None)
    pub tier: Option<ConversionTier>,
    /// Confidence threshold for LLM fallback (default: 0.8)
    pub confidence_threshold: Option<f64>,
    /// Enable LLM fallback
    pub enable_llm_fallback: bool,
    /// LLM model to use (default: sonnet)
    pub llm_model: Option<String>,
}

/// Convert prose to AISP with optional LLM fallback
pub async fn convert_with_fallback(
    prose: &str,
    options: Option<ConversionOptionsExt>,
) -> ConversionResult {
    let opts = options.unwrap_or_default();

    // Convert using rosetta-aisp's ConversionOptions
    let base_options = rosetta_aisp::ConversionOptions {
        tier: opts.tier,
        confidence_threshold: opts.confidence_threshold,
    };

    let result = BaseConverter::convert(prose, Some(base_options));
    let threshold = opts.confidence_threshold.unwrap_or(0.8);

    // Check if LLM fallback is needed
    if opts.enable_llm_fallback && result.confidence < threshold {
        let provider = if let Some(model) = &opts.llm_model {
            ClaudeFallback::with_model(model)
        } else {
            ClaudeFallback::new()
        };

        if provider.is_available().await {
            if let Ok(llm_result) = provider
                .convert(prose, result.tier, &result.unmapped, Some(&result.output))
                .await
            {
                return llm_result.to_conversion_result(result.tier, prose.len());
            }
        }
    }

    result
}
