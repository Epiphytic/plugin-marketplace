//! AISP Conversion Module
//!
//! Provides conversion between prose and AISP symbolic notation.
//! Uses deterministic Rosetta Stone mappings where possible,
//! falling back to Claude LLM for complex conversions.

mod converter;
pub mod rosetta;

pub use converter::{
    AispConverter, ConversionOptions, ConversionResult, ConversionTier, TokenStats,
};
pub use rosetta::{RosettaStone, ROSETTA};
