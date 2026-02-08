FMT 		= cargo fmt --all -- --check
LINT 		= CARGO_BUILD_JOBS=10 cargo clippy -p bijux-core -p bijux-engine -p bijux-api -p bijux --lib --bins --no-deps -- -D warnings
TEST 		= cargo nextest run --workspace --run-ignored all
AUDIT 		= cargo deny check
COVERAGE_ROOT = $(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target)
COVERAGE_OUT = $(COVERAGE_ROOT)/llvm-cov/coverage.json
HTML_OUT     = $(COVERAGE_ROOT)/llvm-cov/html
COVERAGE 	= cargo llvm-cov --json --output-path $(COVERAGE_OUT) test --workspace --all-features --tests --benches --bins -- --include-ignored
COVERAGE_ENV = RUST_TEST_THREADS=1 CARGO_LLVM_COV_TARGET_DIR=$(COVERAGE_ROOT) CARGO_LLVM_COV_BUILD_DIR=$(COVERAGE_ROOT)

fmt:
	$(FMT)

lint:
	$(LINT)

test:
	$(TEST)

ensure-cargo-deny:
	@command -v cargo-deny >/dev/null 2>&1 || cargo install cargo-deny

audit: ensure-cargo-deny
	$(AUDIT)

coverage:
	@mkdir -p $(dir $(COVERAGE_OUT))
	$(COVERAGE_ENV) $(COVERAGE)
	python3 scripts/coverage_summary.py $(COVERAGE_OUT)

fmt-isolate:
	CARGO_TARGET_DIR=target-isolate $(FMT)

lint-isolate:
	CARGO_TARGET_DIR=target-isolate $(LINT)

test-isolate:
	CARGO_TARGET_DIR=target-isolate $(TEST)

audit-isolate: ensure-cargo-deny
	CARGO_TARGET_DIR=target-isolate $(AUDIT)

coverage-isolate: CARGO_TARGET_DIR=target-isolate
coverage-isolate:
	@mkdir -p $(dir $(COVERAGE_OUT))
	$(COVERAGE_ENV) $(COVERAGE)
	python3 scripts/coverage_summary.py $(COVERAGE_OUT)

coverage-html:
	$(COVERAGE_ENV) cargo llvm-cov test --workspace --all-features --tests --benches --bins --html --output-dir $(HTML_OUT) -- --include-ignored

coverage-html-isolate: CARGO_TARGET_DIR=target-isolate
coverage-html-isolate:
	$(COVERAGE_ENV) cargo llvm-cov test --workspace --all-features --tests --benches --bins --html --output-dir $(HTML_OUT) -- --include-ignored

ci: fmt lint audit coverage

ci-isolate:
	CARGO_TARGET_DIR=target-isolate $(MAKE) ci
