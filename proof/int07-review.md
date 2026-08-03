# INT-07 — Simha Teja core/FFI review

**Status: signed off on 2026-08-03 for the current `teja` working tree.**

## Review checks

| Check | Evidence | Result |
| --- | --- | --- |
| No Python fallback | `textdistance/_rust_adapter.py` loads `textdistance_port` and raises if it is unavailable; all exported algorithm modules delegate through it | PASS |
| No unsafe core logic | `rust/src/**` and `python_adapter/**` contain no `unsafe`; the only unsafe code is the counting allocator inside the benchmark harness | PASS |
| Public behavior | `make verify`: 400 non-external original tests, 114 corpus cases, and native/fuzz tests pass | PASS |
| Original snapshot preserved | `shasum -a 256 -c proof/original-tests.sha256` passes for every file; no `tests/original` diff | PASS |
| Adapter edge cases | Matrix float costs, custom alignment matrices, integer lists, bytes, q-values, and sequence output probes pass | PASS |
| External-provider compatibility | Full unchanged run reaches 427/430; three failures are RapidFuzz 3.14.5's large-integer list coercion, also inconsistent with the frozen pure-Python implementation | ENVIRONMENT CAVEAT |

## Review conclusion

The FFI boundary is thin and explicit: Python inputs/options are normalized,
Rust owns computation, and Rust results are reconstructed only where the
source API returns sequences. No original test was edited, no Python
algorithm implementation is used at runtime, and no public mismatch was
found in the supported package path. The three optional-provider failures are
recorded as an environment/library mismatch rather than copied into the Rust
algorithm semantics.
