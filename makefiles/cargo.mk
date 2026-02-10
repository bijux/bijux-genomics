FMT = cargo fmt --all -- --check
CARGO_BUILD_JOBS ?= $(JOBS)
LINT = CARGO_BUILD_JOBS=$(CARGO_BUILD_JOBS) cargo clippy --workspace --all-targets --all-features -- -D warnings
AUDIT = cargo deny check
NEXTEST_PROFILE ?= ci
NEXTEST_CONFIG ?= --config-file nextest.toml
RUN_IGNORED = --run-ignored all
TEST_FEATURES = --all-features
NEXTEST_TEST_THREADS ?= $(CARGO_BUILD_JOBS)
TEST_TARGET_DIR ?= target-test
COV_TARGET_DIR ?= target-cov
TEST_TMP_DIR ?= $(abspath $(TEST_TARGET_DIR))/tmp
COV_TMP_DIR ?= $(abspath $(COV_TARGET_DIR))/tmp
TEST_PROFRAW_DIR ?= $(abspath $(TEST_TARGET_DIR))/profraw
COV_PROFRAW_DIR ?= $(abspath $(COV_TARGET_DIR))/profraw
TEST_CARGO_HOME ?= $(abspath $(TEST_TARGET_DIR))/cargo-home
COV_CARGO_HOME ?= $(abspath $(COV_TARGET_DIR))/cargo-home
TEST_ENV = TZ=UTC LC_ALL=C TMPDIR=$(TEST_TMP_DIR) TMP=$(TEST_TMP_DIR) TEMP=$(TEST_TMP_DIR) \
  TEST_TARGET_DIR=$(TEST_TARGET_DIR) COV_TARGET_DIR=$(COV_TARGET_DIR) \
  TEST_TMP_DIR=$(TEST_TMP_DIR) COV_TMP_DIR=$(COV_TMP_DIR) \
  TEST_PROFRAW_DIR=$(TEST_PROFRAW_DIR) COV_PROFRAW_DIR=$(COV_PROFRAW_DIR) TEST_CARGO_HOME=$(TEST_CARGO_HOME) COV_CARGO_HOME=$(COV_CARGO_HOME) \
  LLVM_PROFILE_FILE=$(TEST_PROFRAW_DIR)/%p.profraw
TEST = $(TEST_ENV) CARGO_HOME=$(TEST_CARGO_HOME) CARGO_TARGET_DIR=$(TEST_TARGET_DIR) cargo nextest run $(NEXTEST_CONFIG) --workspace $(TEST_FEATURES) --profile $(NEXTEST_PROFILE) --test-threads $(NEXTEST_TEST_THREADS) $(RUN_IGNORED)
COVERAGE_ROOT = $(COV_TARGET_DIR)/coverage
COVERAGE_ROOT_ABS = $(abspath $(COVERAGE_ROOT))
COV_TARGET_DIR_ABS = $(abspath $(COV_TARGET_DIR))
COVERAGE_OUT = $(COVERAGE_ROOT)/coverage.json
HTML_OUT = $(COVERAGE_ROOT)/html
COVERAGE_BASELINE = artifacts/coverage/baseline.json
COVERAGE_THRESHOLDS = configs/coverage.toml
COVERAGE_ENV = TZ=UTC LC_ALL=C TMPDIR=$(COV_TMP_DIR) TMP=$(COV_TMP_DIR) TEMP=$(COV_TMP_DIR) \
  TEST_TARGET_DIR=$(TEST_TARGET_DIR) COV_TARGET_DIR=$(COV_TARGET_DIR) \
  TEST_TMP_DIR=$(TEST_TMP_DIR) COV_TMP_DIR=$(COV_TMP_DIR) \
  TEST_PROFRAW_DIR=$(TEST_PROFRAW_DIR) COV_PROFRAW_DIR=$(COV_PROFRAW_DIR) TEST_CARGO_HOME=$(TEST_CARGO_HOME) COV_CARGO_HOME=$(COV_CARGO_HOME) \
  CARGO_HOME=$(COV_CARGO_HOME) \
  CARGO_TARGET_DIR=$(COV_TARGET_DIR) \
  CARGO_LLVM_COV_TARGET_DIR=$(COV_TARGET_DIR_ABS) \
  CARGO_LLVM_COV_BUILD_DIR=$(COV_TARGET_DIR_ABS) \
  LLVM_PROFILE_FILE=$(COV_PROFRAW_DIR)/%p.profraw
COVERAGE_RUN = cargo llvm-cov nextest --no-report --no-cfg-coverage $(NEXTEST_CONFIG) --workspace $(TEST_FEATURES) --profile $(NEXTEST_PROFILE) --test-threads $(NEXTEST_TEST_THREADS) $(RUN_IGNORED)
COVERAGE_JSON = cargo llvm-cov report --json --output-path $(COVERAGE_OUT)
COVERAGE_HTML = cargo llvm-cov report --html --output-dir $(COVERAGE_ROOT)
GIT_SHORT_SHA ?= $(shell git rev-parse --short HEAD 2>/dev/null || echo nogit)
ISOLATE_ID ?= $(shell sh -c 'date -u +%Y%m%d%H%M%S-$$PPID-$(GIT_SHORT_SHA)')
ISOLATE_ROOT ?= artifacts/isolates/$(ISOLATE_ID)
ISOLATE_TEST_TARGET_DIR ?= $(ISOLATE_ROOT)/target-test
ISOLATE_COV_TARGET_DIR ?= $(ISOLATE_ROOT)/target-cov
ISOLATE_TEST_CARGO_HOME ?= $(ISOLATE_ROOT)/cargo-home-test
ISOLATE_COV_CARGO_HOME ?= $(ISOLATE_ROOT)/cargo-home-cov

fmt:
	$(FMT)

lint: domain-validate domain-inventory-drift check-generated-configs check-generated-config-headers
	./scripts/check-artifacts-tracked.sh
	./scripts/check-no-target-paths-in-tests.sh
	$(LINT)

test:
	@rm -rf $(TEST_PROFRAW_DIR)
	@mkdir -p $(TEST_TMP_DIR)
	@mkdir -p $(TEST_PROFRAW_DIR)
	@mkdir -p $(TEST_CARGO_HOME)
	$(TEST)
	./scripts/check-isolation-contract.sh

ensure-cargo-deny:
	@command -v cargo-deny >/dev/null 2>&1 || cargo install cargo-deny

audit: ensure-cargo-deny
	$(AUDIT)

coverage:
	@$(COVERAGE_ENV) cargo llvm-cov clean
	@rm -rf $(COVERAGE_ROOT)
	@mkdir -p $(COVERAGE_ROOT)
	@rm -rf $(COV_PROFRAW_DIR)
	@mkdir -p $(COV_TMP_DIR)
	@mkdir -p $(COV_PROFRAW_DIR)
	@mkdir -p $(COV_CARGO_HOME)
	$(COVERAGE_ENV) $(COVERAGE_RUN)
	$(COVERAGE_ENV) $(COVERAGE_JSON)
	$(COVERAGE_ENV) $(COVERAGE_HTML)
	@test -f $(COVERAGE_OUT)
	@test -d $(HTML_OUT)
	@test -f $(HTML_OUT)/index.html
	@if [ -f $(COVERAGE_BASELINE) ]; then \
		python3 scripts/coverage_summary.py $(COVERAGE_OUT) --baseline $(COVERAGE_BASELINE) --check-thresholds $(COVERAGE_THRESHOLDS); \
	else \
		python3 scripts/coverage_summary.py $(COVERAGE_OUT) --check-thresholds $(COVERAGE_THRESHOLDS); \
	fi

fmt-isolate:
	@ISO=$$(date -u +%Y%m%d%H%M%S)-$$$$-$(GIT_SHORT_SHA); \
	ROOT=artifacts/isolates/$$ISO; \
	TEST_TARGET_DIR=$$ROOT/target-test COV_TARGET_DIR=$$ROOT/target-cov \
	TEST_CARGO_HOME=$$ROOT/cargo-home-test COV_CARGO_HOME=$$ROOT/cargo-home-cov \
	CARGO_HOME=$$ROOT/cargo-home-test \
	$(MAKE) fmt

lint-isolate:
	@ISO=$$(date -u +%Y%m%d%H%M%S)-$$$$-$(GIT_SHORT_SHA); \
	ROOT=artifacts/isolates/$$ISO; \
	TEST_TARGET_DIR=$$ROOT/target-test COV_TARGET_DIR=$$ROOT/target-cov \
	TEST_CARGO_HOME=$$ROOT/cargo-home-test COV_CARGO_HOME=$$ROOT/cargo-home-cov \
	CARGO_HOME=$$ROOT/cargo-home-test \
	$(MAKE) lint

test-isolate:
	@ISO=$$(date -u +%Y%m%d%H%M%S)-$$$$-$(GIT_SHORT_SHA); \
	ROOT=artifacts/isolates/$$ISO; \
	TEST_TARGET_DIR=$$ROOT/target-test COV_TARGET_DIR=$$ROOT/target-cov \
	TEST_CARGO_HOME=$$ROOT/cargo-home-test COV_CARGO_HOME=$$ROOT/cargo-home-cov \
	$(MAKE) test

audit-isolate: ensure-cargo-deny
	@ISO=$$(date -u +%Y%m%d%H%M%S)-$$$$-$(GIT_SHORT_SHA); \
	ROOT=artifacts/isolates/$$ISO; \
	TEST_TARGET_DIR=$$ROOT/target-test COV_TARGET_DIR=$$ROOT/target-cov \
	TEST_CARGO_HOME=$$ROOT/cargo-home-test COV_CARGO_HOME=$$ROOT/cargo-home-cov \
	CARGO_HOME=$$ROOT/cargo-home-test \
	$(MAKE) audit

coverage-isolate:
	@ISO=$$(date -u +%Y%m%d%H%M%S)-$$$$-$(GIT_SHORT_SHA); \
	ROOT=artifacts/isolates/$$ISO; \
	TEST_TARGET_DIR=$$ROOT/target-test COV_TARGET_DIR=$$ROOT/target-cov \
	TEST_CARGO_HOME=$$ROOT/cargo-home-test COV_CARGO_HOME=$$ROOT/cargo-home-cov \
	$(MAKE) coverage

ci:
	$(MAKE) fmt-isolate
	$(MAKE) lint-isolate
	$(MAKE) audit-isolate
	$(MAKE) test-isolate
	$(MAKE) docs-isolate
	./scripts/check-root-pollution.sh

check:
	$(MAKE) fmt lint audit coverage

ci-isolate:
	TEST_TARGET_DIR=$(ISOLATE_TEST_TARGET_DIR) COV_TARGET_DIR=$(ISOLATE_COV_TARGET_DIR) $(MAKE) ci

test-coverage-isolate-parallel:
	$(MAKE) -j2 test-isolate coverage-isolate

ci-local:
	$(MAKE) -j2 test coverage

verify-parallel-isolation:
	@test "$(TEST_TARGET_DIR)" != "$(COV_TARGET_DIR)"
	@test "$(TEST_TMP_DIR)" != "$(COV_TMP_DIR)"
	@test "$(TEST_PROFRAW_DIR)" != "$(COV_PROFRAW_DIR)"
	@case "$(abspath $(TEST_TARGET_DIR))" in "$(abspath $(COV_TARGET_DIR))"/*) echo "TEST_TARGET_DIR is nested in COV_TARGET_DIR"; exit 1;; esac
	@case "$(abspath $(COV_TARGET_DIR))" in "$(abspath $(TEST_TARGET_DIR))"/*) echo "COV_TARGET_DIR is nested in TEST_TARGET_DIR"; exit 1;; esac

test-coverage-parallel:
test-and-coverage: verify-parallel-isolation test coverage
	@# Cross-footprint checks: test must not emit coverage outputs, coverage must not emit test outputs.
	@test ! -e $(TEST_TARGET_DIR)/coverage/coverage.json
	@test ! -e $(TEST_TARGET_DIR)/coverage/html/index.html
	@test ! -e $(COV_TARGET_DIR)/run_manifest.json

test-coverage-parallel: test-and-coverage

clean-isolates:
	@rm -rf artifacts/isolates/*

policy-fast: ## Run fast policy checks (no snapshots)
	cargo test -p bijux-dna-policies --test dependency_graph --test purity_scans --test core_layering --test domain_dependency_policy --test ci_tools_policy --test dev_deps_policy --test heavy_deps_policy
	./scripts/domain-validate.sh
	./scripts/domain-inventory-drift.sh

policy-full: ## Run full policy suite
	cargo test -p bijux-dna-policies
	./scripts/domain-validate.sh
	./scripts/domain-inventory-drift.sh

domain-validate:
	./scripts/domain-validate.sh

domain-inventory-drift:
	./scripts/domain-inventory-drift.sh

snapshots:
	$(TEST_ENV) cargo insta test --workspace

snapshots-accept:
	$(TEST_ENV) cargo insta accept --workspace

snapshots-review:
	$(TEST_ENV) cargo insta review

	.PHONY: fmt lint test audit coverage ci check ci-local test-coverage-parallel verify-parallel-isolation \
		test-and-coverage \
		test-coverage-isolate-parallel \
		fmt-isolate lint-isolate test-isolate audit-isolate coverage-isolate ci-isolate clean-isolates \
		domain-validate domain-inventory-drift generate-configs check-generated-configs check-generated-config-headers \
		policy-fast policy-full \
		snapshots snapshots-accept snapshots-review ensure-cargo-deny
generate-configs:
	cargo run -p bijux-dna-domain-compiler --bin compile_domain_configs -- --domain-dir domain --configs-dir configs

check-generated-configs:
	./scripts/check-generated-configs.sh

check-generated-config-headers:
	./scripts/check-generated-config-headers.sh
