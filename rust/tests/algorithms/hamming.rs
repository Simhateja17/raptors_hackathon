use textdistance_port::algorithms::edit::hamming::Hamming;
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

#[test]
fn hamming_matches_basic_and_unicode_cases() {
    let algorithm = Hamming::default();
    assert_eq!(algorithm.call(&prepared("test", "text", QValue::Elements)), 1.0);
    assert_eq!(algorithm.call(&prepared("é", "e", QValue::Elements)), 1.0);
    assert_eq!(algorithm.call(&prepared("😀", "😀", QValue::Elements)), 0.0);
}

#[test]
fn hamming_preserves_truncate_and_empty_quick_answer() {
    let full = Hamming::default();
    let truncated = Hamming::new(QValue::Elements, true, true);
    let sequences = prepared("test", "testit", QValue::Elements);
    assert_eq!(full.call(&sequences), 2.0);
    assert_eq!(truncated.call(&sequences), 0.0);

    let empty = prepared("", "abc", QValue::Elements);
    assert_eq!(full.call(&empty), 3.0);
    assert_eq!(truncated.call(&empty), 3.0);
}

#[test]
fn hamming_supports_qgrams_and_custom_comparators() {
    let algorithm = Hamming::default();
    assert_eq!(
        algorithm.call(&prepared("test", "text", QValue::NGrams(2))),
        2.0
    );
    let words = prepared("one two", "one three", QValue::Words);
    assert_eq!(algorithm.call(&words), 1.0);

    let always_equal = Hamming::with_test_func(
        QValue::Elements,
        false,
        true,
        |_values| true,
    );
    assert_eq!(always_equal.call(&prepared("a", "b", QValue::Elements)), 0.0);

    let detects_python_fill = Hamming::with_test_func(
        QValue::Elements,
        false,
        true,
        |values| values.iter().any(Option::is_none),
    );
    assert_eq!(
        detects_python_fill.call(&prepared("a", "ab", QValue::Elements)),
        1.0
    );
}
