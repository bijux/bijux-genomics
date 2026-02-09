FMT = cargo fmt --all -- --check
LINT = CARGO_BUILD_JOBS=10 cargo clippy --workspace --all-targets --all-features -- -D warnings
AUDIT = cargo deny check
NEXTEST_PROFILE ?= ci
NEXTEST_CONFIG ?= --config-file nextest.toml
RUN_IGNORED = --run-ignored all
TEST_FEATURES = --all-features
TEST_TARGET_DIR ?= target-test
COV_TARGET_DIR ?= target-cov
TEST_TMP_DIR ?= $(abspath $(TEST_TARGET_DIR))/tmp
COV_TMP_DIR ?= $(abspath $(COV_TARGET_DIR))/tmp
TEST_PROFRAW_DIR ?= $(abspath $(TEST_TARGET_DIR))/profraw
COV_PROFRAW_DIR ?= $(abspath $(COV_TARGET_DIR))/profraw
TEST_ENV = TZ=UTC LC_ALL=C TMPDIR=$(TEST_TMP_DIR) TMP=$(TEST_TMP_DIR) TEMP=$(TEST_TMP_DIR) \
  TEST_TARGET_DIR=$(TEST_TARGET_DIR) COV_TARGET_DIR=$(COV_TARGET_DIR) \
  TEST_TMP_DIR=$(TEST_TMP_DIR) COV_TMP_DIR=$(COV_TMP_DIR) \
  TEST_PROFRAW_DIR=$(TEST_PROFRAW_DIR) COV_PROFRAW_DIR=$(COV_PROFRAW_DIR) \
  LLVM_PROFILE_FILE=$(TEST_PROFRAW_DIR)/%p.profraw
TEST = $(TEST_ENV) CARGO_TARGET_DIR=$(TEST_TARGET_DIR) cargo nextest run $(NEXTEST_CONFIG) --workspace $(TEST_FEATURES) --profile $(NEXTEST_PROFILE) $(RUN_IGNORED)
COVERAGE_ROOT = $(COV_TARGET_DIR)/coverage
COVERAGE_ROOT_ABS = $(abspath $(COVERAGE_ROOT))
COV_TARGET_DIR_ABS = $(abspath $(COV_TARGET_DIR))
COVERAGE_OUT = $(COVERAGE_ROOT)/coverage.json
HTML_OUT = $(COVERAGE_ROOT)/html
COVERAGE_BASELINE = artifacts/coverage/baseline.json
COVERAGE_THRESHOLDS = coverage/thresholds.json
COVERAGE_ENV = TZ=UTC LC_ALL=C TMPDIR=$(COV_TMP_DIR) TMP=$(COV_TMP_DIR) TEMP=$(COV_TMP_DIR) \
  TEST_TARGET_DIR=$(TEST_TARGET_DIR) COV_TARGET_DIR=$(COV_TARGET_DIR) \
  TEST_TMP_DIR=$(TEST_TMP_DIR) COV_TMP_DIR=$(COV_TMP_DIR) \
  TEST_PROFRAW_DIR=$(TEST_PROFRAW_DIR) COV_PROFRAW_DIR=$(COV_PROFRAW_DIR) \
  CARGO_TARGET_DIR=$(COV_TARGET_DIR) \
  CARGO_LLVM_COV_TARGET_DIR=$(COV_TARGET_DIR_ABS) \
  CARGO_LLVM_COV_BUILD_DIR=$(COV_TARGET_DIR_ABS) \
  LLVM_PROFILE_FILE=$(COV_PROFRAW_DIR)/%p.profraw
COVERAGE_RUN = cargo llvm-cov nextest --no-report --no-cfg-coverage $(NEXTEST_CONFIG) --workspace $(TEST_FEATURES) --profile $(NEXTEST_PROFILE) $(RUN_IGNORED)
COVERAGE_JSON = cargo llvm-cov report --json --output-path $(COVERAGE_OUT)
COVERAGE_HTML = cargo llvm-cov report --html --output-dir $(COVERAGE_ROOT)

fmt:
	$(FMT)

lint:
	$(LINT)

test:
	@rm -rf $(TEST_PROFRAW_DIR)
	@mkdir -p $(TEST_TMP_DIR)
	@mkdir -p $(TEST_PROFRAW_DIR)
	$(TEST)

ensure-cargo-deny:
	@command -v cargo-deny >/dev/null 2>&1 || cargo install cargo-deny

audit: ensure-cargo-deny
	$(AUDIT)

coverage:
	@rm -rf $(COVERAGE_ROOT)
	@mkdir -p $(dir $(COVERAGE_OUT))
	@$(COVERAGE_ENV) cargo llvm-cov clean
	@rm -rf $(COV_PROFRAW_DIR)
	@mkdir -p $(COV_TMP_DIR)
	@mkdir -p $(COV_PROFRAW_DIR)
	$(COVERAGE_ENV) $(COVERAGE_RUN)
	$(COVERAGE_ENV) $(COVERAGE_JSON)
	$(COVERAGE_ENV) $(COVERAGE_HTML)
	@test -f $(COVERAGE_OUT)
	@test -f $(HTML_OUT)/index.html
	@if [ -f $(COVERAGE_BASELINE) ]; then \
		python3 scripts/coverage_summary.py $(COVERAGE_OUT) --baseline $(COVERAGE_BASELINE) --check-thresholds $(COVERAGE_THRESHOLDS); \
	else \
		python3 scripts/coverage_summary.py $(COVERAGE_OUT) --check-thresholds $(COVERAGE_THRESHOLDS); \
	fi

fmt-isolate:
	TEST_TARGET_DIR=target-isolate-test COV_TARGET_DIR=target-isolate-cov $(MAKE) fmt

lint-isolate:
	TEST_TARGET_DIR=target-isolate-test COV_TARGET_DIR=target-isolate-cov $(MAKE) lint

test-isolate:
	TEST_TARGET_DIR=target-isolate-test COV_TARGET_DIR=target-isolate-cov $(MAKE) test

audit-isolate: ensure-cargo-deny
	TEST_TARGET_DIR=target-isolate-test COV_TARGET_DIR=target-isolate-cov $(MAKE) audit

coverage-isolate:
	TEST_TARGET_DIR=target-isolate-test COV_TARGET_DIR=target-isolate-cov $(MAKE) coverage

define run_ci
	$(if $(1),TEST_TARGET_DIR=$(1)-test COV_TARGET_DIR=$(1)-cov,) $(MAKE) fmt lint audit coverage
endef

ci:
	$(call run_ci,)

check:
	$(MAKE) fmt lint audit coverage

ci-isolate:
	$(call run_ci,target-isolate)

ci-local:
	$(MAKE) -j2 test coverage

verify-parallel-isolation:
	@test "$(TEST_TARGET_DIR)" != "$(COV_TARGET_DIR)"
	@test "$(TEST_TMP_DIR)" != "$(COV_TMP_DIR)"
	@test "$(TEST_PROFRAW_DIR)" != "$(COV_PROFRAW_DIR)"
	@case "$(abspath $(TEST_TARGET_DIR))" in "$(abspath $(COV_TARGET_DIR))"/*) echo "TEST_TARGET_DIR is nested in COV_TARGET_DIR"; exit 1;; esac
	@case "$(abspath $(COV_TARGET_DIR))" in "$(abspath $(TEST_TARGET_DIR))"/*) echo "COV_TARGET_DIR is nested in TEST_TARGET_DIR"; exit 1;; esac

test-coverage-parallel:
	$(MAKE) -j2 test coverage
	$(MAKE) verify-parallel-isolation

clean-isolate:
	@rm -rf target-isolate-test target-isolate-cov

snapshots:
	$(TEST_ENV) cargo insta test --workspace

snapshots-accept:
	$(TEST_ENV) cargo insta accept --workspace

snapshots-review:
	$(TEST_ENV) cargo insta review

.PHONY: fmt lint test audit coverage ci check ci-local test-coverage-parallel verify-parallel-isolation \
		fmt-isolate lint-isolate test-isolate audit-isolate coverage-isolate ci-isolate clean-isolate \
		snapshots snapshots-accept snapshots-review ensure-cargo-deny
