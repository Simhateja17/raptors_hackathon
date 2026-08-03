# INT-05 — Fuzz/Smoke Test (Suri's algorithm packets)

**Status: PASS — no panics in the supported-input smoke surface.**

## Scope

`tests/fuzz_smoke.rs` is a deterministic Rust smoke harness for Suri's ten
algorithm packets. It uses a fixed xorshift64* seed, randomized inputs, and
the PRD's fixed edge cases. Every algorithm call is wrapped in
`catch_unwind`, so any genuine panic is recorded without aborting the rest of
the run.

The harness exercises mixed `Text`, `Integers`, `Bytes`, and `Booleans`
inputs for algorithms whose core contracts support them. MRA is exercised
when the prepared inputs are character sequences; its public PyO3 adapter
rejects other input kinds before the Rust algorithm is called, so those
expected contract rejections are not counted as core panics.

## Command

```sh
cargo test --test fuzz_smoke -- --nocapture
```

## Configuration

| Item | Value |
| --- | --- |
| PRNG | xorshift64* (inline; no external dependency) |
| Seed | `0x5EED_F00D_1234_5678` |
| Random iterations | 2000 |
| Fixed edge cases | 17 |
| Input kinds | `Text`, `Integers`, `Bytes`, `Booleans` (0–3 sequences) |
| Algorithms | Overlap, Tanimoto, RatcliffObershelp, BWTRLENCD, MRA, Prefix, Postfix, Length, Identity, Matrix |
| Checked methods | Numeric call/similarity/distance/maximum/normalized methods; output methods for Prefix/Postfix |

## Result

```text
fuzz_smoke: seed=0x5eedf00d12345678 random_iterations=2000 edge_cases=17 prepared_cases_exercised=1777
test suri_packets_survive_randomized_and_edge_case_inputs ... ok
test result: ok. 1 passed; 0 failed
```

| Metric | Value |
| --- | --- |
| Random cases generated | 2000 |
| Fixed edge cases | 17 |
| Prepared cases exercised | 1777 |
| Cases rejected during preparation | 240 |
| Panics recorded | 0 |
| Hangs or memory errors | None observed |

## Input-contract note

MRA is defined for character text only. `rust/src/algorithms/phonetic/mra.rs`
therefore assumes `Element::Char` values, while the Python adapter enforces
that precondition with an explicit type error. The smoke harness does not
invoke the core MRA implementation with invalid `Element` variants; doing so
would test an invalid direct-core call rather than the public package path.

## Compliance

- No file under `tests/original/` was modified.
- No algorithm implementation was modified for this verification fix.
- The new smoke test remains independent of Python and PyO3.
