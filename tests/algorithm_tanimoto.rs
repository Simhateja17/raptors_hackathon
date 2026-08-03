use textdistance_port::algorithms::token::tanimoto::Tanimoto;
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

// Tanimoto has no dedicated file under `tests/original/test_token`; it is
// `log2` of the Jaccard similarity (see `textdistance/algorithms/token_based.py`,
// class `Tanimoto(Jaccard)`). Reference values below were captured by running
// `textdistance.Tanimoto(external=False)` / `textdistance.Tanimoto(external=True)`
// directly against the (left, right, expected) triples from
// `tests/original/test_token/test_jaccard.py::test_distance` with `log2`
// applied, plus additional runs of `textdistance.Tanimoto` / `textdistance.tanimoto`
// for behavior not covered by that file (q-grams, words, unicode, multi-sequence,
// set mode, empty/equal quick answers, and the common Base/BaseSimilarity methods).

#[test]
fn tanimoto_matches_log2_of_original_jaccard_test_distance_cases() {
    let algorithm = Tanimoto::default();
    close(
        algorithm.call(&prepared("test", "text", QValue::Elements)),
        (3.0f64 / 5.0).log2(),
    );
    close(
        algorithm.call(&prepared("nelson", "neilsen", QValue::Elements)),
        (5.0f64 / 8.0).log2(),
    );
    close(
        algorithm.call(&prepared("decide", "resize", QValue::Elements)),
        (3.0f64 / 9.0).log2(),
    );
}

#[test]
fn tanimoto_returns_negative_infinity_for_disjoint_sequences() {
    let algorithm = Tanimoto::default();
    let sequences = prepared("", "abc", QValue::Elements);
    let score = algorithm.call(&sequences);
    assert!(score.is_infinite() && score.is_sign_negative());

    let disjoint = prepared("abc", "xyz", QValue::Elements);
    let score = algorithm.call(&disjoint);
    assert!(score.is_infinite() && score.is_sign_negative());
}

#[test]
fn tanimoto_preserves_empty_and_equal_quick_answers() {
    let algorithm = Tanimoto::default();
    close(algorithm.call(&prepared("", "", QValue::Elements)), 0.0);
    close(algorithm.call(&prepared("test", "test", QValue::Elements)), 0.0);

    let unicode = prepared("café", "café", QValue::Elements);
    close(algorithm.call(&unicode), 0.0);
}

#[test]
fn tanimoto_matches_source_examples_and_common_methods() {
    let algorithm = Tanimoto::default();
    let sequences = prepared("test", "text", QValue::Elements);
    let expected = (3.0f64 / 5.0).log2();

    close(algorithm.call(&sequences), expected);
    close(algorithm.similarity(&sequences), expected);
    close(algorithm.distance(&sequences), 1.0 - expected);
    close(algorithm.normalized_distance(&sequences), 1.0 - expected);
    close(algorithm.normalized_similarity(&sequences), expected);
    assert_eq!(algorithm.maximum(&sequences), 1.0);
}

#[test]
fn tanimoto_supports_qgrams_words_unicode_and_three_sequences() {
    let algorithm = Tanimoto::default();
    close(
        algorithm.call(&prepared("test", "text", QValue::NGrams(2))),
        (1.0f64 / 5.0).log2(),
    );
    close(
        algorithm.call(&prepared("one two", "one three", QValue::Words)),
        (1.0f64 / 3.0).log2(),
    );

    let three = prepare_sequences(
        &[
            InputSequence::Text("abc".into()),
            InputSequence::Text("abd".into()),
            InputSequence::Text("abe".into()),
        ],
        QValue::Elements,
    )
    .unwrap();
    close(algorithm.call(&three), (2.0f64 / 5.0).log2());
}

#[test]
fn tanimoto_multiset_vs_as_set_mode() {
    let multiset = Tanimoto::default();
    let set = Tanimoto::new(QValue::Elements, true, true);
    let sequences = prepared("aaaa", "aa", QValue::Elements);

    close(multiset.call(&sequences), (1.0f64 / 2.0).log2());
    close(set.call(&sequences), 0.0);
}
