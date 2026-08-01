# G0 Baseline Freeze

## Snapshot

- Baseline commit: `d6a68d61088a40eef5c88191ccf79323dbf34850`
- Baseline branch: `main`
- System Python at discovery: `Python 3.9.6`
- Isolated baseline environment: `.venvs/g0`, `Python 3.14`
- Original test files frozen: `35`
- Test manifest: [`original-tests.sha256`](original-tests.sha256)

## Verification

The frozen test tree passes its manifest check:

```text
shasum -a 256 -c proof/original-tests.sha256
35 files: OK
```

The baseline test environment is now installed in `.venvs/g0`. The pure/original
suite passes:

```text
./.venvs/g0/bin/python -m pytest -q tests/original -m 'not external'
400 passed, 30 deselected in 8.19s
```

The complete original suite was also run. Its 30 failures are all in the
optional `test_external.py` checks, because the external comparison packages
are not installed in this baseline environment:

```text
./.venvs/g0/bin/python -m pytest -q tests/original
400 passed, 30 failed in 24.10s
```

The missing optional packages reported by the failures include
`jellyfish`, `Levenshtein`, `pylev`, and `pyxdameraulevenshtein`. These are
environment/dependency failures, not changes to the frozen source tests.

The pre-existing planning file `PRD.md` was untracked before G0 and is preserved; no original test file was modified.
