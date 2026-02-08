FMT 		= cargo fmt --all -- --check
LINT 		= CARGO_BUILD_JOBS=10 cargo clippy -p bijux-core -p bijux-engine -p bijux-api -p bijux --lib --bins --no-deps -- -D warnings
TEST 		= cargo nextest run --workspace
AUDIT 		= cargo deny check
COVERAGE 	= cargo llvm-cov nextest run --workspace --json --output-path target/llvm-cov/coverage.json

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
	$(COVERAGE)
	python3 scripts/coverage_summary.py target/llvm-cov/coverage.json

fmt-isolate:
	CARGO_TARGET_DIR=target-isolate $(FMT)

lint-isolate:
	CARGO_TARGET_DIR=target-isolate $(LINT)

test-isolate:
	CARGO_TARGET_DIR=target-isolate $(TEST)

audit-isolate: ensure-cargo-deny
	CARGO_TARGET_DIR=target-isolate $(AUDIT)

coverage-isolate:
	CARGO_TARGET_DIR=target-isolate $(COVERAGE)
	python3 scripts/coverage_summary.py target/llvm-cov/coverage.json

coverage-html:
	cargo llvm-cov nextest run --workspace --html

coverage-html-isolate:
	CARGO_TARGET_DIR=target-isolate cargo llvm-cov nextest run --workspace --html

ci: fmt lint test audit coverage

ci-isolate:
	CARGO_TARGET_DIR=target-isolate $(MAKE) ci
