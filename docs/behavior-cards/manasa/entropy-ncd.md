# Behavior card — Entropy NCD

Source: `textdistance/algorithms/compression_based.py` (`EntropyNCD`, via `_NCDBase`)
Target: `rust/src/algorithms/compression/entropy_ncd.rs`
Original tests: `tests/original/test_compression/test_entropy_ncd.py`, `tests/original/test_compression/test_common.py`

## What it does

Same NCD formula as Sqrt NCD (see that card) and the binary compressors,
but the "compressed size" estimator is Shannon entropy — another pure-math
stand-in, no external library:

```text
NCD(a, b) = (C(a+b) - min(C(a), C(b))) / max(C(a), C(b))
```

`EntropyNCD._compress(data)` computes Shannon entropy of the element
distribution: `entropy = -sum(p * log(p, base) for p in
element_frequencies)`, where `p = count / total_count` per distinct
element. `_get_size(data) = coef + entropy` — a constant offset (`coef`,
default `1`) is added to the raw entropy before it's used as a "size."

## Inputs / options

- `qval: int = 1` — same as Sqrt NCD, standard q-value prep applies (word
  split / individual elements / n-grams).
- `coef: int = 1` — constant added to entropy before treating it as
  "size." Not present on any other card in this set; specific to Entropy
  NCD.
- `base: int = 2` — logarithm base for entropy calculation (bits, by
  default). Directly affects the numeric output — changing `base` changes
  every score, not just a cosmetic option.
- `maximum()` is always `1`.
- Source has `assert entropy >= 0` inside `_compress` — a sanity
  invariant (entropy is mathematically always non-negative for a valid
  probability distribution). The Rust port should preserve this as a
  debug assertion or equivalent, since it documents an expected invariant
  of the math, not defensive-only code.

## Edge cases

- Empty/empty: both entropies undefined by formula (no elements to sum
  over) — need to check how the source handles zero-length input to
  `_compress`/`_get_size` before assuming behavior; likely returns `0`
  entropy for empty (empty sum), giving `coef` as the size, but confirm
  against the empty-input results of `test_common.py`'s shared tests since
  entropy_ncd participates in that shared suite.
- Uniform distribution (e.g. `'aaa'`): every character identical → single
  distinct element, `p=1`, `log(1, base) = 0` → entropy `= 0`.
- Maximally spread distribution (e.g. all-distinct characters): higher
  entropy, approaching `log(n, base)` for `n` distinct equally-likely
  elements.

## Worked examples

From `tests/original/test_compression/test_entropy_ncd.py` — **note this
test file asserts on `.similarity()`, not the raw call**, unlike every
other card in this set:

| left | right | expected similarity |
| --- | --- | --- |
| `test` | `test` | `1` |
| `aaa` | `bbb` | `0` |
| `test` | `nani` | `0.6` |

Since `maximum()` is always `1` for NCD algorithms and `EntropyNCD` extends
`_Base` (not `_BaseSimilarity`), `similarity = maximum() - distance()`, so
the equivalent raw/distance values are `0`, `1`, and `0.4` respectively —
worth double-checking against the trait's common-method conversion in
`docs/API_CONTRACT.md` when writing the native test, since this card is
the one place in your 8 where the frozen fixture is expressed in
similarity terms rather than raw distance.

Plus the same 4 hypothesis-based internal-compressor properties as Sqrt
NCD (symmetry under reversal, sub-linear growth under doubling,
monotonicity on append, subadditivity) — see `test_entropy_ncd.py`'s
`test_simmetry_compressor` / `test_idempotency_compressor` /
`test_monotonicity_compressor` / `test_distributivity_compressor`. Same
caveat as Sqrt NCD: these test internal `_compress`/`_get_size` methods
not present on the public trait, but the underlying math properties should
still hold for whatever internal size function the Rust port uses.

## Numeric tolerance

`math.isclose`, default `rel_tol=1e-9`. Pure arithmetic (logarithms), exact
float parity is achievable.

## Dependencies / compressor settings

None — pure math (`f64::log` / `f64::log2` in Rust, matching whatever
`base` is configured, default `2`).

## Known risks

- Low risk, similar to Sqrt NCD — no external library, no byte-length
  ambiguity.
- **The similarity-vs-distance framing of the fixed examples** (see
  worked examples above) is the one thing to get right early — if the
  native test asserts the raw/distance value against the similarity
  numbers in the table without converting, all three fixed examples will
  fail even with a correct implementation. Confirm the conversion before
  writing the assertion.
- `base` as a configurable log base is unique to this card — make sure the
  Rust struct actually threads `base` through to the log calls rather than
  hardcoding base-2, even though the default and all fixed examples use
  base 2.
- Same floating-point summation-order caution as Sqrt NCD (non-deterministic
  iteration order over grouped elements, floating-point addition not
  strictly associative) — unlikely to matter at `rel_tol=1e-9`.
