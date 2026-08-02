use textdistance_port::algorithms::edit::damerau_levenshtein::DamerauLevenshtein;
use textdistance_port::algorithms::edit::jaro_winkler::JaroWinkler;
use textdistance_port::algorithms::token::monge_elkan::MongeElkan;
use textdistance_port::{prepare_sequences, Algorithm, Element, InputSequence, QValue};

fn tokens(values: &[&str]) -> InputSequence {
    InputSequence::Elements(
        values
            .iter()
            .map(|value| Element::Text((*value).to_owned()))
            .collect(),
    )
}

fn similarity(algorithm: &MongeElkan, left: &[&str], right: &[&str]) -> f64 {
    algorithm
        .try_similarity_inputs(&[tokens(left), tokens(right)])
        .expect("Monge-Elkan input preparation should succeed")
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn monge_elkan_matches_the_frozen_jaro_winkler_fixtures() {
    let algorithm = MongeElkan::from_python(JaroWinkler::default(), false, Some(1), true);

    assert_close(similarity(&algorithm, &["Niall"], &["Neal"]), 0.805);
    assert_close(
        similarity(&algorithm, &["Niall"], &["Nigel"]),
        0.7866666666666667,
    );
}

#[test]
fn monge_elkan_default_uses_damerau_levenshtein() {
    let algorithm = MongeElkan::default();

    assert_close(similarity(&algorithm, &["test"], &["text"]), 3.0);
}

#[test]
fn monge_elkan_preserves_symmetric_permutation_averaging() {
    let forward_algorithm = MongeElkan::from_python(JaroWinkler::default(), false, Some(1), true);
    let reverse_algorithm = MongeElkan::from_python(JaroWinkler::default(), false, Some(1), true);
    let symmetric_algorithm = MongeElkan::from_python(JaroWinkler::default(), true, Some(1), true);

    let forward = similarity(&forward_algorithm, &["Niall", "Bob"], &["Neal"]);
    let reverse = similarity(&reverse_algorithm, &["Neal"], &["Niall", "Bob"]);
    let symmetric = similarity(&symmetric_algorithm, &["Niall", "Bob"], &["Neal"]);

    assert_ne!(forward, reverse);
    assert_close(symmetric, (forward + reverse) / 2.0);
}

#[test]
fn monge_elkan_preserves_quick_answers_and_qvalue_preparation() {
    let algorithm = MongeElkan::from_python(JaroWinkler::default(), false, Some(0), true);
    let words = algorithm
        .try_similarity_inputs(&[
            InputSequence::Text("Niall".into()),
            InputSequence::Text("Neal".into()),
        ])
        .unwrap();
    assert_close(words, 0.805);

    let elements_algorithm = MongeElkan::from_python(JaroWinkler::default(), false, Some(1), true);
    assert_close(similarity(&elements_algorithm, &[], &[]), 1.0);
    assert_close(similarity(&elements_algorithm, &[], &["Neal"]), 0.0);
    assert_close(similarity(&elements_algorithm, &["same"], &["same"]), 1.0);
}

#[test]
fn monge_elkan_exposes_underlying_algorithm_maximum() {
    let jaro = MongeElkan::new(JaroWinkler::default(), false, QValue::Elements, true);
    let damerau = MongeElkan::new(DamerauLevenshtein::default(), false, QValue::Elements, true);
    let prepared =
        prepare_sequences(&[tokens(&["Niall"]), tokens(&["Neal"])], QValue::Elements).unwrap();

    assert_close(jaro.maximum(&prepared), 1.0);
    assert_close(damerau.maximum(&prepared), 5.0);
}

#[test]
fn monge_elkan_supports_ngram_outer_sequences() {
    let algorithm = MongeElkan::from_python(JaroWinkler::default(), false, Some(2), true);
    let score = algorithm
        .try_similarity_inputs(&[
            InputSequence::Text("Niall".into()),
            InputSequence::Text("Neal".into()),
        ])
        .unwrap();

    assert!((0.0..=1.0).contains(&score));
}
