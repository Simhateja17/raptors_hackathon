use textdistance_port::algorithms::edit::strcmp95::StrCmp95;
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

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "actual={actual}, expected={expected}"
    );
}

#[test]
fn matches_original_strcmp95_fixtures() {
    let algorithm = StrCmp95::new();
    for (left, right, expected) in [
        ("MARTHA", "MARHTA", 0.9611111111111111),
        ("DWAYNE", "DUANE", 0.873),
        ("DIXON", "DICKSONX", 0.839333333),
        ("TEST", "TEXT", 0.9066666666666666),
    ] {
        assert_close(algorithm.similarity(&text_pair(left, right)), expected);
    }
}

#[test]
fn preserves_normalization_and_text_preprocessing() {
    let algorithm = StrCmp95::new();

    assert_eq!(algorithm.similarity(&text_pair("", "")), 1.0);
    assert_eq!(algorithm.similarity(&text_pair("a", "")), 0.0);
    assert_eq!(algorithm.similarity(&text_pair("  martha ", "MARTHA")), 1.0);
    assert_eq!(algorithm.maximum(&text_pair("a", "b")), 1.0);

    let different = text_pair("TEST", "QWE");
    let score = algorithm.similarity(&different);
    assert_close(algorithm.distance(&different), 1.0 - score);
    assert_close(algorithm.normalized_distance(&different), 1.0 - score);
    assert_close(algorithm.normalized_similarity(&different), score);
}

#[test]
fn long_string_mode_is_explicit_and_unicode_is_not_byte_indexed() {
    let short = StrCmp95::new();
    let long = StrCmp95::with_long_strings(true);
    assert!(!short.long_strings());
    assert!(long.long_strings());

    let unicode = text_pair("café", "cafe");
    assert!(short.similarity(&unicode) < 1.0);
}
