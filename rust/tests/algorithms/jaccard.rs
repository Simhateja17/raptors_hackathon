use textdistance_port::algorithms::token::jaccard::Jaccard;
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

#[test]
fn jaccard_matches_source_examples_and_common_methods() {
    let algorithm = Jaccard::default();
    let sequences = prepared("test", "text", QValue::Elements);
    close(algorithm.call(&sequences), 3.0 / 5.0);
    close(algorithm.similarity(&sequences), 3.0 / 5.0);
    close(algorithm.distance(&sequences), 2.0 / 5.0);
    close(algorithm.normalized_distance(&sequences), 2.0 / 5.0);
    assert_eq!(algorithm.maximum(&sequences), 1.0);
}

#[test]
fn jaccard_preserves_repeated_tokens_and_set_mode() {
    let multiset = Jaccard::default();
    let set = Jaccard::new(QValue::Elements, true, true);
    let sequences = prepared("aaaa", "aa", QValue::Elements);
    close(multiset.call(&sequences), 0.5);
    close(set.call(&sequences), 1.0);
}

#[test]
fn jaccard_supports_qgrams_unicode_and_three_sequences() {
    let algorithm = Jaccard::default();
    close(
        algorithm.call(&prepared("test", "text", QValue::NGrams(2))),
        0.2,
    );
    close(
        algorithm.call(&prepared("one two", "one three", QValue::Words)),
        1.0 / 3.0,
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
    close(algorithm.call(&three), 2.0 / 5.0);
}

#[test]
fn jaccard_preserves_empty_and_equal_quick_answers() {
    let algorithm = Jaccard::default();
    close(algorithm.call(&prepared("", "", QValue::Elements)), 1.0);
    close(algorithm.call(&prepared("", "abc", QValue::Elements)), 0.0);
}
