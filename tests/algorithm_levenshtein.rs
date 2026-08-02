use textdistance_port::algorithms::edit::levenshtein::Levenshtein;
use textdistance_port::{prepare_sequences, Algorithm, InputSequence, PreparedSequence, QValue};

fn text_pair(left: &str, right: &str, qvalue: QValue) -> Vec<PreparedSequence> {
    prepare_sequences(
        &[
            InputSequence::Text(left.to_owned()),
            InputSequence::Text(right.to_owned()),
        ],
        qvalue,
    )
    .expect("text preparation should succeed")
}

#[test]
fn matches_the_original_distance_fixtures() {
    let algorithm = Levenshtein::new();
    for (left, right, expected) in [
        ("test", "text", 1.0),
        ("test", "tset", 2.0),
        ("test", "qwe", 4.0),
        ("test", "testit", 2.0),
        ("test", "tesst", 1.0),
        ("test", "tet", 1.0),
    ] {
        let prepared = text_pair(left, right, QValue::Elements);
        assert_eq!(
            algorithm.distance(&prepared),
            expected,
            "{left:?}/{right:?}"
        );
    }
}

#[test]
fn handles_empty_unicode_integer_and_qgram_inputs() {
    let algorithm = Levenshtein::new();

    let empty = text_pair("", "abc", QValue::Elements);
    assert_eq!(algorithm.distance(&empty), 3.0);

    let unicode = text_pair("café", "cafe", QValue::Elements);
    assert_eq!(algorithm.distance(&unicode), 1.0);

    let grams = text_pair("abcd", "abce", QValue::NGrams(2));
    assert_eq!(algorithm.distance(&grams), 1.0);

    let integers = prepare_sequences(
        &[
            InputSequence::Integers(vec![1, 2, 3]),
            InputSequence::Integers(vec![1, 4, 3]),
        ],
        QValue::Elements,
    )
    .expect("integer preparation should succeed");
    assert_eq!(algorithm.distance(&integers), 1.0);
}

#[test]
fn normalized_methods_use_the_shared_maximum_contract() {
    let algorithm = Levenshtein::new();
    let prepared = text_pair("test", "qwe", QValue::Elements);

    assert_eq!(algorithm.maximum(&prepared), 4.0);
    assert_eq!(algorithm.normalized_distance(&prepared), 1.0);
    assert_eq!(algorithm.normalized_similarity(&prepared), 0.0);
}
