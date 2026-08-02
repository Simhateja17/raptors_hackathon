//! Cargo-discoverable native tests for Poojitha's Sorensen packet.
//!
//! The implementation and focused assertions remain in the owned packet under
//! rust/tests/algorithms/; including it here makes Cargo execute that exact
//! coverage without changing the shared manifest.
include!("../rust/tests/algorithms/sorensen.rs");
