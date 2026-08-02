use textdistance_port::algorithms::token::overlap::Overlap;
use textdistance_port::{prepare_sequences, Algorithm, InputSequence, QValue};

fn prepared(left: &str, right: &str, qvalue: QValue) -> Vec<Vec<textdistance_port::Element>> {
    prepare_sequences(
        &[
            InputSequence::Text(left.to_owned()),
            InputSequence::Text(right.to_owned()),
        ],
        qvalue,
    )
    .unwrap()
}

fn close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-12, "{actual} != {expected}");
}

// Reference values below were captured from the original Python source:
// `tests/original/test_token/test_overlap.py::test_distance` parametrizes
// exactly the three (left, right, expected) cases reproduced in
// `overlap_matches_original_test_distance_cases`, run against
// `textdistance.Overlap(external=False)` / `textdistance.Overlap(external=True)`.
// The remaining tests were captured by running
// `textdistance.overlap` / `textdistance.Overlap` directly for behavior not
// covered by that file (q-grams, words, unicode, multi-sequence, set mode,
// empty/equal quick answers).

#[test]
fn overlap_matches_original_test_distance_cases() {
    let algorithm = Overlap::default();
    close(
        algorithm.call(&prepared("test", "text", QValue::Elements)),
        3.0 / 4.0,
    );
    close(
        algorithm.call(&prepared("testme", "textthis", QValue::Elements)),
        4.0 / 6.0,
    );
    close(
        algorithm.call(&prepared("nelson", "neilsen", QValue::Elements)),
        5.0 / 6.0,
    );
}

#[test]
fn overlap_matches_source_examples_and_common_methods() {
    let algorithm = Overlap::default();
    let sequences = prepared("test", "text", QValue::Elements);
    close(algorithm.call(&sequences), 0.75);
    close(algorithm.similarity(&sequences), 0.75);
    close(algorithm.distance(&sequences), 0.25);
    close(algorithm.normalized_distance(&sequences), 0.25);
    assert_eq!(algorithm.maximum(&sequences), 1.0);
}

#[test]
fn overlap_is_asymmetric_from_jaccard_for_subsets() {
    // "ab" is fully contained in "abc": overlap coefficient is 1.0 even
    // though the sets are not equal (unlike Jaccard, which would be 2/3).
    let algorithm = Overlap::default();
    let sequences = prepared("ab", "abc", QValue::Elements);
    close(algorithm.call(&sequences), 1.0);
}

#[test]
fn overlap_preserves_repeated_tokens_and_set_mode() {
    let multiset = Overlap::default();
    let set = Overlap::new(QValue::Elements, true, true);
    let sequences = prepared("aaaa", "aa", QValue::Elements);
    close(multiset.call(&sequences), 1.0);
    close(set.call(&sequences), 1.0);
}

#[test]
fn overlap_supports_qgrams_words_unicode_and_three_sequences() {
    let algorithm = Overlap::default();
    close(
        algorithm.call(&prepared("test", "text", QValue::NGrams(2))),
        1.0 / 3.0,
    );
    close(
        algorithm.call(&prepared("one two", "one three", QValue::Words)),
        0.5,
    );
    let unicode = prepared("café", "café", QValue::Elements);
    close(algorithm.call(&unicode), 1.0);

    let three = prepare_sequences(
        &[
            InputSequence::Text("abc".into()),
            InputSequence::Text("abd".into()),
            InputSequence::Text("abe".into()),
        ],
        QValue::Elements,
    )
    .unwrap();
    close(algorithm.call(&three), 2.0 / 3.0);
}

#[test]
fn overlap_preserves_empty_and_equal_quick_answers() {
    let algorithm = Overlap::default();
    close(algorithm.call(&prepared("", "", QValue::Elements)), 1.0);
    close(algorithm.call(&prepared("", "abc", QValue::Elements)), 0.0);
}
