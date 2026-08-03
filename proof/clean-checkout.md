# INT-09 — Clean-checkout rehearsal

**Status: PASS — 2026-08-03.**

The committed tree at `2cb7fdb` was exported with `git archive` into a fresh
temporary directory. No working-tree edits, generated target files, or local
uncommitted changes were included. From that directory, the documented
verification command was run with the project test interpreter:

```sh
make PYTHON=/Users/teja/raptors_hackathon/textdistance/.venvs/g0/bin/python verify
```

Results:

```text
original-test manifest: all files OK
cargo test --features python: passed
proof/verify_corpus.py: 114/114 corpus cases passed
cargo test --test fuzz_smoke: 1 passed, 0 failed
pytest tests/original -m 'not external': 400 passed, 30 deselected
```

The separate full external run remains documented as 427/430 because
RapidFuzz 3.14.5 disagrees with the frozen implementation on three
large-integer list cases; that provider mismatch is not part of `make verify`.
