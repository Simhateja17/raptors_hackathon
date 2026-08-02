# Known issues found while building the INT-04 proof corpus

Both issues below were discovered by cross-checking fresh Rust-backed output
against already-verified reference values while building
[`corpus.md`](corpus.md). Neither is fixed here — INT-04 is scoped to
building the fixture corpus, not to patching the adapter — but both are
reproducible and routed around in the frozen fixtures, so they should not
silently resurface as "passing" evidence.

Both live in `python_adapter/src/lib.rs` (the SIM-10 PyO3 adapter), not in
any Rust algorithm file. No algorithm implementation was touched to identify
or document these.

## 1. `Prefix`/`Postfix` sequence output is wrong for non-default `qval`

Reproduction:

```pycon
>>> import textdistance as td
>>> td.Prefix(qval=None)('one two three', 'one two four')
'onetwo'          # wrong: original Python returns ['one', 'two']
>>> td.Prefix(qval=2)('testing', 'tester')
ValueError: cannot reconstruct a str result from element Gram([Char('t'), Char('e')])
```

**Cause:** `compute()`'s `shape` (used to decide whether to reconstruct the
Rust `Sequence` output as a Python `str`, `bytes`, or `list`) is derived from
the *raw* Python argument's type before `qval` preparation. The original
Python source instead branches on the type of the *already-prepared*
sequence (`sequences[0]` after `_get_sequences`), which is a `list` of word
tokens when `qval=None`, and a `list` of q-gram tuples when `qval>1` — never
a `str` in either case. Only `qval=1` (the default) happens to leave the
prepared representation as something `str`-reconstructible.

**Effect:** only the sequence-returning `call()` path is affected.
`similarity()`/`distance()`/`normalized_*` are scalar and unaffected — they
were verified working correctly for all `qval` values in the corpus
(`prefix.json`'s `qvalue-ngram2-similarity-only` case).

**Corpus workaround:** `prefix.json`/`postfix.json` only exercise `call()` at
the default `qval=1`. The one non-default-`qval` case
(`qvalue-ngram2-similarity-only`) tests `similarity()` only, with a `note`
explaining the omission.

## 2. A `list` of small integers can be misclassified as `bytes`

Reproduction:

```pycon
>>> import textdistance as td
>>> td.Prefix()([1, 2, 3, 4], [1, 2, 5, 6])
b'\x01\x02'       # wrong: original Python returns [1, 2]
```

**Cause:** `convert_sequence()` tries `obj.extract::<Vec<u8>>()` before
checking whether `obj` is an actual `bytes`/`bytearray` object. PyO3's
generic `Vec<u8>` extraction succeeds for *any* Python sequence whose
elements all fit in `0..=255` — including a plain `list[int]` — so a list
like `[1, 2, 3, 4]` is silently treated as raw bytes instead of a list of
`Element::Integer`.

**Effect:** not just display — this can affect the *computed* result. If one
compared sequence has every value `<=255` (coerced to `Element::Byte`) while
another has any value `>255` (correctly `Element::Integer`), elements that
are semantically equal (`1` vs `1`) compare as different `Element` variants
and fail equality checks.

**Corpus workaround:** every `integers` category case in this corpus uses at
least one value `>255` (e.g. `1000`) specifically to force correct
`Vec<u8>`-extraction failure and land on the `Elements`/`Integer` path,
sidestepping the bug rather than freezing it as expected behavior.

## Suggested fix (not applied here)

In `convert_sequence`, check `obj.downcast::<PyByteArray>().is_ok()` /
confirm `obj.downcast::<PyBytes>()` instead of relying on `Vec<u8>`
extraction for the "is this bytes?" test; only fall through to per-element
homogeneous-type detection (bool/int/str) for anything that isn't an actual
`bytes`-like object. For the `Prefix`/`Postfix` reconstruction issue, `shape`
would need to be derived from the *prepared* sequence's element type (or
from the source's `_get_sequences` output shape) rather than from the raw
argument.
