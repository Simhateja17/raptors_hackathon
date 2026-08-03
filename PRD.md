# TextDistance Rust Port — Hackathon PRD

## 1. Project decision

### Chosen track

Track D: Python → Rust

### Chosen repository

`life4/textdistance`, currently checked out in this repository as a public fork.

### Product statement

Port the public, pure-Python TextDistance implementation to a self-contained Rust core while preserving its observable behavior through the unchanged original test suite, a thin compatibility adapter, differential testing, fuzzing, and benchmarks.

The submission is judged on proof that the port holds up—not on how many lines are mechanically translated. The winning story is therefore:

> “The same real library, same public algorithms, same edge cases, independently implemented in Rust, with the original tests left untouched and backed by differential evidence.”

## 2. Rules we must not violate

- All port code must be authored after the official kickoff: **Jul 31, 18:00 UTC**.
- Submission deadline: **Aug 3, 18:00 UTC**.
- Keep the original tests unchanged and record their SHA-256 manifest immediately.
- Do not call or embed the original Python implementation at runtime.
- A thin Python/PyO3 adapter is allowed only as the test boundary; the algorithm logic must execute in Rust.
- Keep the first port commit after kickoff and make incremental commits. Do not squash the entire port into one dump.
- The repository must build and test with one documented command.
- Do not add a web UI, server, unrelated algorithms, or a redesign of TextDistance’s behavior.

## 3. Baseline facts

- The `textdistance/` implementation contains approximately 2,657 Python source lines, excluding tests and fuzzing code.
- It has 29 test files, including Hypothesis property-based tests.
- It has an existing fuzzing directory.
- The implementation exposes more than 30 public algorithm classes and singleton functions.
- `vector_based.py` is marked as draft and is not imported by `textdistance/algorithms/__init__.py`; it is not part of the public port scope.
- The baseline reports version `4.6.3` in `setup.py` but `4.6.2` in `textdistance/__init__.py`; record this as a compatibility observation rather than silently changing it.
- Optional Python acceleration libraries are not part of the Rust core. `external=True` remains accepted for API compatibility, but the Rust implementation must remain correct without them.

## 4. Scope

### Must ship

1. A standalone Rust crate containing the algorithm implementation.
1. Public classes and singleton functions from the exported modules:

   - Edit: Hamming, Levenshtein, Damerau-Levenshtein, Jaro, Jaro-Winkler, StrCmp95, Needleman-Wunsch, Gotoh, Smith-Waterman, MLIPNS.
   - Token: Jaccard, Sørensen/Dice, Tversky, Overlap, Cosine, Tanimoto, Monge-Elkan, Bag.
   - Sequence: LCS sequence, LCS substring, Ratcliff-Obershelp.
   - Compression: Arithmetic NCD, RLE NCD, BWT-RLE NCD, square-root NCD, entropy NCD, BZ2 NCD, LZMA NCD, ZLIB NCD.
   - Phonetic: MRA and Editex.
   - Simple: Prefix, Postfix, Length, Identity, Matrix.

1. Common methods where applicable: `distance`, `similarity`, `maximum`, `normalized_distance`, and `normalized_similarity`.
1. Constructor options used by the public API, especially `qval`, `as_set`, edit costs, and algorithm-specific options.
1. Correct behavior for empty inputs, equal inputs, Unicode strings, q-grams, word splitting, and integer sequences covered by the original tests.
1. An unchanged copy of the complete original `tests/` tree under `tests/original/` plus its manifest.
1. An adapter that lets the unchanged original tests exercise the Rust implementation.
1. Differential fixtures/harness, fuzzing, benchmark report, `README.md`, `DECISIONS.md`, and a five-minute demo path.

### Explicitly out of scope

- The unexported draft `vector_based.py` module.
- Recreating Python’s optional third-party acceleration selection in Rust.
- Calling the original Python implementation from the shipped runtime.
- Modifying, reformatting, or “fixing” original tests.
- New product features or algorithm changes that alter baseline behavior.

## 5. Technical architecture

```text
unchanged tests/original/*.py
            │
            ▼
thin Python/PyO3 compatibility adapter
            │  FFI boundary only
            ▼
standalone Rust TextDistance core
            │
            ├── common sequence preparation and normalization
            ├── edit algorithms
            ├── token and sequence algorithms
            ├── compression and phonetic algorithms
            └── native Rust tests, fuzz target, and benchmarks
```

### Required design decisions

- Each public algorithm is implemented in one dedicated Rust file and tested in one dedicated native test file; this is what makes four-person parallel work safe.
- Every assigned algorithm path is compile-visible from the scaffold. Simha Teja owns the shared registry, while each owner replaces only the placeholder in their assigned file; a registry test must import every packet.
- Native algorithm tests live as unique files directly under the package-root `tests/` directory (for example, `tests/algorithm_jaccard.rs`). Cargo auto-discovers these direct integration-test roots; owners must not create nested test files or edit a shared harness.
- Rust core logic must not import Python or depend on Python objects.
- Python `str` must be handled as Unicode scalar values, matching Python’s code-point length behavior rather than byte length.
- The adapter must cover strings, bytes, and integer sequences used by the original suite.
- Unsupported arbitrary Python objects must fail clearly and deterministically; do not silently coerce them.
- `qval=None` must perform word splitting; `qval=1` must preserve elements; `qval>1` must create n-grams.
- All algorithms must use the shared Rust preparation and common-method contract rather than duplicating normalization logic.
- Floating-point comparisons in proof tooling must use the original suite’s tolerance rules where the source behavior is numeric rather than exact.
- Compression algorithms must report the same distance semantics, with any unavoidable compressor-version caveat documented in `DECISIONS.md` and tested against fixed fixtures.

## 6. Four-person ownership model

All four members are AI-assisted implementation owners. The human responsibility is to understand the source behavior, give the AI a narrow task, inspect the diff, run the acceptance checks, and explain the result. Nobody is expected to design or type the entire port manually.

To prevent file conflicts, every public algorithm gets its own Rust source file and its own native test file. The shared module registry is owned by Simha Teja; he seeds the declarations and placeholders, and algorithm owners replace only their assigned implementation/test files. Native tests use unique direct Rust files under the package-root `tests/` directory so Cargo discovers them without a shared harness. Existing Python tests and `tests/original/` are not edited.

### Simha Teja — architecture, highest-risk algorithms, and FFI

**Owns shared files:**

```text
Cargo.toml
rust/src/lib.rs
rust/src/core/**
rust/src/algorithms/mod.rs
rust/tests/common.rs
python_adapter/**
docs/API_CONTRACT.md
```

**Owns these algorithm files and matching tests:**

```text
rust/src/algorithms/edit/levenshtein.rs
rust/src/algorithms/edit/damerau_levenshtein.rs
rust/src/algorithms/edit/needleman_wunsch.rs
rust/src/algorithms/edit/smith_waterman.rs
rust/src/algorithms/edit/gotoh.rs
rust/src/algorithms/edit/strcmp95.rs
rust/src/algorithms/edit/mlipns.rs
rust/src/algorithms/compression/arith_ncd.rs
rust/src/algorithms/sequence/lcsseq.rs
tests/algorithm_levenshtein.rs
tests/algorithm_damerau_levenshtein.rs
tests/algorithm_needleman_wunsch.rs
tests/algorithm_smith_waterman.rs
tests/algorithm_gotoh.rs
tests/algorithm_strcmp95.rs
tests/algorithm_mlipns.rs
tests/algorithm_arith_ncd.rs
tests/algorithm_lcsseq.rs
```

**Thinking tasks:** freeze the common API, decide the Rust sequence representation, resolve Unicode and numeric semantics, and review all cross-cutting changes.

**Done when:** the crate compiles without Python, the shared contract is frozen, all assigned algorithms have native tests, and the PyO3 adapter exposes the complete public surface required by the original tests.

### Manasa — compression, phonetic, and selected edit algorithms

**Owns these algorithm files and matching tests:**

```text
rust/src/algorithms/edit/jaro.rs
rust/src/algorithms/edit/jaro_winkler.rs
rust/src/algorithms/phonetic/editex.rs
rust/src/algorithms/compression/sqrt_ncd.rs
rust/src/algorithms/compression/entropy_ncd.rs
rust/src/algorithms/compression/bz2_ncd.rs
rust/src/algorithms/compression/lzma_ncd.rs
rust/src/algorithms/compression/zlib_ncd.rs
tests/algorithm_jaro.rs
tests/algorithm_jaro_winkler.rs
tests/algorithm_editex.rs
tests/algorithm_sqrt_ncd.rs
tests/algorithm_entropy_ncd.rs
tests/algorithm_bz2_ncd.rs
tests/algorithm_lzma_ncd.rs
tests/algorithm_zlib_ncd.rs
```

**Thinking tasks:** document compressor settings and numerical tolerance, compare Rust crate behavior with fixed baseline fixtures, and identify any dependency or output-format risk before integration.

**Dependency rule:** Manasa never edits `Cargo.toml`; dependency requirements go in `docs/dependency-notes/manasa.md`, including candidate crates, feature flags, system-library requirements, licenses, and compressor settings. Simha Teja reviews that note, changes `Cargo.toml`/`Cargo.lock` once, and runs the dependency smoke gate. Only BZ2, LZMA, and ZLIB implementation packets wait on that gate; Manasa’s other packets and all behavior-card work remain parallel-safe.

**Done when:** every assigned algorithm has a focused implementation, fixed expected-value tests, and a written compatibility note for any compressor-specific behavior.

### Poojitha — token, sequence, and straightforward compression algorithms

**Owns these algorithm files and matching tests:**

```text
rust/src/algorithms/edit/hamming.rs
rust/src/algorithms/token/jaccard.rs
rust/src/algorithms/token/sorensen.rs
rust/src/algorithms/token/tversky.rs
rust/src/algorithms/token/cosine.rs
rust/src/algorithms/token/monge_elkan.rs
rust/src/algorithms/token/bag.rs
rust/src/algorithms/sequence/lcsstr.rs
rust/src/algorithms/compression/rle_ncd.rs
tests/algorithm_hamming.rs
tests/algorithm_jaccard.rs
tests/algorithm_sorensen.rs
tests/algorithm_tversky.rs
tests/algorithm_cosine.rs
tests/algorithm_monge_elkan.rs
tests/algorithm_bag.rs
tests/algorithm_lcsstr.rs
tests/algorithm_rle_ncd.rs
```

**Thinking tasks:** translate set/multiset definitions into explicit examples, cover q-grams and repeated tokens, and verify returned subsequences and tie-breaking against the source tests.

**Done when:** every assigned algorithm has native tests for normal, empty, equal, repeated-token, and q-gram cases, with no edits outside the owned files.

### Suri — simple, phonetic, sequence, and token algorithms plus proof/release

**Owns these algorithm files and matching tests:**

```text
rust/src/algorithms/token/overlap.rs
rust/src/algorithms/token/tanimoto.rs
rust/src/algorithms/sequence/ratcliff_obershelp.rs
rust/src/algorithms/compression/bwtrle_ncd.rs
rust/src/algorithms/phonetic/mra.rs
rust/src/algorithms/simple/prefix.rs
rust/src/algorithms/simple/postfix.rs
rust/src/algorithms/simple/length.rs
rust/src/algorithms/simple/identity.rs
rust/src/algorithms/simple/matrix.rs
tests/algorithm_overlap.rs
tests/algorithm_tanimoto.rs
tests/algorithm_ratcliff_obershelp.rs
tests/algorithm_bwtrle_ncd.rs
tests/algorithm_mra.rs
tests/algorithm_prefix.rs
tests/algorithm_postfix.rs
tests/algorithm_length.rs
tests/algorithm_identity.rs
tests/algorithm_matrix.rs
```

**Also owns:**

```text
textdistance/**
tests/original/**
proof/**
fuzzing/**
bench/**
docs/DECISIONS.md
docs/DEMO.md
README.md
Makefile
.github/**
```

**Thinking tasks:** preserve the baseline, build the fixed differential corpus, run proof commands, document decisions, and rehearse the demo. The assigned algorithms must still be completed before integration; proof work is not a substitute for implementation.

**Done when:** all assigned algorithm tests pass, the original test tree is hash-verified, the proof commands are reproducible, and the submission package is complete.

### Workload balance

The algorithm count is intentionally not identical because the algorithms have different risk. The target is balanced effort, not equal line count.

| Owner | Algorithm packets | Shared responsibility | Workload shape |
| --- | ---: | --- | --- |
| Simha Teja | 9 | Core, API, FFI, integration fixes | Fewer packets, highest algorithm and integration risk |
| Manasa | 8 | Compression dependencies and numeric fixtures | Fewer packets, higher compatibility risk |
| Poojitha | 9 | Token/sequence invariants and q-gram examples | More bounded packets, medium algorithm risk |
| Suri | 10 | Baseline, proof, benchmark, docs, release | More bounded packets plus the verification path |

Every member must produce Rust code, native tests, a handoff note, and a demo explanation. No member is assigned a documentation-only role.

## 7. AI-assisted work protocol

The team’s job is to direct and verify the AI, not to accept generated code blindly.

For every algorithm, the owner follows this loop:

1. Read the corresponding Python source and original tests.
1. Write a short behavior card: inputs, outputs, edge cases, options, and 3–5 expected examples.
1. Ask the AI to edit only the named algorithm file and its matching native test file.
1. Require the AI to explain each non-obvious translation and identify any source-language semantic risk.
1. Run the focused test command.
1. Inspect `git diff`, reject unrelated files, and ask the AI for a review of the diff.
1. Commit one algorithm or one tightly coupled pair at a time.
1. Record failures with the exact command and output; do not patch around a failing assertion without understanding it.

### Standard implementation prompt

Each owner can use this template with their coding agent:

```text
Implement only <ALGORITHM> from textdistance/algorithms/<SOURCE_MODULE>.py.

Target file: rust/src/algorithms/<TARGET_FILE>.rs
Test file: tests/algorithm_<TARGET_TEST>.rs

Read the original source and its tests. Preserve observable behavior, including
empty inputs, equal inputs, Unicode/code-point semantics, qval behavior, numeric
options, and failure behavior. Use the shared Rust API from docs/API_CONTRACT.md.
Do not edit Cargo.toml, lib.rs, algorithms/mod.rs, core, Python files, other
algorithm files, or unrelated tests. Do not call Python or any original runtime.
Add focused native Rust tests using fixed examples from the original tests.
Before finishing, show the diff, list assumptions, and run the smallest relevant
cargo test command.
```

### Human review checklist

The owner must be able to answer yes to all of these before handoff:

- Does the algorithm match the source’s definition, not merely its name?
- Are empty and equal inputs covered?
- Are Unicode strings compared by code points rather than UTF-8 bytes?
- Are constructor options and aliases covered where applicable?
- Does the test fail if the implementation is replaced with a trivial constant?
- Did the AI touch only the assigned paths?
- Is any unresolved assumption written in the handoff message?

### AI safety rules

- Never ask the AI to rewrite the entire repository in one prompt.
- Never accept a generated dependency or API change without Simha Teja’s review.
- Never copy a Python implementation into Rust mechanically without checking indexing, integer overflow, floating-point, and Unicode behavior.
- Never use the original Python implementation as a runtime fallback.
- AI-generated commits are still owned by the named human; the human must understand and defend them in the demo.

## 8. File ownership and merge policy

The following rule is mandatory: **one file, one owner**.

- Members work on separate branches.
- Simha Teja creates the initial crate, owns the FFI layer, and freezes the core contract.
- Poojitha, Manasa, and Suri create only their individually assigned algorithm/test files after `API-FREEZE`.
- Suri starts baseline preservation immediately, then branches from `API-FREEZE` for proof tooling and release assembly.
- Only Simha Teja edits `Cargo.toml`, `rust/src/lib.rs`, `rust/src/core/**`, `rust/src/algorithms/mod.rs`, `rust/tests/common.rs`, and `python_adapter/**`.
- Only Suri edits root packaging, README, CI, proof, `docs/DECISIONS.md`, `docs/DEMO.md`, and original-test paths; Simha Teja owns `docs/API_CONTRACT.md`, and Manasa owns her dependency note.
- Algorithm owners may not edit another owner’s algorithm file, even to fix a failing integration test. They submit an issue or handoff note to the owner.
- No one edits another member’s tests to make a failure disappear. A failing test is reported to the owner.
- Integration is done by cherry-picking completed commits in dependency order; do not merge half-finished branches.

## 9. Sequential execution plan

### Gate G0 — Baseline freeze

**Owner:** Suri, with all members observing.

1. Verify the working tree and record the current baseline commit.
1. Copy the complete original `tests/` tree byte-for-byte to `tests/original/`.
1. Generate a sorted SHA-256 manifest.
1. Record Python version, dependency versions, and baseline test command/result.
1. Tag the repository state `BASELINE-FROZEN`.

No one edits the copied tests after this gate.

### Gate G1 — Core/API freeze

**Owner:** Simha Teja.

1. Create the Rust crate and common representation.
1. Define algorithm trait/struct conventions and adapter names.
1. Define module declarations and the adapter-facing contract; Simha Teja seeds all compile-visible module paths and placeholders, while algorithm owners replace placeholders only inside their own module files.
1. Document supported input representations and error behavior.
1. Tag `API-FREEZE`.

Poojitha, Manasa, and Suri do not begin algorithm implementation until this gate exists. They may prepare behavior cards and fixtures before it; after the registry sub-gate, their assigned files can proceed independently.

### Gate G2 — Parallel algorithm-packet implementation

**Owners:** all four members in their exclusive files.

- Simha Teja completes the core, nine assigned high-risk algorithm packets, and FFI layer.
- Poojitha completes nine assigned algorithm packets.
- Manasa completes eight assigned algorithm packets and dependency notes.
- Suri completes ten assigned algorithm packets while building proof tooling in parallel.

Each algorithm owner must provide native tests before handing off.

### Gate G3 — Adapter and original-suite integration

**Owner:** Suri; FFI/API fixes by Simha Teja only.

1. Build the Rust extension and thin adapter.
1. Run unchanged original tests through the adapter.
1. Categorize every failure as adapter, algorithm, numeric tolerance, missing dependency, or expected unsupported input.
1. Route each failure to exactly one owner.
1. Do not move to fuzzing until the full non-external original suite passes.

Target: all unchanged original tests pass, including `test_external.py` when the documented optional test dependencies are installed. The non-external suite is the first integration gate; any environment-only external failure must include the exact dependency and command needed to reproduce it.

### Gate G4 — Proof freeze

**Owner:** Suri, with fixes by the relevant algorithm owner.

- Differential corpus passes.
- Fuzz target runs for a fixed time budget with zero crashes and no unexplained mismatches.
- Native tests and unchanged original tests pass.
- Benchmark results are captured on the same machine and command.
- No new feature work is accepted after this gate.

### Gate G5 — Submission freeze

**Owner:** Suri.

- Public GitHub repository is reachable.
- README has setup, one-command build/test, architecture, and results.
- `DECISIONS.md` explains language choice, API boundary, unsupported behavior, and compressor choices.
- `DEMO.md` has a rehearsed five-minute path.
- `git log` shows incremental post-kickoff port commits.
- Test manifest matches the unchanged originals.
- Submission form fields are filled: repo, track D, demo link, benchmark report, and proof commands.

## 10. Verification plan

### Evidence ladder

1. **Compile evidence:** clean Rust build from a fresh checkout.
1. **Unit evidence:** native Rust tests for every algorithm packet.
1. **Original-suite evidence:** unchanged Python tests through the adapter, with SHA-256 manifest.
1. **Differential evidence:** fixed corpus comparing baseline outputs and port outputs over strings, Unicode, empty values, q-grams, and integer sequences.
1. **Fuzz evidence:** randomized inputs exercising all algorithms and common methods, with saved seed and duration.
1. **Performance evidence:** baseline versus Rust benchmark on identical cases; report both speed and any memory trade-off.
1. **Reproducibility evidence:** one command that builds, tests, runs proof checks, and exits nonzero on failure.

### Minimum proof corpus

The corpus must include:

- empty/empty, empty/non-empty, equal, and completely different inputs;
- ASCII and non-ASCII Unicode, including combining characters and emoji;
- one-character and repeated-character strings;
- q-values `None`, `1`, `2`, and `3`;
- two and three-sequence calls where supported;
- lists of integers;
- bytes for binary compression algorithms;
- constructor options for restricted/unrestricted Damerau-Levenshtein, Jaro-Winkler, Editex, Matrix, and alignment algorithms;
- known expected values copied from the baseline run.

## 11. Commands to standardize

Suri owns the final command names, but the intended interface is:

```bash
make build       # clean Rust/adapter build
make test        # native Rust tests plus unchanged original tests
make verify      # manifest, differential corpus, and fuzz smoke run
make benchmark   # reproducible benchmark report
make demo        # deterministic five-minute demo flow
```

The README must also show the underlying commands so judges can audit the wrapper.

## 12. Risk register and mitigation

| Risk | Impact | Mitigation | Owner |
| --- | --- | --- | --- |
| PyO3 adapter becomes the bottleneck | Original tests cannot run | Freeze a small adapter contract at G1; implement only public methods used by tests; use native tests in parallel | Simha Teja/Suri |
| Generic Python sequences are difficult to represent in pure Rust | Hidden compatibility failures | Explicitly support strings, bytes, and integer sequences first; reject unsupported objects clearly; document the boundary | Simha Teja |
| Unicode byte/code-point mismatch | Many subtle failures | Convert Python strings to Rust `char` sequences and add Unicode fixtures before family work | Simha Teja |
| Compression libraries produce different byte lengths | NCD mismatches | Lock crate versions, compare fixed fixtures, and document compressor settings | Manasa |
| Three algorithm families finish late | Incomplete submission | Cut draft/unexported work first; prioritize all public algorithms and their original tests; stop feature work at G4 | Simha Teja/Poojitha/Manasa |
| Original tests are accidentally changed | Disqualification/score loss | Immutable `tests/original/`, manifest check in `make verify`, Suri owns all original-test paths | Suri |
| Git history looks like a single generated dump | Eligibility concern | Small incremental commits by algorithm packet, each after kickoff, with meaningful messages | All |
| Toolchain/dependency installation fails | Lost integration time | Install Rust and cache dependencies immediately; keep a tested minimal dependency set | Simha Teja/Suri |

## 13. Time budget

Use the official kickoff as `H0`; the deadline is `H+72`.

| Window | Outcome |
| --- | --- |
| H0–H+2 | G0 baseline freeze, test hash, toolchain, branches, API decisions |
| H+2–H+8 | G1 core/API/FFI freeze; Suri proof shell |
| H+8–H+34 | Parallel algorithm implementation and native tests |
| H+34–H+48 | Merge algorithm packets; adapter integration; original-suite pass |
| H+48–H+58 | Differential corpus, fuzzing, benchmark, failure fixes |
| H+58–H+66 | README, decisions, demo rehearsal, clean-checkout verification |
| H+66–H+72 | Buffer, final review, submission; no new features |

If the team is already behind, cut in this order: optional external acceleration, draft vector module, extra benchmark cases, demo polish. Do not cut original-test preservation, differential proof, or reproducible build evidence.

## 14. Definition of done

The project is ready to submit only when all are true:

- The selected track is declared as Python → Rust.
- The port is a real Rust implementation, not a wrapper around Python.
- The original test files are unchanged and hash-verified.
- The adapter runs the original suite against Rust.
- All public exported algorithm families are implemented or an explicit, evidenced exception is documented before submission.
- Differential and fuzz proof commands pass.
- A benchmark report exists with reproducible commands.
- The repository builds with one command from a clean checkout.
- `README.md`, `DECISIONS.md`, and the five-minute demo are complete.
- Git history shows incremental work after kickoff.

## 15. Live execution task board

This section is the team’s source of truth while building. The checkbox is
completed only when the listed evidence exists. A task may not be started when
one of its dependencies is incomplete, except where the task is explicitly
marked as parallel-safe.

### How to use this board

For every task, the owner must update the checkbox and add the commit hash or
evidence path in the handoff message. AI-generated code does not count as done
until the human owner has inspected the diff and run the acceptance command.

The atomic unit of work is one task packet:

```text
behavior card → narrow AI prompt → owned-file diff → focused test → human review → commit
```

Do not ask an AI agent to implement an entire family or the whole repository.

### Lane 0 — G0 baseline freeze

These tasks are sequential and block port integration.

- [x] **G0-01 — Suri — snapshot original tests**
  - Dependency: none.
  - Output: complete byte-for-byte copy under `tests/original/`.
  - Acceptance: 35 files exist and no source test was edited.
- [x] **G0-02 — Suri — create the test manifest**
  - Dependency: G0-01.
  - Output: `proof/original-tests.sha256`.
  - Acceptance: `shasum -a 256 -c proof/original-tests.sha256` passes for all 35 files.
- [x] **G0-03 — Suri — record baseline metadata**
  - Dependency: G0-01.
  - Output: `proof/baseline.md` with commit, Python version, test command, and environment result.
  - Acceptance: the missing `pytest` dependency is recorded honestly as setup work, not reported as a passing test.
- [x] **G0-04 — Suri — install the test environment and run the original suite**
  - Dependency: G0-03.
  - Output: baseline test result appended to `proof/baseline.md`.
  - Acceptance: the isolated baseline run recorded 400 passing non-external tests and 30 optional external failures caused by missing comparison packages.
- [x] **G0-05 — all — acknowledge the freeze**
  - Dependency: G0-02 and G0-04.
  - Output: a `BASELINE-FROZEN` tag or commit note.
  - Acceptance: nobody changes `tests/original/**` afterward.

### Lane 1 — G1 shared Rust/API foundation

G1 is sequential only at the shared-contract level. The behavior-card
preparation tasks in Lane 2 are parallel-safe, and algorithm implementation
starts after the shared contract extensions and test-layout gate G1-10. No teammate needs to edit the
shared registry to begin an assigned packet.

- [x] **G1-01 — Simha Teja — create the standalone crate**
  - Dependency: G0-02.
  - Output: `Cargo.toml`, `rust/src/lib.rs`.
  - Acceptance: crate metadata declares a Python-independent Rust library.
- [x] **G1-02 — Simha Teja — define the input model**
  - Dependency: G1-01.
  - Output: `Element`, `InputSequence`, `Sequence`, `QValue`, and input errors in `rust/src/core/mod.rs`.
  - Acceptance: strings use Unicode scalar values; bytes, integers, and booleans have explicit representations.
- [x] **G1-03 — Simha Teja — define preparation semantics**
  - Dependency: G1-02.
  - Output: q-value, word-splitting, n-gram, identity, maximum, and normalization helpers.
  - Acceptance: focused contract tests cover empty values, Unicode, words, n-grams, identity, and normalization.
- [x] **G1-04 — Simha Teja — define the common algorithm trait**
  - Dependency: G1-03.
  - Output: `Algorithm` and `ScoreMode` contract.
  - Acceptance: distance-native and similarity-native algorithms can share common method conversion without Python.
- [x] **G1-05 — Simha Teja — reserve the module registry**
  - Dependency: G1-01.
  - Output: `rust/src/algorithms/mod.rs` with compile-visible declarations, isolated family namespaces, and replaceable packet paths.
  - Acceptance: later owners replace their assigned file without editing this shared registry themselves.
- [x] **G1-06 — Simha Teja — write the API contract**
  - Dependency: G1-02 through G1-05.
  - Output: `docs/API_CONTRACT.md`.
  - Acceptance: all owners can determine input types, q-value behavior, common methods, error rules, and file ownership without guessing.
- [x] **G1-07 — Simha Teja — compile and freeze the contract**
  - Dependency: G1-01 through G1-06 and Rust installed.
  - Output: successful `cargo fmt --check` and `cargo test`; `API-FREEZE` tag/commit.
  - Acceptance: `cargo fmt --check` and `cargo test` pass; the shared crate compiles independently of Python and the core contract tests pass.
- [x] **G1-08 — Simha Teja — make the full registry compile-visible**
  - Dependency: G1-07.
  - Output: all 36 assigned module declarations, replaceable placeholders, and `rust/tests/registry.rs`.
  - Acceptance: `cargo fmt --check && cargo test` passes; the registry test imports every assigned algorithm path. Owners replace placeholders without editing `rust/src/algorithms/mod.rs`.
- [x] **G1-09 — Simha Teja — add the output/error/comparator interface**
  - Dependency: G1-08.
  - Output: `AlgorithmOutput`, `AlgorithmError`, `OutputAlgorithm`, output conversion helpers, and `SimilarityComparator` in `rust/src/core/mod.rs`.
  - Acceptance: numeric algorithms remain source-compatible; sequence-producing algorithms can return their prepared sequence; delegated algorithms can accept a built-in Rust comparator without Python callbacks.
- [x] **G1-10 — Simha Teja — standardize native test discovery**
  - Dependency: G1-09.
  - Output: direct package-root `tests/algorithm_<name>.rs` ownership convention documented in the PRD and API contract.
  - Acceptance: an owner can add one native test file without editing `Cargo.toml` or a shared test harness.

### Lane 2 — Parallel behavior-card preparation

These are safe while G1-10 is pending because they create understanding and
fixtures, not shared Rust changes.

- [x] **PREP-01 — Simha Teja — behavior cards for high-risk algorithms**
  - Output: source/test notes for Levenshtein, Damerau-Levenshtein, Needleman-Wunsch, Smith-Waterman, Gotoh, StrCmp95, MLIPNS, Arithmetic NCD, and LCS sequence.
  - Acceptance: each note lists inputs, options, empty/equal behavior, numeric expectations, and source test references.
  - Evidence: `docs/behavior-cards/simha.md`.
- [x] **PREP-02 — Manasa — compression and phonetic compatibility cards**
  - Output: notes for Jaro, Jaro-Winkler, Editex, Sqrt NCD, Entropy NCD, BZ2 NCD, LZMA NCD, and ZLIB NCD.
  - Acceptance: compressor settings, dependencies, numeric tolerance, and known output risks are explicit.
  - Evidence: `docs/behavior-cards/manasa/*.md` (8 files), merged from Manasa's branch as `d689a84`.
- [ ] **PREP-03 — Poojitha — token and sequence behavior cards**
  - Output: notes for Hamming, Jaccard, Sørensen, Tversky, Cosine, Monge-Elkan, Bag, LCS substring, and RLE NCD.
  - Acceptance: set/multiset behavior, q-grams, repeated tokens, tie-breaking, and fixed examples are explicit.
- [ ] **PREP-04 — Suri — simple/proof behavior cards**
  - Output: notes for Overlap, Tanimoto, Ratcliff-Obershelp, BWT-RLE NCD, MRA, Prefix, Postfix, Length, Identity, and Matrix, plus the proof corpus outline.
  - Acceptance: each note has at least three expected examples and identifies the original test file.

### Lane 3 — Parallel algorithm packets

All tasks below depend on G1-10 and the corresponding PREP task. Each checkbox
means: implementation, focused native test, diff review, and a commit. The
owner may use the standard AI prompt in Section 7, but must keep the AI inside
the exact source/test paths shown in the ownership section.

#### Simha Teja’s packets

- [x] **SIM-01 — Levenshtein** — `edit/levenshtein.rs` + native test.
  - Evidence: `cargo test --test algorithm_levenshtein` (3 passed); `cargo test` (all native, contract, and registry tests passed).
- [x] **SIM-02 — Damerau-Levenshtein** — `edit/damerau_levenshtein.rs` + native test.
  - Evidence: `cargo test --test algorithm_damerau_levenshtein` (3 passed); restricted and unrestricted fixtures covered.
- [x] **SIM-03 — Needleman-Wunsch** — `edit/needleman_wunsch.rs` + native test.
  - Evidence: `cargo test --test algorithm_needleman_wunsch` (3 passed); identity, matrix, gap, empty, Unicode, q-gram, and normalization cases covered.
- [x] **SIM-04 — Smith-Waterman** — `edit/smith_waterman.rs` + native test.
  - Evidence: `cargo test --test algorithm_smith_waterman` (3 passed); local zero-reset, matrix, gap, empty/equal, Unicode, q-gram, and normalization cases covered.
- [x] **SIM-05 — Gotoh** — `edit/gotoh.rs` + native test.
  - Evidence: `cargo test --test algorithm_gotoh` (3 passed); affine gap, empty/equal, Unicode, q-gram, and inherited normalization cases covered.
- [x] **SIM-06 — StrCmp95** — `edit/strcmp95.rs` + native test.
  - Evidence: `cargo test --test algorithm_strcmp95` (3 passed); original floating-point fixtures, preprocessing, empty/equal, long-string option, and Unicode cases covered.
- [x] **SIM-07 — MLIPNS** — `edit/mlipns.rs` + native test.
  - Evidence: `cargo test --test algorithm_mlipns` (3 passed); original binary-similarity fixtures, threshold configuration, Unicode, q-grams, integers, and normalization covered.
- [x] **SIM-08 — Arithmetic NCD** — `compression/arith_ncd.rs` + native test.
  - Evidence: `cargo test --test algorithm_arith_ncd` (3 passed); original NCD values, stable probability ordering, exact `BANANA` numerator, empty, and q-gram cases covered.
- [x] **SIM-09 — LCS sequence** — `sequence/lcsseq.rs` + native test.
  - Dependency: G1-09 because the source call returns a sequence.
  - Evidence: `cargo test --test algorithm_lcsseq` (3 passed); original two-/multi-sequence outputs, tie behavior, empty, Unicode, q-gram, integer, and output-conversion cases covered.
- [x] **SIM-10 — FFI contract implementation** — `python_adapter/**`.
  - Dependency: at least SIM-01, POO-01, and one representative algorithm from Manasa and Suri.
  - Acceptance: the adapter invokes Rust only and exposes the common methods required by the original tests.
  - Evidence: `cargo test --features python` exercises the PyO3 adapter with text, bytes, integer sequences, sequence output, common methods, and callback rejection; `PYO3_BUILD_EXTENSION_MODULE=1 cargo build --features python-extension` plus a Python import probe passed.

#### Manasa’s packets

- [x] **MAN-01 — Jaro** — `edit/jaro.rs` + native test.
  - Evidence: `cargo test --test algorithm_jaro` (6 passed); all 8 frozen fixture values from `test_jaro.py`, empty/identical/no-match edge cases, and the constant-`maximum()` regression case covered.
- [x] **MAN-02 — Jaro-Winkler** — `edit/jaro_winkler.rs` + native test.
  - Evidence: `cargo test --test algorithm_jaro_winkler` (8 passed); all 7 frozen fixture values from `test_jaro_winkler.py`, prefix-boost vs. plain-Jaro comparison, and `long_tolerance` branch (unexercised by the frozen suite) covered.
- [x] **MAN-03 — Editex** — `phonetic/editex.rs` + native test.
  - Evidence: `cargo test --test algorithm_editex` (7 passed); all 17 non-local and 13 local frozen fixture values from `test_editex.py` covered, plus a regression test for the empty-input `quick_answer` shortcut (natural DP gives 13 for `''`/`'neilsen'`; frozen value is 14).
- [x] **MAN-04 — Square-root NCD** — `compression/sqrt_ncd.rs` + native test.
  - Evidence: `cargo test --test algorithm_sqrt_ncd` (6 passed); both frozen fixture values from `test_sqrt_ncd.py`, plus a regression test confirming identical inputs do *not* score zero (no `quick_answer` shortcut in the NCD family).
- [x] **MAN-05 — Entropy NCD** — `compression/entropy_ncd.rs` + native test.
  - Evidence: `cargo test --test algorithm_entropy_ncd` (7 passed); all 3 frozen fixture values from `test_entropy_ncd.py` (converted from similarity to distance), plus a `base` parameter regression test.
- [x] **MAN-06 — BZ2 NCD** — `compression/bz2_ncd.rs` + native test.
  - Dependency: G1-10, PREP-02, and DEP-03.
  - Evidence: `cargo test --test algorithm_bz2_ncd` (6 passed); both frozen fixture values from `test_bz2_ncd.py` matched exactly against the `bzip2` crate's output.
- [x] **MAN-07 — LZMA NCD** — `compression/lzma_ncd.rs` + native test.
  - Dependency: G1-10, PREP-02, and DEP-03.
  - Evidence: `cargo test --test algorithm_lzma_ncd` (7 passed); no frozen fixtures exist for this algorithm (confirmed absent from `tests/original/`), so reference values were generated by running the real, unmodified `textdistance.lzma_ncd` and confirmed to match the `xz2` crate's output exactly.
- [x] **MAN-08 — ZLIB NCD** — `compression/zlib_ncd.rs` + native test.
  - Dependency: G1-10, PREP-02, and DEP-03.
  - Evidence: `cargo test --test algorithm_zlib_ncd` (6 passed); reference values generated from the real `textdistance.zlib_ncd` and confirmed to match the `flate2` crate's output exactly.
- [x] **MAN-09 — dependency handoff** — `docs/dependency-notes/manasa.md`.
  - Acceptance: the note identifies reviewed candidates such as `bzip2`, `xz2` or a pure-Rust LZMA alternative, and `flate2`, plus feature flags, system-library requirements, licenses, settings, and fixed expected-output risks.
  - Evidence: `docs/dependency-notes/manasa.md` merged from Manasa's branch as `a6d8745`.

- [x] **DEP-02 — Simha Teja — integrate reviewed compression dependencies**
  - Dependency: MAN-09.
  - Output: reviewed `Cargo.toml` and `Cargo.lock` changes only for the required compression support.
  - Acceptance: dependency resolution and a minimal Rust compression smoke check pass; no algorithm owner edits `Cargo.toml`.
  - Evidence: `bzip2 = 0.4.4` with `static`, `xz2 = 0.1.7` with `static`, and `flate2 = 1.1.9` resolve and compile; BZ2/XZ/ZLIB encoder smoke check passed; `cargo fmt --check && cargo test` passed.
- [x] **DEP-03 — Manasa — validate the compression dependency lane**
  - Dependency: DEP-02.
  - Output: compile/smoke evidence for BZ2, LZMA, and ZLIB packets under `proof/` or the focused native tests.
  - Acceptance: all three packets can compile against the frozen dependency choices before their final differential fixtures are added.
  - Evidence: all three packets compile and pass against the frozen `DEP-02` dependency choices (`bzip2 0.4.4`/`static`, `xz2 0.1.7`/`static`, `flate2 1.1.9`) — `cargo test --test algorithm_bz2_ncd --test algorithm_lzma_ncd --test algorithm_zlib_ncd` (19 passed); full `cargo test` and `cargo fmt --check` also clean across the whole crate.

#### Poojitha’s packets

- [ ] **POO-01 — Hamming** — `edit/hamming.rs` + native test.
- [ ] **POO-02 — Jaccard** — `token/jaccard.rs` + native test.
- [ ] **POO-03 — Sørensen/Dice** — `token/sorensen.rs` + native test.
- [ ] **POO-04 — Tversky** — `token/tversky.rs` + native test.
- [ ] **POO-05 — Cosine** — `token/cosine.rs` + native test.
- [ ] **POO-06 — Monge-Elkan** — `token/monge_elkan.rs` + native test.
  - Dependency: G1-09 because the underlying comparison must cross the `SimilarityComparator` seam.
- [ ] **POO-07 — Bag** — `token/bag.rs` + native test.
- [ ] **POO-08 — LCS substring** — `sequence/lcsstr.rs` + native test.
  - Dependency: G1-09 because the source call returns the substring, not only its length.
- [ ] **POO-09 — RLE NCD** — `compression/rle_ncd.rs` + native test.

#### Suri’s packets

- [ ] **SUR-01 — Overlap** — `token/overlap.rs` + native test.
- [ ] **SUR-02 — Tanimoto** — `token/tanimoto.rs` + native test.
- [ ] **SUR-03 — Ratcliff-Obershelp** — `sequence/ratcliff_obershelp.rs` + native test.
- [ ] **SUR-04 — BWT-RLE NCD** — `compression/bwtrle_ncd.rs` + native test.
- [ ] **SUR-05 — MRA** — `phonetic/mra.rs` + native test.
- [ ] **SUR-06 — Prefix** — `simple/prefix.rs` + native test.
- [ ] **SUR-07 — Postfix** — `simple/postfix.rs` + native test.
- [ ] **SUR-08 — Length** — `simple/length.rs` + native test.
- [ ] **SUR-09 — Identity** — `simple/identity.rs` + native test.
- [ ] **SUR-10 — Matrix** — `simple/matrix.rs` + native test.

### Lane 4 — Integration and proof gates

These tasks are intentionally sequential after the parallel packets.

- [x] **INT-01 — Suri — wire the Python package path**
  - Dependency: G0-05 and SIM-10.
  - Output: thin `textdistance/**` package path that loads the Rust adapter and does not contain the original implementation.
  - Acceptance: an import probe proves the Rust-backed package is loaded.
- [ ] **INT-02 — Suri — run unchanged original tests**
  - Dependency: INT-01 and all algorithm packets.
  - Output: test report for `tests/original/`.
  - Acceptance: all original tests pass, with external tests run when their documented dependencies are installed.
  - Evidence: `make verify` passes 400 non-external tests; the full run reaches 427/430 because RapidFuzz 3.14.5 disagrees with the frozen implementation on three large-integer list cases. See `docs/DECISIONS.md`.
- [x] **INT-03 — all owners — fix only owned failures**
  - Dependency: INT-02 failure report.
  - Output: one fix commit per owner/algorithm packet.
  - Acceptance: no integration fix edits another owner’s implementation file.
- [x] **INT-04 — Suri — freeze differential fixtures**
  - Dependency: INT-02.
  - Output: fixed corpus under `proof/` covering Unicode, empty/equal, q-values, multi-sequence, integers, bytes, and options.
  - Acceptance: the corpus can be rerun without the original Python implementation as a runtime dependency.
- [x] **INT-05 — Suri — run fuzz smoke test**
  - Dependency: INT-04.
  - Output: seed, duration, and result under `proof/`.
  - Acceptance: zero crashes and zero unexplained mismatches.
- [x] **INT-06 — Suri — capture benchmarks**
  - Dependency: INT-02.
  - Output: reproducible benchmark report under `bench/`.
  - Acceptance: baseline and Rust commands, inputs, machine details, and results are recorded.
- [x] **INT-07 — Simha Teja — final core/FFI review**
  - Dependency: INT-03.
  - Output: review commit or signed-off review note.
  - Acceptance: no Python fallback, no unsafe core logic, and no unexplained public API drift.
- [x] **INT-08 — Suri — README/DECISIONS/DEMO finalization**
  - Dependency: INT-04 through INT-07.
  - Output: complete submission documentation.
  - Acceptance: a new teammate can run the build, tests, proof, benchmark, and five-minute demo from the README.
- [x] **INT-09 — all — clean-checkout rehearsal**
  - Dependency: INT-08.
  - Output: final command log and clean working tree review.
  - Acceptance: one documented command succeeds from a fresh checkout and the original-test manifest still passes.
  - Evidence: `proof/clean-checkout.md` records a fresh archive of commit `2cb7fdb` passing `make verify`.

### Dependency map

```text
G0-01 → G0-02 → G0-04 → G0-05
            │
            └────────── G1-01 → G1-02 → G1-03 → G1-04 → G1-06 → G1-07 → G1-08 → G1-09 → G1-10
                                                                                              │
                                      PREP-01..04 ───────────────────────────────────────────┘
                                                                                              │
                                      SIM/MAN/POO/SUR packets ───────────────────────────────┘
                                                                        │
                                      MAN-09 → DEP-02 → DEP-03 → MAN-06..08
                                                            │
                                      SIM-10 → INT-01 → INT-02
                                                            │
                                      INT-03 → INT-04 → INT-05
                                                  └── INT-06
                                      INT-07 → INT-08 → INT-09
```
