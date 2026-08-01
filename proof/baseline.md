# Baseline — Original Python Test Suite (Lane 0)

**Status: PASSING — 400 tests green on the pure-Python original.**

This file records the state of the original pure-Python `textdistance` project
**before any Rust implementation begins**. Every later claim that the Rust port
"matches the original" is measured against exactly what is written here.

> Revision note: an earlier version of this file recorded status BLOCKED with 0
> tests collected, because `pytest` was not installed at the time. A dedicated
> virtualenv has since been created and the suite now runs. The environment and
> result sections below supersede that record. The findings under
> [Setup issues and notes](#setup-issues-and-notes) were carried forward and
> re-verified.

Recorded on 2026-08-02.

## Git state

| Item | Value |
| --- | --- |
| Commit hash | `d6a68d61088a40eef5c88191ccf79323dbf34850` |
| Short hash | `d6a68d6` |
| Branch | `suri` |
| Commit subject | Merge pull request #96 from christian-eriksson/patch-1 |
| Commit author | Gram |
| Commit date | 2025-04-18T14:13:17+02:00 |
| Remote | <https://github.com/Simhateja17/raptors_hackathon> |

**Why this is recorded:** the commit hash pins the exact Python source the port
must reproduce. Without it, "we matched the baseline" cannot be verified by
anyone else.

### Working tree at time of recording

The working tree was **not** clean. Two untracked directories were present:

```text
?? proof/
?? tests/original/
```

`tests/original/` is the byte-for-byte test snapshot produced by task G0-01. It
is untracked, so it is **not** part of commit `d6a68d6`. No tracked file was
modified — `git diff --name-only HEAD` was empty.

## Python environment

| Item | Value |
| --- | --- |
| Interpreter used for baseline | Python 3.13.2 |
| Interpreter path | `C:\Users\mg875\AppData\Local\Programs\Python\Python313\python.exe` |
| Second interpreter present | Python 3.12.4 |
| Virtual environment | `.venvs/pytest-pure` (created for this task; gitignored) |
| Install performed | `pip install '.[test]'` into that venv |
| `textdistance` resolves to | `C:\Users\mg875\Desktop\raptors_hackathon\textdistance\__init__.py` |

**Why this is recorded:** Python version affects float formatting, dict ordering
and `math` precision. A Rust port that matches 3.13 output is not automatically
identical to 3.8 output, so the version must be part of the record.

### Version caveat

Upstream CI (`.github/workflows/main.yml`) tests **Python 3.8 – 3.11 only**.
This baseline was captured on **Python 3.13.2**, outside that matrix. Any
behavioural change introduced between 3.11 and 3.13 is baked into these numbers.

### Test dependency versions

```text
hypothesis==6.164.0
isort==8.0.1
numpy==2.5.1
pytest==9.1.1
pluggy==1.6.0
iniconfig==2.3.0
packaging==26.2
sortedcontainers==2.4.0
colorama==0.4.6
Pygments==2.20.0
setuptools==83.0.0
wheel==0.47.0
```

## Operating system and hardware

| Item | Value |
| --- | --- |
| OS | Microsoft Windows 11 Home Single Language |
| OS version | 10.0.26100 (build 26100) |
| Architecture | AMD64 (x86-64) |
| CPU | 12th Gen Intel(R) Core(TM) i5-12450H |
| Logical cores | 12 |
| Shell | PowerShell 5.1.26100.6584 and Git Bash |

**Why this is recorded:** platform determines floating-point rounding behaviour
and is the honest denominator for any later performance comparison. A speedup
number is meaningless without the machine it was measured on.

## Test command

### Canonical project command

The project defines its test entry point in `Taskfile.yml`, and CI invokes it
through `.github/workflows/main.yml`:

```sh
task pytest-pure
```

which expands (`Taskfile.yml` line 69) to:

```sh
.venvs/pytest-pure/bin/pytest -m 'not external'
```

`task pytest-external` is the companion target: it drops the marker filter and
additionally runs the 30 tests that compare against third-party libraries.

### Command actually executed for this baseline

The canonical command **does not run on this machine**. `task` (go-task) is not
installed, and `Taskfile.yml` hardcodes the POSIX path `.venvs/.../bin/pytest`,
which on Windows is `.venvs/.../Scripts/`. The Taskfile is Linux/macOS-only as
written. The Windows equivalent used instead, from the repository root:

```sh
.venvs/pytest-pure/Scripts/python.exe -m pytest -m 'not external' --ignore=tests/original -q
```

Two flags carry meaning:

- `-m 'not external'` mirrors the project's own `pytest-pure` target. The
  `external` marker is declared in `setup.cfg` and gates tests that check
  `textdistance` against third-party libraries.
- `--ignore=tests/original` is required because the G0-01 snapshot lives inside
  the test tree. Without it, pytest collects both trees and every count doubles.

## Baseline results

```text
400 passed, 30 deselected in 15.10s
```

| Metric | Value |
| --- | --- |
| Exit code | 0 |
| Tests passed | 400 |
| Tests failed | 0 |
| Tests skipped | 0 |
| Deselected (`external` marker) | 30 |
| Total collected | 430 |
| Wall time | 15.10 s |

### Collected tests by module

| Path | Tests |
| --- | --- |
| `tests/test_common.py` | 168 |
| `tests/test_edit/` | 111 |
| `tests/test_compression/` | 51 |
| `tests/test_external.py` | 30 |
| `tests/test_phonetic/` | 30 |
| `tests/test_sequence/` | 21 |
| `tests/test_token/` | 19 |
| **Total** | **430** |

## External suite — not executed

The 30 tests in `tests/test_external.py` were deselected and **not run**. They
require the `benchmark` extra from `setup.py`, none of which is installed:

| Package | Installed? |
| --- | --- |
| `jellyfish` | no |
| `Levenshtein` | no |
| `pyxDamerauLevenshtein` | no |
| `rapidfuzz>=2.6.0` | no |
| `pylev` | no |
| `py_stringmatching` | no |

To capture an external baseline later, install the `benchmark` extra into a
second venv and drop the `-m 'not external'` filter.

## Setup issues and notes

1. **The Taskfile does not work on Windows.** `Taskfile.yml` expects per-purpose
   venvs under `.venvs/` created via `task`, and uses POSIX `{{.ENV}}/bin/`
   paths. Windows venvs place executables in `Scripts/`. Anyone on Windows must
   use the explicit command recorded above.
1. **`textdistance` is imported from the working tree**, not from an installed
   wheel, so tests exercise local source as long as pytest runs from the repo
   root.
1. **Version mismatch inside the repo.** `setup.py` line 103 declares
   `version='4.6.3'`, while `textdistance.__version__` reports `4.6.2`. This is a
   pre-existing upstream inconsistency. It was **not** modified.
1. **`tests/original/` is untracked in git.** It appears in no commit, so the
   G0-01 snapshot currently has no version-control provenance.
1. **G0-02 was not completed.** `proof/original-tests.sha256` does not exist, so
   the 35 files in `tests/original/` have no integrity manifest yet.

## Meaning of this baseline

The Rust implementation must reproduce **400 passing tests, 0 failures** under
the same command against the same commit's test files. The 15.10 s wall time is
the pure-Python reference point for any speedup claim, and is valid only for the
hardware listed above.

## Compliance for this task

- No Python source modified.
- No Rust source created or modified.
- No test files modified.
- No failing tests fixed or suppressed.
- Packages installed **only** into the gitignored `.venvs/pytest-pure`
  virtualenv; no system interpreter was altered.
