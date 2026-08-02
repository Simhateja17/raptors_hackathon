use textdistance_port::algorithms::simple::identity::Identity;
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
/// original Python `textdistance.identity` implementation.
const FIXTURES: &[(&str, &str, f64, f64, f64)] = &[
    ("", "", 1.0, 0.0, 1.0),
    ("a", "", 0.0, 1.0, 1.0),
    ("", "a", 0.0, 1.0, 1.0),
    ("a", "a", 1.0, 0.0, 1.0),
    ("a", "b", 0.0, 1.0, 1.0),
    ("abc", "abc", 1.0, 0.0, 1.0),
    ("abc", "abd", 0.0, 1.0, 1.0),
    ("spam", "qwer", 0.0, 1.0, 1.0),
    ("", "qwertyui", 0.0, 1.0, 1.0),
];

#[test]
fn matches_original_identity_fixtures() {
    let algorithm = Identity::new();
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
    let algorithm = Identity::new();
    let pair = text_pair("spam", "qwer");
    assert_eq!(algorithm.similarity(&pair), 0.0);
}

#[test]
fn empty_inputs_have_zero_distance() {
    let algorithm = Identity::new();
    let pair = text_pair("", "");
    assert_eq!(algorithm.distance(&pair), 0.0);
}

#[test]
fn unequal_length_inputs_have_positive_distance_when_maximum_is_nonzero() {
    let algorithm = Identity::new();
    let pair = text_pair("", "qwertyui");
    assert!(algorithm.maximum(&pair) > 0.0);
    assert!(algorithm.distance(&pair) > 0.0);
}

#[test]
fn normalization_matches_source_relationship() {
    let algorithm = Identity::new();
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
fn three_sequences_require_all_to_match() {
    let algorithm = Identity::new();
    let same = prepare_sequences(
        &[
            InputSequence::Text("a".to_owned()),
            InputSequence::Text("a".to_owned()),
            InputSequence::Text("a".to_owned()),
        ],
        QValue::Elements,
    )
    .expect("text preparation should succeed");
    assert_eq!(algorithm.similarity(&same), 1.0);

    let differing = prepare_sequences(
        &[
            InputSequence::Text("a".to_owned()),
            InputSequence::Text("a".to_owned()),
            InputSequence::Text("b".to_owned()),
        ],
        QValue::Elements,
    )
    .expect("text preparation should succeed");
    assert_eq!(algorithm.similarity(&differing), 0.0);
}

#[test]
fn single_sequence_is_always_identical_to_itself() {
    // Python's `_ident` special case: `len(set([x])) == 1` is always true
    // for exactly one sequence, regardless of its value.
    let algorithm = Identity::new();
    let single = text_pair("anything", "").into_iter().take(1).collect::<Vec<_>>();
    assert_eq!(algorithm.similarity(&single), 1.0);
    assert_eq!(algorithm.distance(&single), 0.0);
}

#[test]
fn zero_sequences_are_not_identical() {
    // Python's `_ident()` with no arguments is `False`, unlike the shared
    // `all_identical` helper which treats zero sequences as identical.
    let algorithm = Identity::new();
    let none: Vec<PreparedSequence> = Vec::new();
    assert_eq!(algorithm.similarity(&none), 0.0);
    assert_eq!(algorithm.distance(&none), 1.0);
    assert_eq!(algorithm.maximum(&none), 1.0);
}
