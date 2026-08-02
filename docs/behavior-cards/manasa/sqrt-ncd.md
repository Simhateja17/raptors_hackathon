# Behavior card — Sqrt NCD

Source: `textdistance/algorithms/compression_based.py` (`SqrtNCD`, via `_NCDBase`)
Target: `rust/src/algorithms/compression/sqrt_ncd.rs`
Original tests: `tests/original/test_compression/test_sqrt_ncd.py`, `tests/original/test_compression/test_common.py`

## What it does

Same NCD formula as BZ2/LZMA/ZLIB NCD, but the "compressed size" estimator
is pure math instead of a real compressor — no external library involved
at all. This is the **lowest-risk** card in the set of 8.

```text
NCD(a, b) = (C(a+b) - min(C(a), C(b))) / max(C(a), C(b))
```

`SqrtNCD._compress(data)` returns, for each *distinct* element in the
input, the square root of its occurrence count:
`{element: sqrt(count) for element, count in Counter(data).items()}`.
`_get_size(data)` sums those square roots. So "compressed size" here is a
stand-in metric, not an actual byte length — it approximates how
compressible repetitive data is (repeated elements contribute
sub-linearly, via `sqrt`, instead of linearly).

## Inputs / options

- `qval: int = 1` — **unlike** BZ2/LZMA/ZLIB NCD, this one *does* take
  `qval` (inherited straight from `_NCDBase`, not overridden the way
  `_BinaryNCDBase` overrides it to `pass`). Standard q-value prep applies:
  `None`/`0` → word split, `1` → individual elements, `n>1` → n-grams.
- No string/bytes distinction — works over any hashable sequence element
  (not limited to `str`/`bytes` like the binary compressors).
- `maximum()` is always `1`.

## Edge cases

- Empty/empty: `Counter()` is empty, size sums to `0` for both →
  `max_len == 0` branch in `_NCDBase.__call__` returns `0` directly.
- Repeated characters: this is where `sqrt` matters most — e.g. `'aaaa'`
  has one distinct element with count 4, so `_compress` gives
  `{'a': sqrt(4)} = {'a': 2.0}`, not `4.0`.
- q-grams: with `qval > 1`, the "elements" being counted are n-grams, not
  raw characters — same `sqrt(count)` logic applies to gram frequency.

## Worked examples

From `tests/original/test_compression/test_sqrt_ncd.py`:

| left | right | expected |
| --- | --- | --- |
| `test` | `test` | `0.41421356237309503` |
| `test` | `nani` | `1` |

Plus the shared `test_common.py` monotonicity check:
`sqrt_ncd('test','test') <= sqrt_ncd('test','text') <= sqrt_ncd('test','nani')`.

Only 2 fixed numeric examples, but this card has something the compression
cards don't: **dedicated property-based tests on the internal compressor**
(`test_simmetry_compressor`, `test_idempotency_compressor`,
`test_monotonicity_compressor`, `test_distributivity_compressor` — all in
`test_sqrt_ncd.py`, using `hypothesis` to generate arbitrary text). These
assert mathematical properties of `_compress`/`_get_size` directly:

- **Anagram symmetry:** `_compress(text) == _compress(reversed(text))` —
  makes sense since it's just counting elements, order doesn't matter.
- **Sub-linear growth:** `_get_size(text * 2) < _get_size(text) * 2` —
  doubling the input doesn't double the size (confirms `sqrt` is actually
  being applied, not a no-op).
- **Monotonicity on append:** appending a new character never decreases
  size: `_get_size(left) <= _get_size(left + right)`.
- **Subadditivity:** `_get_size(a+b) + _get_size(c) <= _get_size(a+c) +
  _get_size(b+c)` for any three texts.

These test **internal Python methods** (`_compress`, `_get_size`) that
aren't part of the public `Algorithm` trait in `docs/API_CONTRACT.md` — the
Rust port doesn't need to expose equivalent public methods. But whatever
internal size-estimation function the Rust implementation uses internally
should satisfy the same four mathematical properties, since they're what
make the NCD formula behave sensibly. Worth adding as native unit tests on
the internal function even without a 1:1 method-name mapping.

## Numeric tolerance

`math.isclose`, default `rel_tol=1e-9`. Pure arithmetic (square roots),
exact float parity is achievable — no compression-library ambiguity here.

## Dependencies / compressor settings

None — pure math (`sqrt` from Rust's standard library `f64::sqrt`).
Nothing goes in the `MAN-09` dependency note for this one.

## Known risks

- Low risk overall — this is the safest of the 8 cards. Main thing to get
  right is the `Counter`-then-`sqrt`-then-sum sequence exactly, and to
  remember `qval` prep applies here (unlike the binary compressors).
- Floating-point summation order: `_get_size` sums an unordered dict's
  values in Python; Rust's equivalent (e.g. iterating a `HashMap`) has
  non-deterministic order too, and floating-point addition isn't strictly
  associative, so summation order *could* cause tiny last-bit differences
  across runs. Unlikely to matter at `rel_tol=1e-9`, but worth knowing if
  a test ever flakes by an ULP.
