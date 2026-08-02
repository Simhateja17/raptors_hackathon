use textdistance_port::algorithms::compression::rle_ncd::{RleError, RleNcd};
use textdistance_port::{prepare_sequences, Algorithm, Element, InputSequence, QValue};

fn sequence(value: &str) -> Vec<Element> {
    prepare_sequences(
        &[InputSequence::Text(value.to_owned())],
        QValue::Elements,
    )
    .unwrap()
    .pop()
    .unwrap()
}

fn pair(left: &str, right: &str) -> Vec<Vec<Element>> {
    prepare_sequences(
        &[
            InputSequence::Text(left.to_owned()),
            InputSequence::Text(right.to_owned()),
        ],
        QValue::Elements,
    )
    .unwrap()
}

#[test]
fn rle_encoding_matches_python_run_rules() {
    let algorithm = RleNcd::default();
    for (input, expected) in [
        ("", ""),
        ("A", "A"),
        ("AA", "AA"),
        ("AAA", "3A"),
        ("ABBB", "A3B"),
        ("AAAAA", "5A"),
        ("AABBAAA", "AABB3A"),
    ] {
        assert_eq!(algorithm.compress(&sequence(input)).unwrap(), expected);
    }
}

#[test]
fn rle_ncd_matches_source_edge_values() {
    let algorithm = RleNcd::default();
    assert_eq!(algorithm.call(&[]), 0.0);
    assert_eq!(algorithm.call(&pair("", "")), 0.0);
    assert_eq!(algorithm.call(&pair("A", "A")), 1.0);
    assert_eq!(algorithm.call(&pair("AA", "AA")), 0.0);
    assert_eq!(algorithm.call(&pair("AAA", "AAA")), 0.0);
    assert_eq!(algorithm.call(&pair("ABBB", "ABBB")), 1.0);
    assert_eq!(algorithm.call(&pair("test", "test")), 1.0);
    assert_eq!(algorithm.call(&pair("test", "text")), 1.0);
    let words = prepare_sequences(
        &[
            InputSequence::Text("one two".into()),
            InputSequence::Text("one three".into()),
        ],
        QValue::Words,
    )
    .unwrap();
    assert_eq!(algorithm.call(&words), 1.0);
}

#[test]
fn rle_ncd_preserves_unicode_and_rejects_non_string_elements() {
    let algorithm = RleNcd::default();
    assert_eq!(algorithm.compress(&sequence("😀😀😀")).unwrap(), "3😀");

    let qgrams = prepare_sequences(
        &[
            InputSequence::Text("test".into()),
            InputSequence::Text("text".into()),
        ],
        QValue::NGrams(2),
    )
    .unwrap();
    assert_eq!(
        algorithm.try_raw_score(&qgrams),
        Err(RleError::UnsupportedElement("q-gram"))
    );

    let integers = vec![Element::Integer(1), Element::Integer(2)];
    assert_eq!(
        algorithm.try_raw_score(&[integers.clone(), integers]),
        Err(RleError::UnsupportedElement("integer"))
    );
}
