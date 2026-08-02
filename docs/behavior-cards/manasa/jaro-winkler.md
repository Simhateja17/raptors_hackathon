# Behavior card — Jaro-Winkler

Source: `textdistance/algorithms/edit_based.py` (`JaroWinkler`)
Target: `rust/src/algorithms/edit/jaro_winkler.rs`
Original tests: `tests/original/test_edit/test_jaro_winkler.py`

## What it does

Jaro similarity (see `docs/behavior-cards/manasa/jaro.md` for the shared
matching/transposition core) plus a boost for strings that share a common
prefix — designed to score prefix-similar strings higher than plain Jaro
would. Higher = more similar, max `1.0`.

## Algorithm — the parts that differ from plain Jaro

Everything through step 6 in the Jaro card is identical. From there:

1. If `winklerize=False` **or** `weight <= 0.7`, return `weight` as-is
   (same as Jaro — the boost only applies to already-fairly-similar
   strings).
1. **Prefix boost:** find the length of the common prefix, up to 4
   characters (`j = min(min_len, 4)`), by scanning while
   `s1[i] == s2[i]`. If any prefix matched (`i > 0`):
   `weight += i * prefix_weight * (1.0 - weight)`.
1. **Long-string adjustment** (only if `long_tolerance=True` **and**
   `min_len > 4`, and only if `common_chars > i + 1` and
   `2 * common_chars >= min_len + i`):
   `tmp = (common_chars - i - 1) / (s1_len + s2_len - i*2 + 2)`;
   `weight += (1.0 - weight) * tmp`.

Step 9 is the one place `long_tolerance` actually matters — unlike plain
Jaro, where it's dead configuration (see Jaro's card). This is the reason
`long_tolerance` exists as a constructor option at all.

## Inputs / options

- `winklerize: bool = True` — the flag that distinguishes this from Jaro.
  When `False`, `JaroWinkler` behaves identically to `Jaro` (in fact `Jaro`
  is implemented by setting this to `False`).
- `long_tolerance: bool = False` — only meaningful when `winklerize=True`
  and `weight > 0.7` and `min_len > 4` (see step 9 above). Not dead here,
  unlike in plain Jaro.
- `qval: int = 1`, `external: bool = True` — same as Jaro.
- Call signature: `prefix_weight: float = 0.1` — **this one matters** for
  Jaro-Winkler (unlike Jaro, which ignores it). Controls how much each
  matching prefix character boosts the score in step 8.
- `maximum()` is always `1`.

## Edge cases

- Same empty/no-match early returns as Jaro (`0.0`).
- `weight <= 0.7` after the core Jaro computation → no boost applied, same
  result as plain Jaro would give.
- Strings with no common prefix (`i == 0` after the scan) → no boost, even
  if `weight > 0.7`.
- Strings differing only after position 4 → prefix scan caps at 4
  characters regardless of how much more they share beyond that.

## Worked examples

From `tests/original/test_edit/test_jaro_winkler.py` (`winklerize=True`,
the default):

| left | right | expected |
| --- | --- | --- |
| `elephant` | `hippo` | `0.44166666666666665` |
| `fly` | `ant` | `0.0` |
| `frog` | `fog` | `0.925` |
| `MARTHA` | `MARHTA` | `0.9611111111111111` |
| `DWAYNE` | `DUANE` | `0.84` |
| `DIXON` | `DICKSONX` | `0.8133333333333332` |
| `duck donald` | `duck daisy` | `0.867272727272` |

Useful comparison against the Jaro card's identical input pairs — same
strings, different algorithm, different (higher, due to prefix boost)
score:

| left | right | Jaro | Jaro-Winkler |
| --- | --- | --- | --- |
| `frog` | `fog` | `0.9166666666666666` | `0.925` |
| `MARTHA` | `MARHTA` | `0.944444444` | `0.9611111111111111` |
| `DWAYNE` | `DUANE` | `0.822222222` | `0.84` |
| `DIXON` | `DICKSONX` | `0.7666666666666666` | `0.8133333333333332` |
| `fly` | `ant` | `0.0` | `0.0` (no match → no boost possible) |

That side-by-side is a good sanity check for the Rust implementation: if
the shared core is correct (verified by Jaro's tests) and only the boost
logic differs, these deltas should reproduce exactly.

## Numeric tolerance

Same as Jaro: `math.isclose`, default `rel_tol=1e-9`. Pure arithmetic, no
compression ambiguity, exact parity expected.

## Dependencies / compressor settings

None.

## Known risks

- Depends entirely on the shared Jaro core being correct first — implement
  and test Jaro before Jaro-Winkler, not in parallel, since a bug in the
  shared core would silently break both.
- `long_tolerance`'s condition chain (`common_chars <= i + 1 or 2 *
  common_chars < min_len + i` as an early-return-without-boost check) is
  easy to get backwards when translating boolean logic — write a
  dedicated native test specifically for `long_tolerance=True` with a
  string pair long enough to trigger it (`min_len > 4`), since none of the
  7 fixed examples above use `long_tolerance=True` at all — that flag is
  completely unexercised by the frozen fixtures.
- Same Python loop-variable-leak and `usize`-underflow-on-search-range
  cautions as noted in the Jaro card, since this reuses that same scan.
