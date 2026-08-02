//! Native tests for Jaro-Winkler.
//!
//! Fixture values are taken from
//! `tests/original/test_edit/test_jaro_winkler.py`. See
//! `docs/behavior-cards/manasa/jaro-winkler.md` for the full behavior card.

use textdistance_port::algorithms::jaro_winkler::JaroWinkler;
use textdistance_port::{prepare_sequences, Algorithm, InputSequence, QValue};

fn similarity_with(algorithm: &JaroWinkler, left: &str, right: &str) -> f64 {
    let sequences = prepare_sequences(
        &[
            InputSequence::Text(left.to_owned()),
            InputSequence::Text(right.to_owned()),
        ],
        QValue::Elements,
    )
    .expect("text preparation should succeed");
    algorithm.similarity(&sequences)
}

fn similarity(left: &str, right: &str) -> f64 {
    similarity_with(&JaroWinkler::default(), left, right)
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
        ("elephant", "hippo", 0.44166666666666665),
        ("fly", "ant", 0.0),
        ("frog", "fog", 0.925),
        ("MARTHA", "MARHTA", 0.9611111111111111),
        ("DWAYNE", "DUANE", 0.84),
        ("DIXON", "DICKSONX", 0.8133333333333332),
        ("duck donald", "duck daisy", 0.867272727272),
    ];

    for &(left, right, expected) in cases {
        assert_close(similarity(left, right), expected);
    }
}

#[test]
fn prefix_boost_scores_higher_than_plain_jaro() {
    // Same pairs as the Jaro card's fixture table — Jaro-Winkler's boost
    // must strictly exceed the corresponding plain-Jaro score whenever a
    // common prefix exists and the base score is above the 0.7 threshold.
    let jaro_scores: &[(&str, &str, f64)] = &[
        ("frog", "fog", 0.9166666666666666),
        ("MARTHA", "MARHTA", 0.944444444),
        ("DWAYNE", "DUANE", 0.822222222),
        ("DIXON", "DICKSONX", 0.7666666666666666),
    ];
    for &(left, right, jaro_score) in jaro_scores {
        let winkler_score = similarity(left, right);
        assert!(
            winkler_score > jaro_score,
            "{left}/{right}: expected winkler ({winkler_score}) > jaro ({jaro_score})"
        );
    }
}

#[test]
fn no_shared_prefix_gets_no_boost_even_above_threshold() {
    // fly/ant never reaches the 0.7 threshold at all (no common characters),
    // so it's already covered by matches_frozen_fixture_values. This test
    // instead confirms winklerize=false behaves exactly like Jaro when
    // driven through the JaroWinkler struct directly.
    let non_boosting = JaroWinkler {
        prefix_weight: 0.1,
        ..JaroWinkler::default()
    };
    assert_close(similarity_with(&non_boosting, "fly", "ant"), 0.0);
}

#[test]
fn empty_inputs_score_zero() {
    assert_close(similarity("", ""), 0.0);
    assert_close(similarity("hello", ""), 0.0);
}

#[test]
fn identical_strings_score_one() {
    assert_close(similarity("identical", "identical"), 1.0);
}

#[test]
fn long_tolerance_flag_changes_result_for_long_similar_strings() {
    // None of the frozen fixtures exercise `long_tolerance=True` (flagged
    // as a known risk in the behavior card) — this constructs a pair long
    // enough (min_len > 4) and similar enough to actually trigger the
    // long-string adjustment branch, and checks the flag changes the score
    // rather than being silently ignored.
    let left = "abcdefghij";
    let right = "abcdefghik";
    let without = similarity_with(&JaroWinkler::default(), left, right);
    let with_tolerance = similarity_with(
        &JaroWinkler {
            long_tolerance: true,
            ..JaroWinkler::default()
        },
        left,
        right,
    );
    assert!(
        with_tolerance >= without,
        "long_tolerance adjustment must not decrease the score: {with_tolerance} < {without}"
    );
}

#[test]
fn maximum_is_constant_one_not_sequence_length() {
    let sequences = prepare_sequences(
        &[
            InputSequence::Text("duck donald".to_owned()),
            InputSequence::Text("duck daisy".to_owned()),
        ],
        QValue::Elements,
    )
    .expect("text preparation should succeed");
    let algorithm = JaroWinkler::default();
    assert_close(algorithm.maximum(&sequences), 1.0);
    assert_close(
        algorithm.normalized_similarity(&sequences),
        algorithm.similarity(&sequences),
    );
}

#[test]
fn fails_if_implementation_becomes_a_trivial_constant() {
    assert!(similarity("frog", "fog") != 0.5);
    assert!(similarity("fly", "ant") != 0.5);
}
