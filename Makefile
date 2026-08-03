.PHONY: build test verify benchmark demo test-external

PYTHON ?= $(or $(wildcard .venvs/g0/bin/python),$(wildcard .venvs/pytest-pure/bin/python),$(wildcard .venvs/pytest-ext/bin/python),python3)
CARGO ?= cargo
PYO3_PYTHON ?= $(if $(wildcard $(PYTHON)),$(abspath $(PYTHON)),$(shell command -v $(PYTHON)))

export PYO3_PYTHON

build:
	PYO3_BUILD_EXTENSION_MODULE=1 $(CARGO) build --features python-extension

test: build
	$(CARGO) test --features python --quiet
	$(PYTHON) -m pytest tests/original -m 'not external' -q

test-external: build
	$(PYTHON) -m pytest tests/original -q

verify: build
	shasum -a 256 -c proof/original-tests.sha256
	$(CARGO) test --features python --quiet
	$(PYTHON) proof/verify_corpus.py
	$(CARGO) test --test fuzz_smoke -- --nocapture
	$(PYTHON) -m pytest tests/original -m 'not external' -q

benchmark: build
	$(CARGO) bench --bench suri_bench
	$(PYTHON) bench/scripts/bench_python_baseline.py

demo: build
	$(PYTHON) -c "import textdistance as td; print('Levenshtein:', td.levenshtein('kitten', 'sitting')); print('Jaro-Winkler:', td.jaro_winkler('MARTHA', 'MARHTA')); print('Prefix qval=2:', td.Prefix(qval=2)('testing', 'tester')); print('Matrix:', td.Matrix(match_cost=2.0, mismatch_cost=-1.0)('x', 'y'))"
