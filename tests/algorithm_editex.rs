//! Native tests for Editex.
//!
//! Fixture values are taken from
//! `tests/original/test_phonetic/test_editex.py`. See
//! `docs/behavior-cards/manasa/editex.md` for the full behavior card.

use textdistance_port::algorithms::editex::Editex;
use textdistance_port::{prepare_sequences, Algorithm, InputSequence, QValue};

fn distance_with(algorithm: &Editex, left: &str, right: &str) -> f64 {
    let sequences = prepare_sequences(
        &[
            InputSequence::Text(left.to_owned()),
            InputSequence::Text(right.to_owned()),
        ],
        QValue::Elements,
    )
    .expect("text preparation should succeed");
    algorithm.distance(&sequences)
}

fn distance(left: &str, right: &str) -> f64 {
    distance_with(&Editex::default(), left, right)
}

#[test]
fn matches_frozen_fixture_values_non_local() {
    let cases: &[(&str, &str, f64)] = &[
        ("", "", 0.0),
        ("nelson", "", 12.0),
        ("", "neilsen", 14.0),
        ("ab", "a", 2.0),
        ("ab", "c", 4.0),
        ("nelson", "neilsen", 2.0),
        ("neilsen", "nelson", 2.0),
        ("niall", "neal", 1.0),
        ("neal", "niall", 1.0),
        ("niall", "nihal", 2.0),
        ("nihal", "niall", 2.0),
        ("neal", "nihl", 3.0),
        ("nihl", "neal", 3.0),
        ("cat", "hat", 2.0),
        ("Niall", "Neil", 2.0),
        ("aluminum", "Catalan", 12.0),
        ("ATCG", "TAGC", 6.0),
    ];
    for &(left, right, expected) in cases {
        assert_eq!(
            distance(left, right),
            expected,
            "distance({left:?}, {right:?})"
        );
    }
}

#[test]
fn matches_frozen_fixture_values_local() {
    let local = Editex::new(true, 0, 1, 2, true);
    let cases: &[(&str, &str, f64)] = &[
        ("", "", 0.0),
        ("nelson", "", 12.0),
        ("", "neilsen", 14.0),
        ("ab", "a", 2.0),
        ("ab", "c", 2.0),
        ("nelson", "neilsen", 2.0),
        ("neilsen", "nelson", 2.0),
        ("niall", "neal", 1.0),
        ("neal", "niall", 1.0),
        ("niall", "nihal", 2.0),
        ("nihal", "niall", 2.0),
        ("neal", "nihl", 3.0),
        ("nihl", "neal", 3.0),
    ];
    for &(left, right, expected) in cases {
        assert_eq!(
            distance_with(&local, left, right),
            expected,
            "local distance({left:?}, {right:?})"
        );
    }
}

#[test]
fn local_and_non_local_diverge_on_ab_vs_c() {
    // The clearest example of the row-0-initialization asymmetry: local
    // mode skips paying for an unmatched leading prefix on the `s1` side.
    let non_local = distance("ab", "c");
    let local = distance_with(&Editex::new(true, 0, 1, 2, true), "ab", "c");
    assert_eq!(non_local, 4.0);
    assert_eq!(local, 2.0);
    assert_ne!(non_local, local);
}

#[test]
fn empty_input_uses_maximum_not_the_cheaper_dp_path() {
    // Regression test for the exact bug this behavior card and
    // implementation flagged: without the `quick_answer`-equivalent
    // shortcut, the natural DP computation for '' vs 'neilsen' would give
    // 13 (since 'E'/'I' share a phonetic group and cost only 1, not 2),
    // not the correct 14. This must stay at 14 (`len('neilsen') * 2`).
    assert_eq!(distance("", "neilsen"), 14.0);
    assert_eq!(distance("nelson", ""), 12.0);
}

#[test]
fn identical_strings_score_zero() {
    assert_eq!(distance("identical", "identical"), 0.0);
}

#[test]
fn distance_is_symmetric_for_fixed_examples() {
    assert_eq!(distance("nelson", "neilsen"), distance("neilsen", "nelson"));
    assert_eq!(distance("niall", "neal"), distance("neal", "niall"));
}

#[test]
fn fails_if_implementation_becomes_a_trivial_constant() {
    assert_ne!(distance("cat", "hat"), 1.0);
    assert_ne!(distance("ATCG", "TAGC"), 1.0);
}
