use textdistance_port::algorithms::edit::needleman_wunsch::{EqualityScorer, MatrixScorer};
use textdistance_port::algorithms::edit::smith_waterman::SmithWaterman;
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
fn matches_identity_alignment_fixtures() {
    let algorithm = SmithWaterman::with_scorer(1.0, EqualityScorer::new(1.0, -1.0));
    let prepared = text_pair("GATTACA", "GCATGCU", QValue::Elements);
    assert_eq!(algorithm.similarity(&prepared), 0.0);
}

#[test]
fn matches_gap_five_and_matrix_fixtures() {
    let gap_five = SmithWaterman::with_scorer(5.0, EqualityScorer::new(1.0, -1.0));
    for (left, right, expected) in [
        ("CGATATCAG", "TGACGSTGC", 0.0),
        ("AGACTAGTTAC", "TGACGSTGC", 1.0),
        ("AGACTAGTTAC", "CGAGACGT", 0.0),
    ] {
        let prepared = text_pair(left, right, QValue::Elements);
        assert_eq!(
            gap_five.similarity(&prepared),
            expected,
            "{left:?}/{right:?}"
        );
    }

    let matrix = MatrixScorer::from_char_scores(
        [
            (('A', 'A'), 10.0),
            (('G', 'G'), 7.0),
            (('C', 'C'), 9.0),
            (('T', 'T'), 8.0),
            (('A', 'G'), -1.0),
            (('A', 'C'), -3.0),
            (('A', 'T'), -4.0),
            (('G', 'C'), -5.0),
            (('G', 'T'), -3.0),
            (('C', 'T'), 0.0),
        ],
        0.0,
        1.0,
        true,
    );
    let matrix_algorithm = SmithWaterman::with_scorer(5.0, matrix);
    let prepared = text_pair("AGACTAGTTAC", "CGAGACGT", QValue::Elements);
    assert_eq!(matrix_algorithm.similarity(&prepared), 26.0);
}

#[test]
fn preserves_local_empty_equal_unicode_qgram_and_normalized_behavior() {
    let algorithm = SmithWaterman::new();

    let empty = text_pair("", "abc", QValue::Elements);
    assert_eq!(algorithm.similarity(&empty), 0.0);

    let equal = text_pair("same", "same", QValue::Elements);
    assert_eq!(algorithm.similarity(&equal), 4.0);

    let unicode = text_pair("café", "cafe", QValue::Elements);
    assert_eq!(algorithm.similarity(&unicode), 3.0);

    let grams = text_pair("abcd", "abce", QValue::NGrams(2));
    assert_eq!(algorithm.similarity(&grams), 2.0);

    let different = text_pair("test", "qwe", QValue::Elements);
    assert_eq!(algorithm.maximum(&different), 3.0);
    assert_eq!(algorithm.normalized_distance(&different), 1.0);
    assert_eq!(algorithm.normalized_similarity(&different), 0.0);
}
