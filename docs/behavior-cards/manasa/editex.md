# Behavior card — Editex

Source: `textdistance/algorithms/phonetic.py` (`Editex`)
Target: `rust/src/algorithms/phonetic/editex.rs`
Original tests: `tests/original/test_phonetic/test_editex.py`

## What it does

Edit-distance variant that gives reduced cost to substitutions between
letters in the same "phonetic group" (e.g. `B`/`P`, or `C`/`K`/`Q`) instead
of always charging full mismatch cost — the idea being that phonetically
similar substitutions should count as "cheaper" edits than arbitrary ones.
Lower = more similar; `0` means identical. This is a **distance**, not a
similarity (`_Base`, not `_BaseSimilarity`), and unlike Jaro/Jaro-Winkler,
scores are always non-negative integers, not floats in `[0, 1]`.

## Phonetic groups (fixed data, must be ported exactly)

```text
AEIOUY | BP | CKQ | DT | LR | MN | GJ | FPV | SXZ | CSZ
ungrouped: HW
```

Costs, constructor-clamped so `match_cost <= group_cost <= mismatch_cost`
always holds even if constructed with an inconsistent ordering:

- `match_cost = 0` (default) — identical letters.
- `group_cost = max(group_cost, match_cost) = 1` (default) — both letters
  in the same phonetic group.
- `mismatch_cost = max(mismatch_cost, group_cost) = 2` (default) — no
  shared group.

## Algorithm

Dynamic-programming edit distance (like Levenshtein) but with two cost
functions instead of one:

- **`r_cost(a, b)`** (substitution cost): `match_cost` if identical; if
  either letter isn't in *any* group, `mismatch_cost`; else `group_cost` if
  both letters share a group, otherwise `mismatch_cost`.
- **`d_cost(a, b)`** (used for the DP matrix's row/column initialization —
  effectively insertion/deletion cost along a single string): same as
  `r_cost`, **except** if the letters differ and the *first* one is in the
  ungrouped set (`H` or `W`), cost is `group_cost` instead of falling
  through to `r_cost`'s stricter rule. This is the mechanism that makes
  silent letters like `H` cheap to insert/delete.

Both strings are uppercased and prefixed with a leading space before
building the DP matrix (`s1 = ' ' + s1.upper()`), so the matrix has an
extra row/column for the empty-prefix case, same shape as classic
Levenshtein DP. Row/column 0 are initialized using `d_cost` between
*consecutive same-string* characters (not a flat per-character cost) —
this is the actual "editex" modification, distinct from plain Levenshtein
where deletion/insertion is always cost `1`.

Final result: `min(d_mat[len_s1][len_s2], max_length)` where
`max_length = max(len(s1), len(s2)) * mismatch_cost`.

## Inputs / options

- `local: bool = False` — when `True`, **row 0 is not initialized** (only
  column 0 is), meaning a leading unmatched prefix in `s1` costs nothing,
  while a leading unmatched prefix in `s2` still does. This asymmetry is
  in the source, not a bug — confirmed by `test_local`'s expected values
  differing from `test_distance`'s only on cases with an empty/short `s1`.
- `match_cost`, `group_cost`, `mismatch_cost` — see clamping above.
- `groups`, `ungrouped` — overridable, but if you pass `groups` you must
  also pass `ungrouped` (source raises `ValueError` otherwise). Default to
  the fixed data above.
- `external: bool = True` — no-op in the Rust core (no external path).
- No `qval` — Editex operates on raw characters, not q-grams.
- `maximum()` = `max(len(s1), len(s2)) * mismatch_cost`.

## Edge cases

- Empty/empty → `0`.
- One empty → `len(other) * mismatch_cost` in non-local mode (e.g.
  `'nelson'` vs `''` → `6 * 2 = 12`); **different** in local mode
  (`'nelson'` vs `''` → `12` too here, but `''` vs `'neilsen'` → `14`
  non-local vs `14` local — check the full table below, the two modes
  diverge specifically on which side is empty, due to the row/column
  asymmetry above).
- Source comment flags a genuine Unicode risk: uppercasing can change
  string length for certain glyphs (e.g. German `ß` → `SS`), which could
  push the raw DP distance above `max_length` if `max_length` were
  computed *after* uppercasing — that's why `max_length` is captured
  **before** the uppercase conversion, then used to cap the final result
  via `min(...)`. The Rust port must preserve this exact ordering (compute
  max first, then uppercase, then cap final result) — don't reorder these
  steps.

## Worked examples

From `tests/original/test_phonetic/test_editex.py`, non-local mode
(`assert actual == expected` — exact integer equality, not `isclose`):

| left | right | expected |
| --- | --- | --- |
| `''` | `''` | `0` |
| `nelson` | `''` | `12` |
| `''` | `neilsen` | `14` |
| `ab` | `a` | `2` |
| `ab` | `c` | `4` |
| `nelson` | `neilsen` | `2` |
| `neilsen` | `nelson` | `2` |
| `niall` | `neal` | `1` |
| `neal` | `niall` | `1` |
| `niall` | `nihal` | `2` |
| `nihal` | `niall` | `2` |
| `neal` | `nihl` | `3` |
| `nihl` | `neal` | `3` |
| `cat` | `hat` | `2` |
| `Niall` | `Neil` | `2` |
| `aluminum` | `Catalan` | `12` |
| `ATCG` | `TAGC` | `6` |

Local mode (`local=True`) — note the divergences from non-local on the
first two empty-string rows:

| left | right | expected (local) | expected (non-local) |
| --- | --- | --- | --- |
| `nelson` | `''` | `12` | `12` (same) |
| `''` | `neilsen` | `14` | `14` (same) |
| `ab` | `c` | `2` | `4` (**differs**) |

That's 17 non-local + 13 local = 30 examples total, far above the minimum —
this is the best-covered card of the eight.

## Numeric tolerance

Exact integer equality (`assert actual == expected`, not `isclose`) — this
algorithm produces whole-number costs, so the Rust port should too (no
floating-point tolerance question here at all).

## Dependencies / compressor settings

None — pure algorithm. Source has an optional `numpy` fast-path for the DP
matrix (`if numpy: ... else: defaultdict`), but that's a Python
performance detail with no Rust equivalent needed; a plain 2D array/Vec in
Rust covers both cases.

## Known risks

- The `local=True` row/column asymmetry is easy to get backwards — write a
  dedicated native test for `local=True` using the `'ab'`/`'c'` case
  (`2` local vs `4` non-local), since that's the clearest example of the
  divergence.
- The uppercase-before/after `max_length` capture ordering (see edge cases
  above) is a subtle but explicit correctness requirement from the source
  comment — a naive port that uppercases first and computes `max_length`
  second would be wrong for certain Unicode inputs, even though no fixed
  example currently exercises a length-changing uppercase case.
- `d_cost`'s asymmetry (`elements[0] in self.ungrouped` — checks only the
  *first* argument, not both) means `d_cost('H', 'X')` and `d_cost('X',
  'H')` can differ. Must preserve argument order exactly when porting, not
  treat the two characters as interchangeable.
