//! Native tests for Entropy NCD.
//!
//! Fixture values are taken from
//! `tests/original/test_compression/test_entropy_ncd.py` — note that file
//! asserts on `.similarity()`, not the raw distance, so the values below
//! are converted (`distance = maximum - similarity = 1 - similarity`). See
//! `docs/behavior-cards/manasa/entropy-ncd.md` for the full behavior card.

use textdistance_port::algorithms::entropy_ncd::EntropyNcd;
use textdistance_port::{prepare_sequences, Algorithm, InputSequence, QValue};

fn prepared(left: &str, right: &str) -> Vec<Vec<textdistance_port::Element>> {
    prepare_sequences(
        &[
            InputSequence::Text(left.to_owned()),
            InputSequence::Text(right.to_owned()),
        ],
        QValue::Elements,
    )
    .expect("text preparation should succeed")
}

fn distance(left: &str, right: &str) -> f64 {
    EntropyNcd::default().distance(&prepared(left, right))
}

fn similarity(left: &str, right: &str) -> f64 {
    EntropyNcd::default().similarity(&prepared(left, right))
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn matches_frozen_fixture_values_as_similarity() {
    // Source test file asserts these directly against `.similarity()`.
    assert_close(similarity("test", "test"), 1.0);
    assert_close(similarity("aaa", "bbb"), 0.0);
    assert_close(similarity("test", "nani"), 0.6);
}

#[test]
fn matches_frozen_fixture_values_as_distance() {
    // Same three cases, converted: distance = 1 - similarity.
    assert_close(distance("test", "test"), 0.0);
    assert_close(distance("aaa", "bbb"), 1.0);
    assert_close(distance("test", "nani"), 0.4);
}

#[test]
fn identical_input_naturally_scores_zero_distance() {
    // Unlike Sqrt NCD, this one *does* land on 0 for identical input — but
    // that's a consequence of entropy being proportion-based (doubling a
    // sequence doesn't change its element proportions), not a shortcut.
    assert_close(distance("identical", "identical"), 0.0);
}

#[test]
fn empty_empty_scores_zero() {
    assert_close(distance("", ""), 0.0);
}

#[test]
fn base_parameter_changes_the_result() {
    // `base` must actually be threaded through to the log calls, not
    // hardcoded to base-2 — flagged as a known risk in the behavior card
    // since every frozen fixture uses the default base.
    let base_2 = EntropyNcd::default().distance(&prepared("test", "nani"));
    let base_10 = EntropyNcd {
        coef: 1.0,
        base: 10.0,
    }
    .distance(&prepared("test", "nani"));
    assert!(
        (base_2 - base_10).abs() > 1e-9,
        "changing base should change the result: base2={base_2}, base10={base_10}"
    );
}

#[test]
fn maximum_is_constant_one() {
    let sequences = prepared("test", "nani");
    assert_close(EntropyNcd::default().maximum(&sequences), 1.0);
}

#[test]
fn fails_if_implementation_becomes_a_trivial_constant() {
    assert!(distance("test", "test") != distance("aaa", "bbb"));
}
