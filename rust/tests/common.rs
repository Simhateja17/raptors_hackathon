use textdistance_port::{
    all_identical, maximum_length, normalize_distance, normalize_similarity, prepare_sequences,
    Algorithm, Element, InputSequence, QValue, ScoreMode,
};

struct DistanceOne;

impl Algorithm for DistanceOne {
    fn raw_score(&self, _sequences: &[Vec<Element>]) -> f64 {
        1.0
    }
}

struct SimilarityOne;

impl Algorithm for SimilarityOne {
    fn raw_score(&self, _sequences: &[Vec<Element>]) -> f64 {
        1.0
    }

    fn score_mode(&self) -> ScoreMode {
        ScoreMode::Similarity
    }
}

#[test]
fn text_is_prepared_as_unicode_scalars() {
    let prepared = prepare_sequences(&[InputSequence::Text("café".to_owned())], QValue::Elements)
        .expect("text preparation should succeed");

    assert_eq!(
        prepared[0],
        vec![
            Element::Char('c'),
            Element::Char('a'),
            Element::Char('f'),
            Element::Char('é'),
        ]
    );
    assert_eq!(maximum_length(&prepared), 4);
}

#[test]
fn words_and_ngrams_follow_the_contract() {
    let text = InputSequence::Text("one  two".to_owned());
    let words = prepare_sequences(&[text.clone()], QValue::Words).unwrap();
    assert_eq!(
        words[0],
        vec![Element::Text("one".into()), Element::Text("two".into())]
    );

    let grams = prepare_sequences(&[text], QValue::NGrams(2)).unwrap();
    assert_eq!(grams[0].len(), 7);
    assert_eq!(
        grams[0][0],
        Element::Gram(vec![Element::Char('o'), Element::Char('n')])
    );
}

#[test]
fn identity_and_normalization_helpers_are_stable() {
    let sequence = vec![Element::Integer(1), Element::Integer(2)];
    assert!(all_identical(&[sequence.clone(), sequence.clone()]));
    assert!(!all_identical(&[
        sequence.clone(),
        vec![Element::Integer(3)]
    ]));
    assert_eq!(normalize_distance(2.0, 4.0), 0.5);
    assert_eq!(normalize_similarity(2.0, 4.0), 0.5);
}

#[test]
fn common_algorithm_methods_convert_raw_scores() {
    let sequences = prepare_sequences(
        &[
            InputSequence::Text("a".to_owned()),
            InputSequence::Text("b".to_owned()),
        ],
        QValue::Elements,
    )
    .unwrap();

    let distance = DistanceOne;
    assert_eq!(distance.call(&sequences), 1.0);
    assert_eq!(distance.distance(&sequences), 1.0);
    assert_eq!(distance.similarity(&sequences), 0.0);

    let similarity = SimilarityOne;
    assert_eq!(similarity.score_mode(), ScoreMode::Similarity);
    assert_eq!(similarity.similarity(&sequences), 1.0);
    assert_eq!(similarity.distance(&sequences), 0.0);
}
