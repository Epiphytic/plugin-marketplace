//! AISP Converter - Main conversion API
//!
//! Provides 3-tier conversion with LLM fallback:
//! - Minimal: Direct symbol substitution (0.5-1x tokens)
//! - Standard: + Header + evidence block (1.5-2x tokens)
//! - Full: + All blocks + proofs (4-8x tokens)

use crate::aisp::rosetta::RosettaStone;
use crate::llm::{ClaudeFallback, LlmProvider};
use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Conversion tier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConversionTier {
    Minimal,
    Standard,
    Full,
}

impl std::fmt::Display for ConversionTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversionTier::Minimal => write!(f, "minimal"),
            ConversionTier::Standard => write!(f, "standard"),
            ConversionTier::Full => write!(f, "full"),
        }
    }
}

/// Conversion options
#[derive(Debug, Clone, Default)]
pub struct ConversionOptions {
    /// Force specific tier (auto-detect if None)
    pub tier: Option<ConversionTier>,
    /// Confidence threshold for LLM fallback (default: 0.8)
    pub confidence_threshold: Option<f64>,
    /// Enable LLM fallback
    pub enable_llm_fallback: bool,
    /// LLM model to use (default: sonnet)
    pub llm_model: Option<String>,
}

/// Token statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenStats {
    pub input: usize,
    pub output: usize,
    pub ratio: f64,
}

/// Conversion result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionResult {
    /// Converted AISP output
    pub output: String,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// Words that couldn't be mapped
    pub unmapped: Vec<String>,
    /// Conversion tier used
    pub tier: ConversionTier,
    /// Token statistics
    pub tokens: TokenStats,
    /// Whether LLM fallback was used
    #[serde(default)]
    pub used_fallback: bool,
}

/// AISP Converter
pub struct AispConverter;

impl AispConverter {
    /// Convert prose to AISP with specified options
    pub fn convert(prose: &str, options: Option<ConversionOptions>) -> ConversionResult {
        let opts = options.unwrap_or_default();
        let tier = opts.tier.unwrap_or_else(|| Self::detect_tier(prose));

        let result = match tier {
            ConversionTier::Minimal => Self::convert_minimal(prose),
            ConversionTier::Standard => Self::convert_standard(prose),
            ConversionTier::Full => Self::convert_full(prose),
        };

        ConversionResult {
            tokens: TokenStats {
                input: prose.len(),
                output: result.output.len(),
                ratio: if prose.is_empty() {
                    0.0
                } else {
                    (result.output.len() as f64 / prose.len() as f64 * 100.0).round() / 100.0
                },
            },
            ..result
        }
    }

    /// Convert prose to AISP with LLM fallback support (Async)
    pub async fn convert_async(prose: &str, options: Option<ConversionOptions>) -> ConversionResult {
        let result = Self::convert(prose, options.clone());
        let opts = options.unwrap_or_default();
        let threshold = opts.confidence_threshold.unwrap_or(0.8);

        if opts.enable_llm_fallback && result.confidence < threshold {
            let provider = if let Some(model) = &opts.llm_model {
                ClaudeFallback::with_model(model)
            } else {
                ClaudeFallback::new()
            };
            
            if provider.is_available().await
                && let Ok(llm_result) = provider
                    .convert(prose, result.tier, &result.unmapped, Some(&result.output))
                    .await
                {
                    return llm_result.to_conversion_result(result.tier, prose.len());
                }
        }

        result
    }

    /// Auto-detect appropriate tier based on prose complexity
    pub fn detect_tier(prose: &str) -> ConversionTier {
        let word_count = prose.split_whitespace().count();

        let types_regex =
            Regex::new(r"(?i)\b(type|class|struct|interface|schema|model|entity)\b").unwrap();
        let rules_regex = Regex::new(
            r"(?i)\b(must|should|always|never|require|ensure|guarantee|constraint|rule)\b",
        )
        .unwrap();
        let proof_regex =
            Regex::new(r"(?i)\b(prove|verify|validate|certify|demonstrate|qed|proven)\b").unwrap();
        let complex_regex =
            Regex::new(r"(?i)\b(for all|there exists|if and only if|implies|therefore)\b").unwrap();
        let api_regex =
            Regex::new(r"(?i)\b(api|endpoint|route|controller|handler|service)\b").unwrap();
        let contractor_regex =
            Regex::new(r"(?i)\b(delta|invariant|precondition|postcondition|requires|ensures)\b")
                .unwrap();
        let intent_regex =
            Regex::new(r"(?i)\b(intent|goal|purpose|objective|fitness|risk|utility)\b").unwrap();

        let has_types = types_regex.is_match(prose);
        let has_rules = rules_regex.is_match(prose);
        let has_proof = proof_regex.is_match(prose);
        let has_complex = complex_regex.is_match(prose);
        let has_api = api_regex.is_match(prose);
        let has_contractor = contractor_regex.is_match(prose);
        let has_intent = intent_regex.is_match(prose);

        // Full tier: proofs, contractors, intents required, or types + rules together
        if has_proof || has_contractor || has_intent || (has_types && has_rules) {
            return ConversionTier::Full;
        }

        // Standard tier: types OR rules OR complex logic OR API OR longer text
        if has_types || has_rules || has_complex || has_api || word_count > 20 {
            return ConversionTier::Standard;
        }

        // Minimal tier: simple, short prose
        ConversionTier::Minimal
    }

    /// Minimal conversion - direct Rosetta mapping
    fn convert_minimal(prose: &str) -> ConversionResult {
        let (output, mapped_chars, unmapped) = RosettaStone::convert(prose);
        let confidence = RosettaStone::confidence(prose.len(), mapped_chars);

        ConversionResult {
            output,
            confidence,
            unmapped,
            tier: ConversionTier::Minimal,
            tokens: TokenStats {
                input: 0,
                output: 0,
                ratio: 0.0,
            },
            used_fallback: false,
        }
    }

    /// Standard conversion - minimal + header + evidence
    fn convert_standard(prose: &str) -> ConversionResult {
        let minimal = Self::convert_minimal(prose);
        let domain = Self::extract_domain(prose);
        let date = Utc::now().format("%Y-%m-%d").to_string();

        let output = format!(
            r#"𝔸5.1.{domain}@{date}
γ≔{domain}

⟦Ω:Meta⟧{{
  domain≜{domain}
  version≜1.0.0
}}

⟦Σ:Types⟧{{
  ∅
}}

⟦Γ:Rules⟧{{
  ∅
}}

⟦Λ:Funcs⟧{{
  {body}
}}

⟦Ε⟧⟨δ≜0.70;τ≜◊⁺⟩"#,
            domain = domain,
            date = date,
            body = minimal.output
        );

        ConversionResult {
            output,
            confidence: minimal.confidence,
            unmapped: minimal.unmapped,
            tier: ConversionTier::Standard,
            tokens: TokenStats {
                input: 0,
                output: 0,
                ratio: 0.0,
            },
            used_fallback: false,
        }
    }

    /// Full conversion - complete AISP document
    fn convert_full(prose: &str) -> ConversionResult {
        let minimal = Self::convert_minimal(prose);
        let domain = Self::extract_domain(prose);
        let date = Utc::now().format("%Y-%m-%d").to_string();
        let types = Self::infer_types(prose);
        let rules = Self::infer_rules(prose);
        let errors = Self::infer_errors(prose);

        let output = format!(
            r#"𝔸5.1.{domain}@{date}
γ≔{domain}.definitions
ρ≔⟨{domain},types,rules⟩

⟦Ω:Meta⟧{{
  domain≜{domain}
  version≜1.0.0
  ∀D∈AISP:Ambig(D)<0.02
}}

⟦Σ:Types⟧{{
{types}
}}

⟦Γ:Rules⟧{{
{rules}
}}

⟦Λ:Funcs⟧{{
  {body}
}}

⟦Χ:Errors⟧{{
{errors}
}}

⟦Ε⟧⟨δ≜0.82;φ≜100;τ≜◊⁺⁺;⊢valid;∎⟩"#,
            domain = domain,
            date = date,
            types = types,
            rules = rules,
            body = minimal.output,
            errors = errors
        );

        ConversionResult {
            output,
            confidence: minimal.confidence,
            unmapped: minimal.unmapped,
            tier: ConversionTier::Full,
            tokens: TokenStats {
                input: 0,
                output: 0,
                ratio: 0.0,
            },
            used_fallback: false,
        }
    }

    /// Extract domain from prose
    fn extract_domain(prose: &str) -> &'static str {
        let lower = prose.to_lowercase();

        if lower.contains("api") || lower.contains("endpoint") {
            return "api";
        }
        if lower.contains("auth") || lower.contains("login") || lower.contains("password") {
            return "auth";
        }
        if lower.contains("math") || lower.contains("sum") || lower.contains("calculate") {
            return "math";
        }
        if lower.contains("database") || lower.contains("store") || lower.contains("persist") {
            return "data";
        }
        if lower.contains("file") || lower.contains("read") || lower.contains("write") {
            return "io";
        }
        if lower.contains("test") || lower.contains("assert") || lower.contains("expect") {
            return "test";
        }
        if lower.contains("user") {
            return "user";
        }

        "domain"
    }

    /// Infer types from prose
    fn infer_types(prose: &str) -> String {
        let lower = prose.to_lowercase();
        let mut types = Vec::new();

        if lower.contains("number") || lower.contains("integer") || lower.contains("count") {
            types.push("  ℕ≜natural_numbers");
        }
        if lower.contains("string") || lower.contains("text") || lower.contains("name") {
            types.push("  𝕊≜strings");
        }
        if lower.contains("bool")
            || lower.contains("flag")
            || lower.contains("true")
            || lower.contains("false")
        {
            types.push("  𝔹≜booleans");
        }
        if lower.contains("function") || lower.contains("lambda") {
            types.push("  Fn⟨A,B⟩≜A→B");
        }
        if lower.contains("user") {
            types.push("  User≜⟨id:ℕ,name:𝕊⟩");
        }
        if lower.contains("list") || lower.contains("array") {
            types.push("  List⟨T⟩≜⟨items:T*⟩");
        }

        if types.is_empty() {
            types.push("  T≜⟨value:Any⟩");
        }

        types.join("\n")
    }

    /// Infer rules from prose
    fn infer_rules(prose: &str) -> String {
        let lower = prose.to_lowercase();
        let mut rules = Vec::new();

        if lower.contains("constant") || lower.contains("immutable") {
            rules.push("  ∀c∈Const:c.immutable≡⊤");
        }
        if lower.contains("valid") || lower.contains("check") {
            rules.push("  ∀x:T:valid(x)⇒accept(x)");
        }
        if lower.contains("all") || lower.contains("every") {
            rules.push("  ∀x∈S:P(x)");
        }
        if lower.contains("must") || lower.contains("require") {
            rules.push("  ∀x:T:require(x)⇒proceed(x)");
        }
        if lower.contains("unique") {
            rules.push("  ∃!x:T:unique(x)");
        }
        if lower.contains("admin") {
            rules.push("  ∀u∈User:u.admin≡⊤⇒allow(u)");
        }

        // New Contractor detections
        if lower.contains("invariant") || lower.contains("always true") {
            rules.push("  Inv(s)≜always(s)");
        }
        if lower.contains("precondition") || lower.contains("before") {
            rules.push("  Pre(f)≜req(args)");
        }
        if lower.contains("postcondition") || lower.contains("after") || lower.contains("ensures") {
            rules.push("  Post(f)≜guarantee(result)");
        }
        if lower.contains("delta") || lower.contains("change") {
            rules.push("  Δ(s)≜s'−s");
        }

        if rules.is_empty() {
            rules.push("  ∀x:T:⊤");
        }

        rules.join("\n")
    }

    /// Infer errors from prose
    fn infer_errors(prose: &str) -> String {
        let lower = prose.to_lowercase();
        let mut errors = Vec::new();

        if lower.contains("error") || lower.contains("exception") {
            errors.push("  E≜GenericError");
        }
        if lower.contains("fail") || lower.contains("failure") {
            errors.push("  fail(x)⇒⊥");
        }
        if lower.contains("crash") || lower.contains("panic") {
            errors.push("  crash⇒⊥⊥");
        }
        if lower.contains("not found") || lower.contains("missing") {
            errors.push("  NotFound⇒∅");
        }
        if lower.contains("unauthorized") || lower.contains("forbidden") || lower.contains("denied")
        {
            errors.push("  AuthError⇒⊘");
        }

        if errors.is_empty() {
            errors.push("  ∅");
        }

        errors.join("\n")
    }

    /// Convert AISP back to prose
    pub fn to_prose(aisp: &str) -> String {
        RosettaStone::to_prose(aisp)
    }

    /// Validate AISP document using the aisp crate
    pub fn validate(aisp: &str) -> aisp::ValidationResult {
        aisp::validate(aisp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_tier_minimal() {
        assert_eq!(
            AispConverter::detect_tier("Define x as 5"),
            ConversionTier::Minimal
        );
    }

    #[test]
    fn test_detect_tier_standard() {
        assert_eq!(
            AispConverter::detect_tier("The user must provide valid authentication"),
            ConversionTier::Standard
        );
    }

    #[test]
    fn test_detect_tier_full() {
        assert_eq!(
            AispConverter::detect_tier("Define a type User and verify all users are valid"),
            ConversionTier::Full
        );
    }

    #[test]
    fn test_convert_minimal() {
        let result = AispConverter::convert("Define x as 5", None);
        assert!(result.output.contains("≜"));
        assert_eq!(result.tier, ConversionTier::Minimal);
    }

    #[test]
    fn test_convert_standard() {
        let result = AispConverter::convert(
            "Define x as 5",
            Some(ConversionOptions {
                tier: Some(ConversionTier::Standard),
                ..Default::default()
            }),
        );
        assert!(result.output.contains("𝔸5.1"));
        assert!(result.output.contains("⟦Ω:Meta⟧"));
        assert!(result.output.contains("⟦Σ:Types⟧"));
        assert!(result.output.contains("⟦Γ:Rules⟧"));
        assert!(result.output.contains("⟦Λ:Funcs⟧"));
    }

    #[test]
    fn test_convert_full() {
        let result = AispConverter::convert(
            "Define x as 5",
            Some(ConversionOptions {
                tier: Some(ConversionTier::Full),
                ..Default::default()
            }),
        );
        assert!(result.output.contains("⟦Ω:Meta⟧"));
        assert!(result.output.contains("⟦Σ:Types⟧"));
        assert!(result.output.contains("⟦Γ:Rules⟧"));
        assert!(result.output.contains("⟦Λ:Funcs⟧"));
        assert!(result.output.contains("⟦Χ:Errors⟧"));
    }
}
