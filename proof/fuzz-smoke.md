# INT-05 — Fuzz/Smoke Test (Suri's algorithm packets)

**Status: FAIL — 1 algorithm (MRA) panics under fuzzing. Not fixed (INT-05 is discovery-only).**

## Scope

This task's dependency (INT-04 differential fixtures) does not exist yet
(`proof/` has no differential corpus), and the repository's designated
`INT-05` predecessor tooling could not run natively:

- **`fuzzing/textdistance_fuzzer.py`** (the repo's existing atheris-based
  fuzzer) requires atheris, which only ships libFuzzer-instrumented builds for
  Linux and macOS. It cannot be installed or run on native Windows.
- **`cargo-fuzz`** requires nightly Rust plus sanitizer support, which has no
  usable Windows target either.
- No WSL distribution is installed on this machine (`wsl --list --verbose`
  reports none), so neither tool could be run through a Linux compatibility
  layer without first installing a distro — a disruptive environment change
  the user chose not to make for this task.

**Decision (confirmed with the user):** add a Windows-native Rust smoke-test
harness instead, under `tests/fuzz_smoke.rs` (auto-discovered by Cargo per the
project's direct-test-root convention; no edits to `Cargo.toml` or any shared
harness). It feeds a deterministic, seeded PRNG's randomized inputs plus the
PRD's fixed minimum-proof-corpus edge cases directly to the 10 assigned Rust
implementations, wrapping every call in `catch_unwind` so a panic in one
algorithm is recorded rather than aborting the run. No algorithm
implementation or existing test was modified.

## Command

```sh
cargo test --test fuzz_smoke -- --nocapture
```

## Configuration

| Item | Value |
| --- | --- |
| PRNG | xorshift64* (no external `rand` dependency; inline in the test file) |
| Seed | `0x5EED_F00D_1234_5678` (fixed, for reproducibility) |
| Random iterations | 2000 |
| Fixed edge cases | 17 (PRD §10 minimum proof corpus: empty/empty, empty/non-empty, equal, different, Unicode incl. combining marks and emoji, repeated character, qval 1/2/3/None, two/three sequences, integers, bytes, booleans) |
| Input kinds exercised | `Text`, `Integers`, `Bytes`, `Booleans` (0-3 sequences per case) |
| Algorithms exercised per case | Overlap, Tanimoto, RatcliffObershelp, BWTRLENCD, MRA, Prefix, Postfix, Length, Identity, Matrix |
| Methods called per algorithm | `call`, `similarity`, `distance`, `maximum`, `normalized_distance`, `normalized_similarity` (or `output`/`output_maximum`/`output_mode` + `output_distance`/`output_similarity` for Prefix/Postfix) |

## Result

```text
fuzz_smoke: seed=0x5eedf00d12345678 random_iterations=2000 edge_cases=17 prepared_cases_exercised=1583
1202 panic(s) found during fuzz smoke test
test suri_packets_survive_randomized_and_edge_case_inputs ... FAILED
```

| Metric | Value |
| --- | --- |
| Random cases generated | 2000 |
| Fixed edge cases | 17 |
| Cases where input preparation succeeded (actually exercised) | 1583 |
| Cases rejected by `prepare_sequences` (expected — e.g. `Words` on non-text input) | 434 |
| Panics recorded | 1202 (all in one algorithm) |
| Algorithms with zero panics | Overlap, Tanimoto, RatcliffObershelp, BWTRLENCD, Prefix, Postfix, Length, Identity, Matrix (9/10) |
| Algorithms with panics | MRA (1/10) |
| Infinite loops / hangs | None observed (full run completed in ~11s) |
| Memory errors | None observed (safe Rust; no `unsafe` in any of the 10 files) |

## Issue found — do not fix (INT-05 is discovery-only)

- **Algorithm:** MRA (`rust/src/algorithms/phonetic/mra.rs`)
- **Location:** `MRA::to_text`, line 23 — `_ => panic!("MRA requires character sequences")`
- **Failing input pattern:** any prepared sequence containing an `Element`
  that is not `Element::Char`. Concretely this fires whenever:
  1. the input `InputSequence` is `Integers`, `Bytes`, or `Booleans` (any
     `qval`), or
  2. the input is plain `Text` but `qval` is anything other than
     `QValue::Elements` — i.e. `QValue::NGrams(_)` (produces `Element::Gram`)
     or `QValue::Words` (produces `Element::Text`).
- **Smallest reproduction:** `MRA::new().call(&prepare_sequences(&[InputSequence::Text("test".into()), InputSequence::Text("text".into())], QValue::NGrams(2)).unwrap())` panics with `MRA requires character sequences`, even though both inputs are plain ASCII text and q-gram mode is a documented, commonly-exercised constructor option for the public API (PRD §4, item 4).
- **Representative captured messages** (full list of 1202 in the raw
  `cargo test -- --nocapture` output):
  ```text
  PANIC in MRA: input=qval=2 message=MRA requires character sequences
  PANIC in MRA: input=qval=3 message=MRA requires character sequences
  PANIC in MRA: input=qval=None word split message=MRA requires character sequences
  PANIC in MRA: input=integer sequences message=MRA requires character sequences
  PANIC in MRA: input=byte sequences message=MRA requires character sequences
  PANIC in MRA: input=boolean sequences message=MRA requires character sequences
  PANIC in MRA: input=[Text("😹< ]"), Bytes([94, 251, 229, 231, 164, 158, 217, 121, 159])] qval=Elements message=MRA requires character sequences
  ```
- **Owner:** Suri (MRA is packet `SUR-05`).
- **Why this matters for INT-05/INT-01:** a Rust `panic!` that crosses the
  planned PyO3 adapter boundary (SIM-10, not yet built) unwinds as a Rust
  abort, not a catchable Python exception — it would crash the Python
  process instead of raising `TypeError`/`ValueError` the way the original
  `textdistance.MRA` does for unsupported input. This is worth resolving
  before SIM-10/INT-01 wire MRA through the adapter, since qval={2,3,None}
  are default, expected configurations, not just malformed input.
- **Not fixed.** Per this task's instructions, no algorithm implementation
  was modified. Fixing this is out of scope for INT-05 and is deferred to
  INT-03 (owner-scoped fixes) or a dedicated MRA follow-up commit.

## Compliance for this task

- No algorithm implementation modified.
- No existing test modified.
- One new file added: `tests/fuzz_smoke.rs` (auto-discovered by Cargo; no
  `Cargo.toml`/shared-harness edit required).
- No Python/PyO3 boundary involved — this exercises the Rust core directly,
  consistent with `docs/API_CONTRACT.md`'s "Rust core logic must not import
  Python" rule.
