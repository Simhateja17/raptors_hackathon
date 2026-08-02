use textdistance_port::algorithms::edit::mlipns::MLIPNS;
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
fn matches_original_mlipns_fixtures() {
    let algorithm = MLIPNS::new();
    for (left, right, expected) in [
        ("", "", 1.0),
        ("a", "", 0.0),
        ("", "a", 0.0),
        ("a", "a", 1.0),
        ("ab", "a", 1.0),
        ("abc", "abc", 1.0),
        ("abc", "abcde", 1.0),
        ("abcg", "abcdeg", 1.0),
        ("abcg", "abcdefg", 0.0),
        ("Tomato", "Tamato", 1.0),
        ("ato", "Tam", 1.0),
    ] {
        assert_eq!(
            algorithm.similarity(&text_pair(left, right, QValue::Elements)),
            expected,
            "{left:?}/{right:?}"
        );
    }
}

#[test]
fn preserves_binary_normalization_unicode_qgram_and_integer_behavior() {
    let algorithm = MLIPNS::new();

    let unicode = text_pair("café", "cafe", QValue::Elements);
    assert_eq!(algorithm.similarity(&unicode), 1.0);

    let grams = text_pair("abcd", "abce", QValue::NGrams(2));
    assert_eq!(algorithm.similarity(&grams), 1.0);

    let different = text_pair("abcg", "abcdefg", QValue::Elements);
    assert_eq!(algorithm.maximum(&different), 1.0);
    assert_eq!(algorithm.distance(&different), 1.0);
    assert_eq!(algorithm.normalized_distance(&different), 1.0);
    assert_eq!(algorithm.normalized_similarity(&different), 0.0);

    let integers = prepare_sequences(
        &[
            InputSequence::Integers(vec![1, 2, 3]),
            InputSequence::Integers(vec![1, 2, 3]),
        ],
        QValue::Elements,
    )
    .expect("integer preparation should succeed");
    assert_eq!(algorithm.similarity(&integers), 1.0);
}

#[test]
fn exposes_source_threshold_configuration() {
    let algorithm = MLIPNS::with_params(0.5, 4);
    assert_eq!(algorithm.threshold(), 0.5);
    assert_eq!(algorithm.maxmismatches(), 4);
}
