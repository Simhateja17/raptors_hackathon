use textdistance_port::algorithms::phonetic::mra::MRA;
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

/// (left, right, similarity, distance, maximum) fixtures generated from the
/// original Python `textdistance.mra` implementation.
const FIXTURES: &[(&str, &str, f64, f64, f64)] = &[
    ("", "", 0.0, 0.0, 0.0),
    ("a", "", 0.0, 1.0, 1.0),
    ("", "a", 0.0, 1.0, 1.0),
    ("a", "a", 1.0, 0.0, 1.0),
    ("ab", "a", 1.0, 1.0, 2.0),
    ("abc", "abc", 3.0, 0.0, 3.0),
    ("abc", "abcde", 3.0, 1.0, 4.0),
    ("abcg", "abcdeg", 3.0, 2.0, 5.0),
    ("abcg", "abcdefg", 3.0, 3.0, 6.0),
    ("Tomato", "Tamato", 3.0, 0.0, 3.0),
    ("ato", "Tam", 0.0, 2.0, 2.0),
    ("spam", "qwer", 0.0, 3.0, 3.0),
    ("mra", "mra", 2.0, 0.0, 2.0),
    ("MARTHA", "MARHTA", 2.0, 2.0, 4.0),
    ("MARTHA", "MARTHA", 4.0, 0.0, 4.0),
    ("JON", "JAN", 2.0, 0.0, 2.0),
    ("BYRON", "BOYRON", 4.0, 0.0, 4.0),
    ("BYRON", "BYRON", 4.0, 0.0, 4.0),
    ("SMITH", "SMYTH", 2.0, 3.0, 5.0),
    ("CATHERINE", "KATHRYN", 3.0, 3.0, 6.0),
    ("BRIAN", "BRYAN", 2.0, 2.0, 4.0),
    ("PETER", "PIETER", 3.0, 0.0, 3.0),
    ("aeiou", "aeiou", 1.0, 0.0, 1.0),
    ("AEIOUY", "Y", 0.0, 2.0, 2.0),
    ("hello world", "hello world", 6.0, 0.0, 6.0),
    ("aaa", "aaaa", 1.0, 0.0, 1.0),
    ("abcdefgh", "abcdefgh", 6.0, 0.0, 6.0),
    ("test", "text", 2.0, 1.0, 3.0),
    ("kitten", "sitting", 2.0, 2.0, 4.0),
    ("mississippi", "ississippi", 2.0, 1.0, 3.0),
    ("a", "b", 0.0, 1.0, 1.0),
    ("cafe", "caffe", 2.0, 0.0, 2.0),
    ("straße", "STRASSE", 4.0, 0.0, 4.0),
];

#[test]
fn matches_original_mra_fixtures() {
    let algorithm = MRA::new();
    for &(left, right, similarity, distance, maximum) in FIXTURES {
        let pair = text_pair(left, right);
        assert_eq!(algorithm.maximum(&pair), maximum, "maximum({left:?}, {right:?})");
        assert_eq!(
            algorithm.similarity(&pair),
            similarity,
            "similarity({left:?}, {right:?})"
        );
        assert_eq!(
            algorithm.distance(&pair),
            distance,
            "distance({left:?}, {right:?})"
        );
    }
}

#[test]
fn no_common_chars_gives_zero_similarity() {
    let algorithm = MRA::new();
    let pair = text_pair("spam", "qwer");
    assert_eq!(algorithm.similarity(&pair), 0.0);
}

#[test]
fn empty_inputs_have_zero_distance() {
    let algorithm = MRA::new();
    let pair = text_pair("", "");
    assert_eq!(algorithm.distance(&pair), 0.0);
}

#[test]
fn unequal_length_inputs_have_positive_distance_when_maximum_is_nonzero() {
    let algorithm = MRA::new();
    let pair = text_pair("", "qwertyui");
    assert!(algorithm.maximum(&pair) > 0.0);
    assert!(algorithm.distance(&pair) > 0.0);
}

#[test]
fn normalization_matches_source_relationship() {
    let algorithm = MRA::new();
    for &(left, right, ..) in FIXTURES {
        let pair = text_pair(left, right);
        let nd = algorithm.normalized_distance(&pair);
        let ns = algorithm.normalized_similarity(&pair);
        assert!((0.0..=1.0).contains(&nd), "{left:?}/{right:?} nd={nd}");
        assert!((0.0..=1.0).contains(&ns), "{left:?}/{right:?} ns={ns}");
        assert!((nd + ns - 1.0).abs() < 1e-9, "{left:?}/{right:?}");
    }
}

#[test]
fn identical_text_is_fully_similar() {
    let algorithm = MRA::new();
    let pair = text_pair("hello world", "hello world");
    assert_eq!(algorithm.normalized_distance(&pair), 0.0);
    assert_eq!(algorithm.normalized_similarity(&pair), 1.0);
}

#[test]
fn length_over_six_keeps_only_first_three_and_last_three() {
    // "CATHERINE"/"KATHRYN" only agree once truncated to codex form; this
    // exercises the `len(word) > 6` branch of `_calc_mra`.
    let algorithm = MRA::new();
    let pair = text_pair("CATHERINE", "KATHRYN");
    assert_eq!(algorithm.maximum(&pair), 6.0);
    assert_eq!(algorithm.similarity(&pair), 3.0);
}
