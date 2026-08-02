use std::collections::BTreeMap;

use textdistance_port::algorithms::simple::matrix::Matrix;
use textdistance_port::{prepare_sequences, Algorithm, InputSequence, PreparedSequence, QValue};

/// Prepare one whole raw input as a `PreparedSequence`. Matrix does not
/// split by qval in the Python source, so `QValue::Elements` (no splitting)
/// is the faithful choice here.
fn seq(text: &str) -> PreparedSequence {
    prepare_sequences(&[InputSequence::Text(text.to_owned())], QValue::Elements).unwrap()[0]
        .clone()
}

fn seqs(texts: &[&str]) -> Vec<PreparedSequence> {
    texts.iter().map(|t| seq(t)).collect()
}

// Matrix has no dedicated file under `tests/original`; reference values
// below were captured by running `textdistance.Matrix` directly against
// `textdistance/algorithms/simple.py`.

#[test]
fn identity_fallback_matches_source_when_no_matrix_is_configured() {
    let algorithm = Matrix::default();

    // Python: `Matrix()()` (zero sequences) calls `_ident()` directly,
    // bypassing `quick_answer`'s empty-sequence shortcut. `_ident()` with no
    // elements is `len(set(())) == 1` -> False -> mismatch_cost (0).
    assert_eq!(algorithm.call(&[]), 0.0);

    // A single sequence is trivially identical to itself -> match_cost (1).
    assert_eq!(algorithm.call(&seqs(&["a"])), 1.0);

    assert_eq!(algorithm.call(&seqs(&["a", "a"])), 1.0);
    assert_eq!(algorithm.call(&seqs(&["a", "b"])), 0.0);
    assert_eq!(algorithm.call(&seqs(&["abc", "abc"])), 1.0);
    assert_eq!(algorithm.call(&seqs(&["abc", "abd"])), 0.0);

    // Three sequences: identity requires all of them to match.
    assert_eq!(algorithm.call(&seqs(&["a", "a", "a"])), 1.0);
    assert_eq!(algorithm.call(&seqs(&["a", "a", "b"])), 0.0);
}

#[test]
fn matrix_lookup_matches_source_including_symmetric_fallback() {
    let mut mat = BTreeMap::new();
    mat.insert(seqs(&["a", "b"]), 5.0);
    mat.insert(seqs(&["a", "a"]), 10.0);
    let algorithm = Matrix::new(Some(mat), 0.0, 1.0, true, true);

    assert_eq!(algorithm.call(&seqs(&["a", "b"])), 5.0);
    // Symmetric: the reversed pair also resolves to the same entry.
    assert_eq!(algorithm.call(&seqs(&["b", "a"])), 5.0);
    assert_eq!(algorithm.call(&seqs(&["a", "a"])), 10.0);

    // Key not found (even after trying the reversal): fall back to identity.
    assert_eq!(algorithm.call(&seqs(&["c", "c"])), 1.0);
    assert_eq!(algorithm.call(&seqs(&["c", "d"])), 0.0);

    // Matrix entries may key on whole multi-character sequences, not just
    // single elements.
    let mut whole = BTreeMap::new();
    whole.insert(seqs(&["test", "text"]), 7.0);
    let whole_algorithm = Matrix::new(Some(whole), 0.0, 1.0, true, true);
    assert_eq!(whole_algorithm.call(&seqs(&["test", "text"])), 7.0);
    assert_eq!(whole_algorithm.call(&seqs(&["foo", "bar"])), 0.0);
}

#[test]
fn non_symmetric_matrix_only_matches_the_exact_key_order() {
    let mut mat = BTreeMap::new();
    mat.insert(seqs(&["a", "b"]), 5.0);
    let algorithm = Matrix::new(Some(mat), 0.0, 1.0, false, true);

    assert_eq!(algorithm.call(&seqs(&["a", "b"])), 5.0);
    // Reversal is not attempted, and 'b'/'a' are not identical either.
    assert_eq!(algorithm.call(&seqs(&["b", "a"])), 0.0);
}

#[test]
fn three_sequence_matrix_lookup_only_tries_full_reversal() {
    let mut mat = BTreeMap::new();
    mat.insert(seqs(&["a", "b", "c"]), 9.0);
    let algorithm = Matrix::new(Some(mat), 0.0, 1.0, true, true);

    assert_eq!(algorithm.call(&seqs(&["a", "b", "c"])), 9.0);
    assert_eq!(algorithm.call(&seqs(&["c", "b", "a"])), 9.0);
    // Not the exact key and not its full reversal: falls back to identity,
    // and these three sequences are not all equal, so mismatch_cost.
    assert_eq!(algorithm.call(&seqs(&["b", "a", "c"])), 0.0);
}

#[test]
fn empty_matrix_behaves_like_no_matrix_configured() {
    // Python: `not self.mat` is True for both `None` and `{}`.
    let algorithm = Matrix::new(Some(BTreeMap::new()), 0.0, 1.0, true, true);
    assert_eq!(algorithm.call(&seqs(&["a", "a"])), 1.0);
    assert_eq!(algorithm.call(&seqs(&["a", "b"])), 0.0);
}

#[test]
fn custom_costs_and_common_methods_match_source() {
    let algorithm = Matrix::new(None, -1.0, 2.0, true, true);
    assert_eq!(algorithm.call(&seqs(&["x", "x"])), 2.0);
    assert_eq!(algorithm.call(&seqs(&["x", "y"])), -1.0);
    assert_eq!(algorithm.maximum(&seqs(&["x", "y"])), 2.0);

    let default_algorithm = Matrix::default();
    let sequences = seqs(&["a", "b"]);
    assert_eq!(default_algorithm.similarity(&sequences), 0.0);
    assert_eq!(default_algorithm.distance(&sequences), 1.0);
    assert_eq!(default_algorithm.maximum(&sequences), 1.0);
    assert_eq!(default_algorithm.normalized_distance(&sequences), 1.0);
}
