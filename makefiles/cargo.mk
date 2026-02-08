FMT 		= cargo fmt --all -- --check
LINT 		= CARGO_BUILD_JOBS=10 cargo clippy -p bijux-core -p bijux-engine -p bijux-api -p bijux --lib --bins --no-deps -- -D warnings
AUDIT 		= cargo deny check
NEXTEST_PROFILE ?= ci
NEXTEST_CONFIG  ?= --config-file nextest.toml
RUN_IGNORED 	= --run-ignored all
TEST_FEATURES 	= --all-features
TEST_TMPDIR 	= $(abspath $(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target))/tmp
TEST_ENV 	= TZ=UTC LC_ALL=C TMPDIR=$(TEST_TMPDIR)
TEST 		= $(TEST_ENV) cargo nextest run $(NEXTEST_CONFIG) --workspace $(TEST_FEATURES) --profile $(NEXTEST_PROFILE) $(RUN_IGNORED)
COVERAGE_ROOT = $(if $(CARGO_TARGET_DIR),$(CARGO_TARGET_DIR),target)
COVERAGE_OUT = $(COVERAGE_ROOT)/llvm-cov/coverage.json
HTML_OUT     = $(COVERAGE_ROOT)/llvm-cov/html
COVERAGE_BASELINE = coverage/baseline.json
COVERAGE_THRESHOLDS = coverage/thresholds.json
COVERAGE_ENV = $(TEST_ENV) CARGO_LLVM_COV_TARGET_DIR=$(COVERAGE_ROOT) CARGO_LLVM_COV_BUILD_DIR=$(COVERAGE_ROOT) LLVM_PROFILE_FILE=$(abspath $(COVERAGE_ROOT))/llvm-cov/profraw/%p.profraw
COVERAGE_RUN = cargo llvm-cov nextest --no-report --no-cfg-coverage $(NEXTEST_CONFIG) --workspace $(TEST_FEATURES) --profile $(NEXTEST_PROFILE) $(RUN_IGNORED)
COVERAGE_JSON = cargo llvm-cov report --json --output-path $(COVERAGE_OUT)
COVERAGE_HTML = cargo llvm-cov report --html --output-dir $(HTML_OUT)

fmt:
	$(FMT)

lint:
	$(LINT)

test:
	@mkdir -p $(TEST_TMPDIR)
	$(TEST)

ensure-cargo-deny:
	@command -v cargo-deny >/dev/null 2>&1 || cargo install cargo-deny

audit: ensure-cargo-deny
	$(AUDIT)

coverage:
	@mkdir -p $(dir $(COVERAGE_OUT))
	@mkdir -p $(TEST_TMPDIR)
	@mkdir -p $(COVERAGE_ROOT)/llvm-cov/profraw
	$(COVERAGE_ENV) $(COVERAGE_RUN)
	$(COVERAGE_ENV) $(COVERAGE_JSON)
	$(COVERAGE_ENV) $(COVERAGE_HTML)
	@if [ -f $(COVERAGE_BASELINE) ]; then \
		python3 scripts/coverage_summary.py $(COVERAGE_OUT) --baseline $(COVERAGE_BASELINE) --check-thresholds $(COVERAGE_THRESHOLDS); \
	else \
		python3 scripts/coverage_summary.py $(COVERAGE_OUT) --check-thresholds $(COVERAGE_THRESHOLDS); \
	fi

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
	CARGO_TARGET_DIR=target-isolate $(MAKE) coverage

define run_ci
	$(if $(1),CARGO_TARGET_DIR=$(1) ,)$(MAKE) fmt lint audit coverage
endef

ci:
	$(call run_ci,)

ci-isolate:
	$(call run_ci,target-isolate)

snapshots:
	cargo insta test --workspace

snapshots-accept:
	cargo insta accept --workspace

snapshots-review:
	cargo insta review
