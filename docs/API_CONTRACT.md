# Rust API Contract — G1

This is the shared contract for the parallel algorithm owners. It is frozen
before algorithm implementation begins. Changes to this file belong only to
Simha Teja and must be announced before any dependent branch is updated.

G1 verification evidence: [`proof/g1.md`](../proof/g1.md).

## Core boundary

The Rust core is independent of Python. It accepts normalized `InputSequence`
values and exposes algorithms through the `Algorithm` trait in
`rust/src/core/mod.rs`.

The initial adapter domain is:

- Python `str` → Rust `char` values, preserving Unicode code-point length;
- Python `bytes` → `u8` values;
- lists of integers → `i64` values;
- lists of booleans → `bool` values;
- already-normalized Rust `Element` values for native tests.

Unsupported arbitrary Python objects must produce an explicit adapter error.
They must not be stringified or silently coerced.

## Preparation contract

`QValue::from_python` maps the source behavior as follows:

| Python value | Rust meaning |
| --- | --- |
| `None` or `0` | split text into whitespace-separated `Text` tokens |
| `1` | compare individual elements |
| `n > 1` | compare sliding n-grams represented as `Element::Gram` |

All algorithm implementations receive prepared sequences. They must not
reimplement q-value preparation.

## Common algorithm contract

Each algorithm implements:

```rust
pub trait Algorithm {
    fn raw_score(&self, sequences: &[PreparedSequence]) -> f64;
    fn maximum(&self, sequences: &[PreparedSequence]) -> f64;
    fn score_mode(&self) -> ScoreMode;
    fn distance(&self, sequences: &[PreparedSequence]) -> f64;
    fn similarity(&self, sequences: &[PreparedSequence]) -> f64;
    fn normalized_distance(&self, sequences: &[PreparedSequence]) -> f64;
    fn normalized_similarity(&self, sequences: &[PreparedSequence]) -> f64;
    fn quick_answer(&self, sequences: &[PreparedSequence]) -> Option<f64>;
}
```

The trait supplies common conversions. An algorithm only overrides
`maximum`, `score_mode`, or `quick_answer` when the original Python source
does so.

## File and ownership contract

- One public algorithm per file under `rust/src/algorithms/`.
- One focused native test file per algorithm under `rust/tests/algorithms/`.
- All assigned module paths are declared in `rust/src/algorithms/mod.rs` from
  the scaffold. `rust/tests/registry.rs` imports every path so missing or
  misnamed packets fail at compile time.
- `rust/src/algorithms/mod.rs`, `rust/src/core/**`, and `python_adapter/**`
  are shared surfaces owned by Simha Teja.
- Algorithm owners replace only their assigned implementation/test files; they
  do not edit the registry, Cargo dependencies, or another owner’s files. They
  report API gaps instead.

## Compatibility priorities

1. Exact empty/equal behavior.
1. Correct Unicode and sequence lengths.
1. Correct constructor options and aliases.
1. Correct multi-sequence behavior where the source supports it.
1. Matching numeric results within the original test tolerance.
1. Performance only after behavior is proven.
