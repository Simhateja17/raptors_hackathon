//! Rust implementation boundary for the TextDistance port.
//!
//! The algorithm modules are intentionally added one file at a time after the
//! shared contract is frozen.  This crate must remain usable without Python.

pub mod algorithms;
pub mod core;

pub use core::{
    all_identical, maximum_length, normalize_distance, normalize_similarity, prepare_sequences,
    Algorithm, Element, InputError, InputSequence, PreparedSequence, QValue, ScoreMode, Sequence,
};

/// Port version.  The source package has an existing 4.6.2/4.6.3 metadata
/// mismatch; this independent port version is intentionally explicit until
/// compatibility packaging is finalized.
pub const VERSION: &str = "0.1.0";
