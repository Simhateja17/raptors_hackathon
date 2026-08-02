use textdistance_port::algorithms::edit::damerau_levenshtein::DamerauLevenshtein;
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

fn distance(left: &str, right: &str, restricted: bool) -> f64 {
    let algorithm = DamerauLevenshtein::with_restricted(restricted);
    let prepared = text_pair(left, right, QValue::Elements);
    algorithm.distance(&prepared)
}

#[test]
fn matches_restricted_original_fixtures() {
    for (left, right, expected) in [
        ("test", "text", 1.0),
        ("test", "tset", 1.0),
        ("test", "qwy", 4.0),
        ("test", "testit", 2.0),
        ("test", "tesst", 1.0),
        ("test", "tet", 1.0),
        ("cat", "hat", 1.0),
        ("Niall", "Neil", 3.0),
        ("aluminum", "Catalan", 7.0),
        ("ATCG", "TAGC", 2.0),
        ("ab", "ba", 1.0),
        ("ab", "cde", 3.0),
        ("ab", "ac", 1.0),
        ("ab", "bc", 2.0),
        ("ab", "bca", 3.0),
        ("abcd", "bdac", 4.0),
    ] {
        assert_eq!(distance(left, right, true), expected, "{left:?}/{right:?}");
    }
}

#[test]
fn matches_unrestricted_transposition_fixtures() {
    for (left, right, expected) in [
        ("test", "text", 1.0),
        ("test", "tset", 1.0),
        ("test", "qwy", 4.0),
        ("test", "testit", 2.0),
        ("test", "tesst", 1.0),
        ("test", "tet", 1.0),
        ("cat", "hat", 1.0),
        ("Niall", "Neil", 3.0),
        ("aluminum", "Catalan", 7.0),
        ("ATCG", "TAGC", 2.0),
        ("ab", "ba", 1.0),
        ("ab", "cde", 3.0),
        ("ab", "ac", 1.0),
        ("ab", "bc", 2.0),
        ("ab", "bca", 2.0),
        ("abcd", "bdac", 3.0),
    ] {
        assert_eq!(distance(left, right, false), expected, "{left:?}/{right:?}");
    }
}

#[test]
fn handles_empty_unicode_and_qgram_inputs() {
    let restricted = DamerauLevenshtein::new();

    let empty = text_pair("", "abc", QValue::Elements);
    assert_eq!(restricted.distance(&empty), 3.0);

    let equal = text_pair("same", "same", QValue::Elements);
    assert_eq!(restricted.distance(&equal), 0.0);

    let unicode = text_pair("café", "cafe", QValue::Elements);
    assert_eq!(restricted.distance(&unicode), 1.0);

    let grams = text_pair("abcd", "abdc", QValue::NGrams(2));
    assert_eq!(restricted.distance(&grams), 2.0);
    assert!(restricted.is_restricted());
    assert!(!DamerauLevenshtein::with_restricted(false).is_restricted());
}
