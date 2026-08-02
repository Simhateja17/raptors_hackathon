# Behavior card — Jaro

Source: `textdistance/algorithms/edit_based.py` (`Jaro`, a thin subclass of `JaroWinkler`)
Target: `rust/src/algorithms/edit/jaro.rs`
Original tests: `tests/original/test_edit/test_jaro.py`

## What it does

Character-matching similarity measure. Counts characters common to both
strings (matched within a bounded search window, not requiring identical
position), then counts transpositions among the matched characters, and
combines both into a weighted score. Higher = more similar, max `1.0`.

`Jaro` is **not** a separate algorithm in the source — it's `JaroWinkler`
with the Winkler prefix-boost permanently disabled:

```python
class Jaro(JaroWinkler):
    def __init__(self, long_tolerance=False, qval=1, external=True):
        super().__init__(long_tolerance=long_tolerance, winklerize=False,
                          qval=qval, external=external)
```

So the Rust port of Jaro should implement the *shared* matching/scoring core
once (matched-character search-window scan → transposition count → weighted
average), then Jaro-Winkler (card next in line) layers the prefix boost on
top of the same core rather than duplicating it. See "known risks" for why
this matters for file ownership.

## Algorithm (from `JaroWinkler.__call__`, with `winklerize=False`)

1. If either string is empty, return `0.0`.
2. `search_range = max(len(s1), len(s2)) // 2 - 1` (clamped to `>= 0`).
3. For each character in `s1`, look for an unflagged matching character in
   `s2` within `[i - search_range, i + search_range]`; flag both sides on
   match. Count `common_chars`.
4. If no characters matched, return `0.0`.
5. Walk matched characters in order on both sides; every position where the
   matched characters differ counts as a transposition; final
   `trans_count // 2`.
6. `weight = (common/len(s1) + common/len(s2) + (common - trans_count)/common) / 3`.
7. Since `winklerize=False`, return `weight` directly (no prefix boost, no
   long-string adjustment).

## Inputs / options

- `long_tolerance: bool = False` — accepted by the constructor but has
  **no effect** when `winklerize=False`, since the long-tolerance branch is
  inside the `if not self.winklerize: return weight` early-return path.
  Still must be present on the Rust struct for API-shape parity, per
  `docs/API_CONTRACT.md`'s constructor-options-and-aliases priority.
  Actually verify: the winklerize check happens *before* long_tolerance is
  ever consulted, so for `Jaro` specifically, `long_tolerance` is dead
  configuration — worth a code comment in the Rust port, not silent
  removal (the option must still exist for API-shape parity with Python).
- `qval: int = 1` — standard q-value preparation (element comparison; not
  word-splitting or n-grams by default).
- `external: bool = True` — accepted for API compatibility; Rust core has
  no external-library path, so this is a no-op flag per the project's
  overall external-library exclusion decision.
- Call signature takes a `prefix_weight: float = 0.1` parameter even though
  Jaro itself never uses it (only `JaroWinkler` with `winklerize=True`
  does) — must exist on the trait signature since it's shared with
  Jaro-Winkler.
- `maximum()` is always `1`.

## Edge cases

- Either string empty → `0.0` (explicit early return, not derived from the
  formula).
- No common characters at all → `0.0` (explicit early return after the
  match-scan, e.g. `'fly'` vs `'ant'`).
- Identical strings → all characters match, zero transpositions → `1.0`
  in principle (not directly in the fixed examples below, but follows from
  the formula: `common=len`, `trans_count=0` → `weight = (1+1+1)/3 = 1`).
- Unicode: matching is by scalar value (`s1[i] == s2[j]`), no
  transliteration or normalization — must compare Rust `char`s, matching
  `docs/API_CONTRACT.md`'s code-point-length rule.

## Worked examples

From `tests/original/test_edit/test_jaro.py` — note the test file calls
`textdistance.JaroWinkler(winklerize=False, ...)` rather than
`textdistance.Jaro(...)` directly, but since `Jaro.__init__` forces
`winklerize=False`, these values are exactly the Jaro algorithm's ground
truth:

| left | right | expected |
| --- | --- | --- |
| `hello` | `haloa` | `0.7333333333333334` |
| `fly` | `ant` | `0.0` |
| `frog` | `fog` | `0.9166666666666666` |
| `ATCG` | `TAGC` | `0.8333333333333334` |
| `MARTHA` | `MARHTA` | `0.944444444` |
| `DWAYNE` | `DUANE` | `0.822222222` |
| `DIXON` | `DICKSONX` | `0.7666666666666666` |
| `Sint-Pietersplein 6, 9000 Gent` | `Test 10, 1010 Brussel` | `0.5182539682539683` |

That's already 8 examples — well above the 3–5 minimum, and it includes
long strings with punctuation/spaces (the last row), which is good
Unicode/edge coverage.

## Numeric tolerance

`math.isclose(actual, expected)` in the original test — default
`rel_tol=1e-9`. No compression-style byte-length ambiguity here; this is a
pure arithmetic algorithm, so exact parity (within float rounding) is
achievable and expected, unlike the compression cards.

## Dependencies / compressor settings

None — pure algorithm, no external library, no compressor. Nothing goes in
the `MAN-09` dependency note for this one.

## Known risks

- **Shared-core risk with Jaro-Winkler:** since Jaro and Jaro-Winkler are
  one algorithm in Python (subclass relationship), there's a design
  decision for the Rust side: implement the matching/transposition core
  once and have both `jaro.rs` and `jaro_winkler.rs` call it, or duplicate
  the core in both files. Per `docs/API_CONTRACT.md`'s "one file, one
  owner" rule, both files are still owned solely by Manasa, so sharing code
  between them is fine — just don't leak the shared core into a file owned
  by someone else or into `rust/src/algorithms/mod.rs`.
- The transposition-counting loop (`for j in range(k, s2_len): if
  s2_flags[j]: k = j + 1; break`) has a subtle indexing dependency: `j` is
  read *after* the loop using the loop variable's last value, which is a
  Python-ism (loop variables leak out of the loop scope) — Rust's `for`
  loop does not do this. This needs explicit handling (e.g. track `j`
  outside the loop) when translating, not a direct one-to-one port of the
  loop shape.
- `search_range` can be `0` for short/near-equal-length strings — confirm
  the window-clamping logic (`max(0, i - search_range)` /
  `min(i + search_range, s2_len - 1)`) handles a zero range without
  underflow in Rust (Python's `max(0, negative)` has no direct unsigned
  analog — watch for `usize` underflow if ported naively).
