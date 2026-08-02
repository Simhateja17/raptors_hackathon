use textdistance_port::algorithms::sequence::ratcliff_obershelp::RatcliffObershelp;
use textdistance_port::{prepare_sequences, Algorithm, InputSequence, PreparedSequence, QValue};

fn text_pair(left: &str, right: &str) -> Vec<PreparedSequence> {
    prepare_sequences(
        &[
            InputSequence::Text(left.to_owned()),
            InputSequence::Text(right.to_owned()),
        ],
        QValue::Elements,
    )
    .expect("text preparation should succeed")
}

fn text_many(values: &[&str]) -> Vec<PreparedSequence> {
    let inputs: Vec<InputSequence> = values
        .iter()
        .map(|value| InputSequence::Text((*value).to_owned()))
        .collect();
    prepare_sequences(&inputs, QValue::Elements).expect("text preparation should succeed")
}

// Expected values below were captured from the original Python
// `textdistance.ratcliff_obershelp` (a `RatcliffObershelp()` instance) so the
// Rust port's numeric output matches exactly.
#[test]
fn matches_original_two_sequence_fixtures() {
    let algorithm = RatcliffObershelp::new();
    for (left, right, expected) in [
        ("", "", 1.0),
        ("a", "", 0.0),
        ("", "a", 0.0),
        ("a", "a", 1.0),
        ("abcd", "abcd", 1.0),
        ("ab", "cd", 0.0),
        ("spam", "qwer", 0.0),
        ("test", "text", 0.75),
        ("gestalt pattern matching", "gestalt practice", 0.6),
        ("DIXON", "DICKSONX", 0.6153846153846154),
        ("thisisatest", "testing123testing", 0.2857142857142857),
    ] {
        assert_eq!(
            algorithm.similarity(&text_pair(left, right)),
            expected,
            "{left:?}/{right:?}"
        );
    }
}

#[test]
fn matches_original_multi_sequence_fixtures() {
    let algorithm = RatcliffObershelp::new();

    let prepared = text_many(&["abc", "abc", "abc"]);
    assert_eq!(algorithm.similarity(&prepared), 1.0);

    let prepared = text_many(&["night", "nacht", "nought"]);
    assert_eq!(algorithm.similarity(&prepared), 0.5625);
}

#[test]
fn breaks_longest_common_substring_ties_like_difflib() {
    // Two candidate common substrings of equal length ("AAAA" and "BBBB").
    // difflib's SequenceMatcher (used by the source library for two
    // sequences under 200 elements) picks the one starting earliest in the
    // first sequence, i.e. "AAAA", not the one starting earliest in the
    // shorter sequence ("BBBB"). Confirmed against the real Python output.
    let algorithm = RatcliffObershelp::new();
    let prepared = text_pair("AAAAxxxxxxxxxxxxxxBBBB", "BBBBAAAA");
    assert_eq!(algorithm.similarity(&prepared), 0.26666666666666666);
}

#[test]
fn preserves_unicode_maximum_and_normalization_behavior() {
    let algorithm = RatcliffObershelp::new();

    let unicode = text_pair("café", "cafe");
    assert_eq!(algorithm.similarity(&unicode), 0.75);

    let different = text_pair("spam", "qwer");
    assert_eq!(algorithm.maximum(&different), 1.0);
    assert_eq!(algorithm.distance(&different), 1.0);
    assert_eq!(algorithm.normalized_distance(&different), 1.0);
    assert_eq!(algorithm.normalized_similarity(&different), 0.0);

    let identical = text_pair("test", "test");
    assert_eq!(algorithm.maximum(&identical), 1.0);
    assert_eq!(algorithm.distance(&identical), 0.0);
    assert_eq!(algorithm.normalized_distance(&identical), 0.0);
    assert_eq!(algorithm.normalized_similarity(&identical), 1.0);
}

#[test]
fn empty_pair_has_zero_distance() {
    // Ported from the original test_common.py::test_empty, parametrized over
    // all algorithms including ratcliff_obershelp.
    let algorithm = RatcliffObershelp::new();
    assert_eq!(algorithm.distance(&text_pair("", "")), 0.0);
}

#[test]
fn no_common_chars_has_zero_similarity() {
    // Ported from the original test_common.py::test_no_common_chars.
    let algorithm = RatcliffObershelp::new();
    assert_eq!(algorithm.similarity(&text_pair("spam", "qwer")), 0.0);
}

#[test]
fn unequal_distance_is_positive() {
    // Ported from the original test_common.py::test_unequal_distance.
    let algorithm = RatcliffObershelp::new();
    let prepared = text_pair("", "qwertyui");
    if algorithm.maximum(&prepared) != 0.0 {
        assert!(algorithm.distance(&prepared) > 0.0);
    }
}
