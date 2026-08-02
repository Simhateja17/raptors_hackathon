//! Native tests for Square-root NCD.
//!
//! Fixture values are taken from
//! `tests/original/test_compression/test_sqrt_ncd.py`. See
//! `docs/behavior-cards/manasa/sqrt-ncd.md` for the full behavior card.

use textdistance_port::algorithms::sqrt_ncd::SqrtNcd;
use textdistance_port::{prepare_sequences, Algorithm, InputSequence, QValue};

fn distance(left: &str, right: &str) -> f64 {
    let sequences = prepare_sequences(
        &[
            InputSequence::Text(left.to_owned()),
            InputSequence::Text(right.to_owned()),
        ],
        QValue::Elements,
    )
    .expect("text preparation should succeed");
    SqrtNcd.distance(&sequences)
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn matches_frozen_fixture_values() {
    assert_close(distance("test", "test"), 0.41421356237309503);
    assert_close(distance("test", "nani"), 1.0);
}

#[test]
fn identical_input_is_not_zero() {
    // Deliberate: unlike Jaro/Editex, NCD algorithms never shortcut
    // identical inputs to zero. sqrt(count) is not scale-invariant under
    // doubling, so identical non-empty strings do NOT score 0 here — this
    // guards against "simplifying" the implementation by adding an
    // identical-input shortcut that would silently break this algorithm.
    assert_close(distance("test", "test"), 0.41421356237309503);
    assert!(distance("test", "test") != 0.0);
}

#[test]
fn empty_empty_scores_zero() {
    assert_close(distance("", ""), 0.0);
}

#[test]
fn monotonicity_holds() {
    // Shared property from tests/original/test_compression/test_common.py.
    let same = distance("test", "test");
    let similar = distance("test", "text");
    let different = distance("test", "nani");
    assert!(same <= similar, "{same} <= {similar}");
    assert!(similar <= different, "{similar} <= {different}");
}

#[test]
fn maximum_is_constant_one() {
    let sequences = prepare_sequences(
        &[
            InputSequence::Text("test".to_owned()),
            InputSequence::Text("nani".to_owned()),
        ],
        QValue::Elements,
    )
    .expect("text preparation should succeed");
    assert_close(SqrtNcd.maximum(&sequences), 1.0);
}

#[test]
fn fails_if_implementation_becomes_a_trivial_constant() {
    assert!(distance("test", "test") != distance("test", "nani"));
}
