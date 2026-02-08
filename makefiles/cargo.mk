FMT 		= cargo fmt --all -- --check
LINT 		= CARGO_BUILD_JOBS=10 cargo clippy --workspace --all-targets --all-features -- -D warnings
AUDIT 		= cargo deny check
NEXTEST_PROFILE 	?= ci
NEXTEST_CONFIG  	?= --config-file nextest.toml
RUN_IGNORED 		= --run-ignored all
TEST_FEATURES 		= --all-features
TEST_TARGET_DIR 	?= target-test
COVERAGE_TARGET_DIR ?= target-cov
TEST_TMPDIR 		= $(abspath $(TEST_TARGET_DIR))/tmp
TEST_PROFILE_DIR 	= $(TEST_TMPDIR)/profraw
TEST_ENV 			= TZ=UTC LC_ALL=C TMPDIR=$(TEST_TMPDIR) TMP=$(TEST_TMPDIR) TEMP=$(TEST_TMPDIR) LLVM_PROFILE_FILE=$(TEST_PROFILE_DIR)/%p.profraw
TEST 				= $(TEST_ENV) CARGO_TARGET_DIR=$(TEST_TARGET_DIR) cargo nextest run $(NEXTEST_CONFIG) --workspace $(TEST_FEATURES) --profile $(NEXTEST_PROFILE) $(RUN_IGNORED)
COVERAGE_ROOT 		= $(COVERAGE_TARGET_DIR)
COVERAGE_ROOT_ABS 	= $(abspath $(COVERAGE_ROOT))
COVERAGE_TARGET_DIR_ABS = $(abspath $(COVERAGE_TARGET_DIR))
COVERAGE_OUT 		= $(COVERAGE_ROOT)/llvm-cov/coverage.json
HTML_OUT     		= $(COVERAGE_ROOT)/llvm-cov/html
COVERAGE_BASELINE 	= coverage/baseline.json
COVERAGE_THRESHOLDS = coverage/thresholds.json
COVERAGE_TMPDIR 	= $(COVERAGE_ROOT_ABS)/tmp
COVERAGE_ENV 		= TZ=UTC LC_ALL=C TMPDIR=$(COVERAGE_TMPDIR) TMP=$(COVERAGE_TMPDIR) TEMP=$(COVERAGE_TMPDIR) \
  CARGO_TARGET_DIR=$(COVERAGE_TARGET_DIR) \
  CARGO_LLVM_COV_TARGET_DIR=$(COVERAGE_TARGET_DIR_ABS) \
  CARGO_LLVM_COV_BUILD_DIR=$(COVERAGE_TARGET_DIR_ABS) \
  LLVM_PROFILE_FILE=$(COVERAGE_ROOT_ABS)/llvm-cov/profraw/%p.profraw
COVERAGE_RUN 		= cargo llvm-cov nextest --no-report --no-cfg-coverage $(NEXTEST_CONFIG) --workspace $(TEST_FEATURES) --profile $(NEXTEST_PROFILE) $(RUN_IGNORED)
COVERAGE_JSON 		= cargo llvm-cov report --json --output-path $(COVERAGE_OUT)
COVERAGE_HTML 		= cargo llvm-cov report --html --output-dir $(HTML_OUT)

fmt:
	$(FMT)

lint:
	$(LINT)

test:
	@mkdir -p $(TEST_TMPDIR)
	@mkdir -p $(TEST_PROFILE_DIR)
	@find crates -name '*.profraw' -delete
	$(TEST)

ensure-cargo-deny:
	@command -v cargo-deny >/dev/null 2>&1 || cargo install cargo-deny

audit: ensure-cargo-deny
	$(AUDIT)

coverage:
	@mkdir -p $(dir $(COVERAGE_OUT))
	@mkdir -p $(COVERAGE_TMPDIR)
	@mkdir -p $(COVERAGE_ROOT)/llvm-cov/profraw
	@find crates -name '*.profraw' -delete
	$(COVERAGE_ENV) $(COVERAGE_RUN)
	$(COVERAGE_ENV) $(COVERAGE_JSON)
	$(COVERAGE_ENV) $(COVERAGE_HTML)
	@if [ -f $(COVERAGE_BASELINE) ]; then \
		python3 scripts/coverage_summary.py $(COVERAGE_OUT) --baseline $(COVERAGE_BASELINE) --check-thresholds $(COVERAGE_THRESHOLDS); \
	else \
		python3 scripts/coverage_summary.py $(COVERAGE_OUT) --check-thresholds $(COVERAGE_THRESHOLDS); \
	fi

fmt-isolate:
	TEST_TARGET_DIR=target-isolate-test COVERAGE_TARGET_DIR=target-isolate-cov $(MAKE) fmt

lint-isolate:
	TEST_TARGET_DIR=target-isolate-test COVERAGE_TARGET_DIR=target-isolate-cov $(MAKE) lint

test-isolate:
	TEST_TARGET_DIR=target-isolate-test COVERAGE_TARGET_DIR=target-isolate-cov $(MAKE) test

audit-isolate: ensure-cargo-deny
	TEST_TARGET_DIR=target-isolate-test COVERAGE_TARGET_DIR=target-isolate-cov $(MAKE) audit

coverage-isolate:
	TEST_TARGET_DIR=target-isolate-test COVERAGE_TARGET_DIR=target-isolate-cov $(MAKE) coverage

define run_ci
	$(if $(1),TEST_TARGET_DIR=$(1)-test COVERAGE_TARGET_DIR=$(1)-cov,) $(MAKE) fmt lint audit coverage
endef

ci:
	$(call run_ci,)

ci-isolate:
	$(call run_ci,target-isolate)

ci-local:
	$(MAKE) -j2 test coverage

snapshots:
	$(TEST_ENV) cargo insta test --workspace

snapshots-accept:
	$(TEST_ENV) cargo insta accept --workspace

snapshots-review:
	$(TEST_ENV) cargo insta review

.PHONY: fmt lint test audit coverage ci ci-local \
		fmt-isolate lint-isolate test-isolate audit-isolate coverage-isolate ci-isolate \
		snapshots snapshots-accept snapshots-review ensure-cargo-deny
