//! Native tests for Jaro.
//!
//! Fixture values are taken from `tests/original/test_edit/test_jaro.py`
//! (the frozen original suite calls `JaroWinkler(winklerize=False, ...)`,
//! which is exactly what `Jaro` is). See `docs/behavior-cards/manasa/jaro.md`
//! for the full behavior card.

use textdistance_port::algorithms::jaro::Jaro;
use textdistance_port::{prepare_sequences, Algorithm, InputSequence, QValue};

fn similarity(left: &str, right: &str) -> f64 {
    let sequences = prepare_sequences(
        &[
            InputSequence::Text(left.to_owned()),
            InputSequence::Text(right.to_owned()),
        ],
        QValue::Elements,
    )
    .expect("text preparation should succeed");
    Jaro::default().similarity(&sequences)
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn matches_frozen_fixture_values() {
    let cases: &[(&str, &str, f64)] = &[
        ("hello", "haloa", 0.7333333333333334),
        ("fly", "ant", 0.0),
        ("frog", "fog", 0.9166666666666666),
        ("ATCG", "TAGC", 0.8333333333333334),
        ("MARTHA", "MARHTA", 0.944444444),
        ("DWAYNE", "DUANE", 0.822222222),
        ("DIXON", "DICKSONX", 0.7666666666666666),
        (
            "Sint-Pietersplein 6, 9000 Gent",
            "Test 10, 1010 Brussel",
            0.5182539682539683,
        ),
    ];

    for &(left, right, expected) in cases {
        assert_close(similarity(left, right), expected);
    }
}

#[test]
fn empty_inputs_score_zero() {
    assert_close(similarity("", ""), 0.0);
    assert_close(similarity("hello", ""), 0.0);
    assert_close(similarity("", "hello"), 0.0);
}

#[test]
fn identical_strings_score_one() {
    assert_close(similarity("identical", "identical"), 1.0);
}

#[test]
fn no_common_characters_scores_zero() {
    assert_close(similarity("fly", "ant"), 0.0);
}

#[test]
fn maximum_is_constant_one_not_sequence_length() {
    // Source: `JaroWinkler.maximum` always returns `1`, unlike the trait's
    // default (longest-sequence length). A long, very different pair
    // exercises this: if `maximum` were wrongly derived from length, the
    // normalized methods below would be wrong even though raw similarity
    // is fine.
    let sequences = prepare_sequences(
        &[
            InputSequence::Text("Sint-Pietersplein 6, 9000 Gent".to_owned()),
            InputSequence::Text("Test 10, 1010 Brussel".to_owned()),
        ],
        QValue::Elements,
    )
    .expect("text preparation should succeed");
    let jaro = Jaro::default();
    assert_close(jaro.maximum(&sequences), 1.0);
    assert_close(
        jaro.normalized_similarity(&sequences),
        jaro.similarity(&sequences),
    );
}

#[test]
fn fails_if_implementation_becomes_a_trivial_constant() {
    // Sanity check demanded by the human review checklist: a fake
    // implementation that always returns e.g. 0.5 must fail this suite.
    assert!(similarity("hello", "haloa") != 0.5);
    assert!(similarity("fly", "ant") != 0.5);
}
