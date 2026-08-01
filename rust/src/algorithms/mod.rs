//! Algorithm module registry.
//!
//! Individual algorithm files are added by their named owners after
//! `API-FREEZE`.  Keeping the registry in one file prevents parallel branches
//! from editing the same module declaration surface.

/// Edit-distance algorithms.
pub mod edit {}

/// Token/set-based algorithms.
pub mod token {}

/// Sequence-based algorithms.
pub mod sequence {}

/// Compression-based algorithms.
pub mod compression {}

/// Phonetic algorithms.
pub mod phonetic {}

/// Simple comparison algorithms.
pub mod simple {}
