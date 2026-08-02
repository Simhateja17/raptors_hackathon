//! Native tests for BZ2 NCD.
//!
//! Fixture values are taken from
//! `tests/original/test_compression/test_bz2_ncd.py`. See
//! `docs/behavior-cards/manasa/bz2-ncd.md` for the full behavior card.

use textdistance_port::algorithms::bz2_ncd::Bz2Ncd;
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
    Bz2Ncd.distance(&sequences)
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn matches_frozen_fixture_values() {
    // Per the behavior card: this algorithm only has 2 frozen fixtures,
    // treat these as the full ground truth rather than a sample.
    assert_close(distance("test", "test"), 0.08);
    assert_close(distance("test", "nani"), 0.16);
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
fn distance_is_symmetric() {
    // Shared property from test_common.py::test_simmetry — real compressors
    // are not provably order-invariant the way Sqrt/Entropy NCD are, so
    // this is a meaningful check here, not a redundant one.
    assert_close(distance("test", "nani"), distance("nani", "test"));
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
    assert_close(Bz2Ncd.maximum(&sequences), 1.0);
}

#[test]
fn fails_if_implementation_becomes_a_trivial_constant() {
    assert!(distance("test", "test") != distance("test", "nani"));
}
