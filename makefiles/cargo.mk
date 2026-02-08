FMT 		= cargo fmt --all -- --check
LINT 		= CARGO_BUILD_JOBS=10 cargo clippy -p bijux-core -p bijux-engine -p bijux-api -p bijux --lib --bins --no-deps -- -D warnings
TEST 		= cargo nextest run --workspace
AUDIT 		= cargo deny check
COVERAGE_OUT = $(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target)/llvm-cov/coverage.json
HTML_OUT     = $(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target)/llvm-cov/html
COVERAGE 	= cargo llvm-cov nextest run --workspace --json --output-path $(COVERAGE_OUT)

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
	$(COVERAGE)
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
	$(COVERAGE)
	python3 scripts/coverage_summary.py $(COVERAGE_OUT)

coverage-html:
	cargo llvm-cov nextest run --workspace --html --output-dir $(HTML_OUT)

coverage-html-isolate: CARGO_TARGET_DIR=target-isolate
coverage-html-isolate:
	cargo llvm-cov nextest run --workspace --html --output-dir $(HTML_OUT)

ci: fmt lint test audit coverage

ci-isolate:
	CARGO_TARGET_DIR=target-isolate $(MAKE) ci
