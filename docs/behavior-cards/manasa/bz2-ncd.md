# Behavior card — BZ2 NCD

Source: `textdistance/algorithms/compression_based.py` (`BZ2NCD`, via `_BinaryNCDBase` and `_NCDBase`)
Target: `rust/src/algorithms/compression/bz2_ncd.rs`
Original tests: `tests/original/test_compression/test_bz2_ncd.py`, `tests/original/test_compression/test_common.py`

## What it does

Normalized Compression Distance (NCD) using the bz2 compressor as the size
estimator. Compresses each input alone and the concatenation of both inputs,
then scores similarity from how much smaller the concatenation compresses
relative to compressing the inputs separately — the intuition being that
similar inputs compress better together than different ones.

Formula (`_NCDBase.__call__`, 2-sequence case):

```text
NCD(a, b) = (C(a+b) - min(C(a), C(b))) / max(C(a), C(b))
```

where `C(x)` is the compressed byte length of `x`, and `C(a+b)` uses
whichever concatenation order (`a+b` or `b+a`) compresses smaller.

`BZ2NCD._compress` calls `codecs.encode(data, 'bz2_codec')` and strips the
first **15 bytes** (the bz2 stream header) before measuring length.

## Inputs / options

- No constructor options — `_BinaryNCDBase.__init__` is `pass`, so `BZ2NCD()`
  takes no `qval` or other arguments (unlike most other algorithms in this
  project). No q-gram behavior applies here.
- Accepts `str` or `bytes`. If given `str`, `_BinaryNCDBase.__call__` first
  encodes to UTF-8 bytes before compressing.
- `maximum()` is always `1` (inherited from `_NCDBase`).

## Edge cases

- Empty/empty: both sides compress to the same (near-zero) header-only size;
  confirm exact behavior against the bz2 codec rather than assuming 0.
- Equal inputs: `C(a+a)` vs `C(a)` — still governed by the same formula, not
  special-cased in source.
- Completely different inputs: covered by `test_monotonicity` in
  `test_common.py`.
- Unicode input: must be UTF-8-encoded before compression, per
  `_BinaryNCDBase`.
- `bytes` input: passed to the compressor unchanged, no encoding step.

## Worked examples

Only 2 fixed-value examples exist in the frozen original suite — this
algorithm is thin on frozen fixtures, so treat these as the full ground
truth rather than a sample:

| left | right | expected |
| --- | --- | --- |
| `test` | `test` | `0.08` |
| `test` | `nani` | `0.16` |

Additional *qualitative* (non-numeric) evidence from
`test_common.py::test_monotonicity`, shared across all NCD algorithms:

```text
bz2_ncd('test', 'test') <= bz2_ncd('test', 'text') <= bz2_ncd('test', 'nani')
```

And from the same file, these properties must hold for any input pair
(not specific numbers, but invariants the Rust port must preserve):

- `similarity(a, b) == similarity(b, a)` and same for `distance` (symmetry).
- `distance(a, b) == normalized_distance(a, b)` (since `maximum() == 1`).
- `normalized_similarity(a, b) + normalized_distance(a, b) ≈ 1`.

## Numeric tolerance

Original tests use `math.isclose(actual, expected)` — Python's default
relative tolerance (`rel_tol=1e-9`). That tolerance is only meaningful if the
Rust side reproduces byte-identical compressed output; see "known risks"
below for why that's not guaranteed here.

## Dependencies / compressor settings

- Python side: standard library `bz2` via the `codecs` module's `bz2_codec`,
  default compression settings (Python's codec doesn't expose a compression
  level here — it's whatever `bz2_codec` defaults to internally).
- Rust side: no crate chosen yet. Candidate: `bzip2` crate (libbz2 binding).
  Per project rule, Manasa does not add this to `Cargo.toml` — it goes in the
  `docs/dependency-notes/manasa.md` handoff (task `MAN-09`) for Simha Teja to
  review and add (tracked as `DEP-02`/`DEP-03` in the PRD).

## Known risks

- **Exact-byte-match risk (flagged in PRD §12 risk register, owner: Manasa):**
  a different bz2 binding/version may produce a different compressed byte
  length than Python's `bz2_codec`, even for identical input and algorithm.
  If so, exact numeric parity with the two table values above may not be
  achievable. Per the team's agreed fallback, this is acceptable — document
  any deviation as a compatibility caveat in `DECISIONS.md` rather than
  chasing byte-perfect parity.
- The `[15:]` header-strip is specific to Python's `bz2_codec` header format;
  a Rust bz2 crate's raw output framing may differ, and the strip offset may
  need to change or become unnecessary depending on what the crate returns.
- Only 2 fixed examples exist in the frozen suite — low fixture coverage
  means most confidence must come from the shared property tests
  (monotonicity, symmetry, normalization) rather than exact-value tests.
