use textdistance_port::algorithms::edit::gotoh::Gotoh;
use textdistance_port::algorithms::edit::needleman_wunsch::EqualityScorer;
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
fn matches_gap_one_and_extension_one_fixture() {
    let algorithm = Gotoh::with_scorer(1.0, 1.0, EqualityScorer::new(1.0, -1.0));
    let prepared = text_pair("GATTACA", "GCATGCU", QValue::Elements);
    assert_eq!(algorithm.similarity(&prepared), 0.0);
}

#[test]
fn matches_affine_gap_fixtures() {
    let gap_half = Gotoh::with_scorer(1.0, 0.5, EqualityScorer::new(1.0, -1.0));
    for (left, right, expected) in [
        ("GATTACA", "GCATGCU", 0.0),
        ("AGACTAGTTAC", "TGACGSTGC", 1.5),
        ("AGACTAGTTAC", "CGAGACGT", 1.0),
    ] {
        let prepared = text_pair(left, right, QValue::Elements);
        assert_eq!(
            gap_half.similarity(&prepared),
            expected,
            "{left:?}/{right:?}"
        );
    }

    let gap_five = Gotoh::with_scorer(5.0, 5.0, EqualityScorer::new(1.0, -1.0));
    let prepared = text_pair("AGACTAGTTAC", "CGAGACGT", QValue::Elements);
    assert_eq!(gap_five.similarity(&prepared), -15.0);
}

#[test]
fn preserves_affine_empty_equal_unicode_qgram_and_normalized_behavior() {
    let algorithm = Gotoh::new();

    let empty = text_pair("", "abc", QValue::Elements);
    assert_eq!(algorithm.similarity(&empty), -1.8);

    let equal = text_pair("same", "same", QValue::Elements);
    assert_eq!(algorithm.similarity(&equal), 4.0);

    let unicode = text_pair("café", "cafe", QValue::Elements);
    assert_eq!(algorithm.similarity(&unicode), 3.0);

    let grams = text_pair("abcd", "abce", QValue::NGrams(2));
    assert_eq!(algorithm.similarity(&grams), 2.0);

    let different = text_pair("test", "qwe", QValue::Elements);
    assert_eq!(algorithm.maximum(&different), 3.0);
    assert_eq!(algorithm.normalized_distance(&different), 7.0 / 6.0);
    assert_eq!(algorithm.normalized_similarity(&different), 1.0 / 3.0);
}
