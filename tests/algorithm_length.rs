use textdistance_port::algorithms::simple::length::Length;
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
// `textdistance.length` (a `Length()` instance) so the Rust port's numeric
// output matches exactly. Note content never matters, only lengths.
#[test]
fn matches_original_length_fixtures() {
    let algorithm = Length::new();
    for (left, right, expected) in [
        ("", "", 0.0),
        ("a", "", 1.0),
        ("", "a", 1.0),
        ("abc", "xyz", 0.0),
        ("abc", "ab", 1.0),
        ("hello world", "hi", 9.0),
    ] {
        assert_eq!(
            algorithm.distance(&text_pair(left, right)),
            expected,
            "{left:?}/{right:?}"
        );
    }
}

#[test]
fn content_is_ignored_only_length_matters() {
    // "abc" vs "xyz" share no characters at all, yet distance is 0 because
    // both have length 3.
    let algorithm = Length::new();
    assert_eq!(algorithm.distance(&text_pair("abc", "xyz")), 0.0);
}

#[test]
fn matches_original_multi_sequence_fixtures() {
    let algorithm = Length::new();

    let prepared = text_many(&["a", "a", "a"]);
    assert_eq!(algorithm.distance(&prepared), 0.0);

    let prepared = text_many(&["ab", "abc", "abcd"]);
    assert_eq!(algorithm.distance(&prepared), 2.0);
}

#[test]
fn preserves_similarity_maximum_and_normalization_behavior() {
    let algorithm = Length::new();

    let equal_lengths = text_pair("abc", "xyz");
    assert_eq!(algorithm.maximum(&equal_lengths), 3.0);
    assert_eq!(algorithm.similarity(&equal_lengths), 3.0);
    assert_eq!(algorithm.normalized_distance(&equal_lengths), 0.0);
    assert_eq!(algorithm.normalized_similarity(&equal_lengths), 1.0);

    let unequal_lengths = text_pair("abc", "ab");
    assert_eq!(algorithm.maximum(&unequal_lengths), 3.0);
    assert_eq!(algorithm.similarity(&unequal_lengths), 2.0);
    assert_eq!(
        algorithm.normalized_distance(&unequal_lengths),
        1.0 / 3.0
    );
    assert_eq!(
        algorithm.normalized_similarity(&unequal_lengths),
        1.0 - 1.0 / 3.0
    );

    let both_empty = text_pair("", "");
    assert_eq!(algorithm.maximum(&both_empty), 0.0);
    assert_eq!(algorithm.normalized_distance(&both_empty), 0.0);
    assert_eq!(algorithm.normalized_similarity(&both_empty), 1.0);
}

#[test]
fn empty_pair_has_zero_distance() {
    // Ported from the original test_common.py::test_empty (Length is
    // commented out of that suite's ALGS tuple upstream, but the property
    // still holds).
    let algorithm = Length::new();
    assert_eq!(algorithm.distance(&text_pair("", "")), 0.0);
}

#[test]
fn unicode_and_qgram_lengths_are_counted_correctly() {
    let algorithm = Length::new();

    let unicode = text_pair("café", "cafe");
    assert_eq!(algorithm.distance(&unicode), 0.0);

    let grams = prepare_sequences(
        &[
            InputSequence::Text("abcd".to_owned()),
            InputSequence::Text("abcde".to_owned()),
        ],
        QValue::NGrams(2),
    )
    .expect("n-gram preparation should succeed");
    // "abcd" -> 3 bigrams, "abcde" -> 4 bigrams.
    assert_eq!(algorithm.distance(&grams), 1.0);
}
