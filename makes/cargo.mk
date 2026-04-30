NEXTEST_PROFILE ?= full
NEXTEST_PROFILE_FAST ?= fast-unit
NEXTEST_PROFILE_SLOW ?= slow-integration
NEXTEST_PROFILE_CERT ?= certification
NEXTEST_PROFILE_ALL ?= full
ARTIFACTS_DIR ?= $(ARTIFACT_ROOT)/make/$(or $(MAKECMDGOALS),manual)
NEXTEST_TOML := configs/rust/nextest.toml
NEXTEST_CONFIG ?= --config-file $(NEXTEST_TOML)
NEXTEST_FAST_EXPR ?= not test(/::slow__/)
NEXTEST_SLOW_EXPR ?= test(/::slow__/)
NEXTEST_NO_TESTS ?= pass
RUN_IGNORED = --run-ignored all
TEST_FEATURES = --all-features
CARGO_BUILD_JOBS ?= $(JOBS)
NEXTEST_TEST_THREADS ?= $(CARGO_BUILD_JOBS)
LINT_PARALLEL_JOBS ?= $(if $(CARGO_BUILD_JOBS),$(CARGO_BUILD_JOBS),8)
LINT_PARALLEL_COMMANDS_FILE ?= makes/lint.parallel.commands.txt
COVERAGE_BASELINE = artifacts/coverage/baseline.json
COVERAGE_THRESHOLDS := configs/coverage/thresholds.toml
COVERAGE_OUT = coverage.json
DEV_DNA_BIN ?= $(CARGO_TARGET_DIR)/debug/bijux-dna-dev
DEV_DNA_BOOTSTRAP ?= makes/bin/dev_dna_bootstrap.sh
RUST_GATE_BIN ?= makes/bin/rust_gate.sh
RS_ARTIFACT_ROOT ?= $(ARTIFACT_ROOT)/rust
RS_RUN_ID ?= local
RS_TARGET_DIR ?= $(abspath $(RS_ARTIFACT_ROOT)/target)
RS_NEXTEST_CACHE_DIR ?= $(RS_TARGET_DIR)/nextest
RS_NEXTEST_CONFIG_HOME ?= $(abspath $(RS_ARTIFACT_ROOT)/nextest/config)
RS_PROFRAW_DIR ?= $(abspath $(RS_ARTIFACT_ROOT)/coverage/profraw)
RS_LLVM_PROFILE_FILE ?= $(abspath $(RS_PROFRAW_DIR)/default_%m_%p.profraw)
RS_COVERAGE_TARGET_DIR ?= $(abspath $(RS_ARTIFACT_ROOT)/coverage/target)
RS_FMT_REPORT ?= $(RS_ARTIFACT_ROOT)/fmt/$(RS_RUN_ID)/report.txt
RS_LINT_REPORT ?= $(RS_ARTIFACT_ROOT)/lint/$(RS_RUN_ID)/report.txt
RS_TEST_REPORT ?= $(RS_ARTIFACT_ROOT)/test/$(RS_RUN_ID)/nextest.log
RS_TEST_SLOW_REPORT ?= $(RS_ARTIFACT_ROOT)/test/$(RS_RUN_ID)/nextest-slow.log
RS_TEST_ALL_REPORT ?= $(RS_ARTIFACT_ROOT)/test/$(RS_RUN_ID)/nextest-all.log
RS_AUDIT_REPORT ?= $(RS_ARTIFACT_ROOT)/audit/$(RS_RUN_ID)/report.txt
RS_COVERAGE_DIR ?= $(RS_ARTIFACT_ROOT)/coverage/$(RS_RUN_ID)
RS_LCOV_FILE ?= $(RS_COVERAGE_DIR)/lcov.info
RS_COVERAGE_TEST_REPORT ?= $(RS_COVERAGE_DIR)/nextest.log
RS_COVERAGE_SUMMARY_REPORT ?= $(RS_COVERAGE_DIR)/summary.txt
RS_CLIPPY_EXCLUDES ?= bijux-dna-dev
NEXTEST_STATUS_LEVEL ?= all
NEXTEST_FINAL_STATUS_LEVEL ?= all

fmt:
	@$(ensure_artifact_env)
	@$(MAKE) fmt-rs

_dev-dna-bin:
	@$(ensure_artifact_env)
	@$(DEV_DNA_BOOTSTRAP) "$(DEV_DNA_BIN)"

fmt-rs: ## Run Rust formatting checks.
	@$(ensure_artifact_env)
	@RS_ARTIFACT_ROOT="$(RS_ARTIFACT_ROOT)" RS_RUN_ID="$(RS_RUN_ID)" RS_TARGET_DIR="$(RS_TARGET_DIR)" RS_FMT_REPORT="$(RS_FMT_REPORT)" CARGO_TERM_COLOR="$(CARGO_TERM_COLOR)" CARGO_TERM_PROGRESS_WHEN="$(CARGO_TERM_PROGRESS_WHEN)" CARGO_TERM_PROGRESS_WIDTH="$(CARGO_TERM_PROGRESS_WIDTH)" CARGO_TERM_VERBOSE="$(CARGO_TERM_VERBOSE)" "$(RUST_GATE_BIN)" fmt

_fmt:
	@$(ensure_artifact_env)
	@$(MAKE) fmt-rs

lint:
	@$(ensure_artifact_env)
	@$(MAKE) lint-rs

lint-rs: ## Run Rust clippy checks with deny-warnings.
	@$(ensure_artifact_env)
	@RS_ARTIFACT_ROOT="$(RS_ARTIFACT_ROOT)" RS_RUN_ID="$(RS_RUN_ID)" RS_TARGET_DIR="$(RS_TARGET_DIR)" RS_LINT_REPORT="$(RS_LINT_REPORT)" RS_CLIPPY_EXCLUDES="$(RS_CLIPPY_EXCLUDES)" CARGO_TERM_COLOR="$(CARGO_TERM_COLOR)" CARGO_TERM_PROGRESS_WHEN="$(CARGO_TERM_PROGRESS_WHEN)" CARGO_TERM_PROGRESS_WIDTH="$(CARGO_TERM_PROGRESS_WIDTH)" CARGO_TERM_VERBOSE="$(CARGO_TERM_VERBOSE)" "$(RUST_GATE_BIN)" lint

lint-workspace: ## Run Rust lint plus workspace config/docs/automation policy gates.
	@$(ensure_artifact_env)
	@$(MAKE) _lint

_lint:
	@$(MAKE) _lint-rustfmt
	@$(MAKE) _lint-configs
	@$(MAKE) _lint-docs
	@$(MAKE) _lint-automation
	@$(MAKE) _lint-clippy

_lint-rustfmt:
	@$(ensure_artifact_env)
	@$(MAKE) _dev-dna-bin >/dev/null
	@$(DEV_DNA_BIN) tooling run ci-fmt

_lint-configs:
	@$(ensure_artifact_env)
	@$(MAKE) _dev-dna-bin >/dev/null
	@$(DEV_DNA_BIN) checks run check-config-schema
	@$(DEV_DNA_BIN) checks run check-config-layout
	@$(DEV_DNA_BIN) checks run check-generated-configs
	@$(DEV_DNA_BIN) checks run check-generated-config-headers

_lint-docs:
	@$(ensure_artifact_env)
	@$(MAKE) _dev-dna-bin >/dev/null
	@$(DEV_DNA_BIN) docs run check-doc-links
	@$(DEV_DNA_BIN) checks run check-docs-build-contract

_lint-automation:
	@$(ensure_artifact_env)
	@$(MAKE) _dev-dna-bin >/dev/null
	$(DEV_DNA_BIN) tooling run repo-doctor --fast
	@rm -rf "$(ARTIFACTS_DIR)/lint-parallel"
	@mkdir -p "$(ARTIFACTS_DIR)/lint-parallel"
	@cp "$(LINT_PARALLEL_COMMANDS_FILE)" "$(ARTIFACTS_DIR)/lint-parallel/commands.txt"
	@echo "Running automation lint gates in parallel (jobs=$(LINT_PARALLEL_JOBS)); logs: $(ARTIFACTS_DIR)/lint-parallel"
	@while IFS= read -r cmd; do printf '%s\0' "$$cmd"; done < "$(ARTIFACTS_DIR)/lint-parallel/commands.txt" \
	| xargs -0 -n1 -P "$(LINT_PARALLEL_JOBS)" sh -c '\
		cmd="$$2"; \
		name=$$(printf "%s" "$$cmd" | tr -cs "[:alnum:]._-" "_"); \
		log_file="$$1/$$name.log"; \
		if sh -c "$$cmd" >"$$log_file" 2>&1; then \
			printf "ok %s\n" "$$cmd"; \
		else \
			printf "FAILED %s\n" "$$cmd" >&2; \
			tail -n 80 "$$log_file" >&2; \
			exit 1; \
		fi' sh "$(ARTIFACTS_DIR)/lint-parallel"
	@find "$(ARTIFACTS_DIR)/lint-parallel" -type f -name '._*' -delete

lint-automation: ## Run repo-doctor + automation/container lint checks (parallelized), without clippy.
	@$(ensure_artifact_env)
	@$(MAKE) _lint-automation

lint-scripts: ## Compatibility alias for lint-automation.
	@$(ensure_artifact_env)
	@$(MAKE) lint-automation

lint-rustfmt: ## Run rustfmt gate only.
	@$(ensure_artifact_env)
	@$(MAKE) _lint-rustfmt

lint-clippy: ## Run clippy gate only.
	@$(ensure_artifact_env)
	@$(MAKE) _lint-clippy

lint-docs: ## Run docs lint gates only.
	@$(ensure_artifact_env)
	@$(MAKE) _lint-docs

lint-configs: ## Run config/schema lint gates only.
	@$(ensure_artifact_env)
	@$(MAKE) _lint-configs

lint-fast: ## Run lint checks relevant to changed paths only.
	@$(ensure_artifact_env)
	@$(MAKE) _dev-dna-bin >/dev/null
	@$(DEV_DNA_BIN) tooling run lint-fast

_lint-clippy:
	@$(ensure_artifact_env)
	@$(MAKE) _dev-dna-bin >/dev/null
	@CARGO_BUILD_JOBS="$(CARGO_BUILD_JOBS)" $(DEV_DNA_BIN) tooling run ci-clippy

_lint-clippy-executors:
	@$(ensure_artifact_env)
	@$(MAKE) _dev-dna-bin >/dev/null
	@CARGO_BUILD_JOBS="$(CARGO_BUILD_JOBS)" $(DEV_DNA_BIN) tooling run ci-clippy-executors

_clippy: ## Run workspace clippy only (no automation gates).
	@$(MAKE) _lint-clippy

_clippy-executors: ## Run deny-warnings clippy for runner/executor crates.
	@$(MAKE) _lint-clippy-executors

test:
	@$(ensure_artifact_env)
	@$(MAKE) test-rs

test-rs: ## Run Rust fast suite, exclude slow-labeled tests, and enforce a 10s per-test budget.
	@$(ensure_artifact_env)
	@RS_ARTIFACT_ROOT="$(RS_ARTIFACT_ROOT)" RS_RUN_ID="$(RS_RUN_ID)" RS_TARGET_DIR="$(RS_TARGET_DIR)" RS_NEXTEST_CACHE_DIR="$(RS_NEXTEST_CACHE_DIR)" RS_NEXTEST_CONFIG_HOME="$(RS_NEXTEST_CONFIG_HOME)" RS_PROFRAW_DIR="$(RS_PROFRAW_DIR)" RS_LLVM_PROFILE_FILE="$(RS_LLVM_PROFILE_FILE)" RS_TEST_REPORT="$(RS_TEST_REPORT)" NEXTEST_CONFIG_FILE="$(NEXTEST_TOML)" NEXTEST_PROFILE_FAST="$(NEXTEST_PROFILE_FAST)" NEXTEST_FAST_EXPR="$(NEXTEST_FAST_EXPR)" NEXTEST_STATUS_LEVEL="$(NEXTEST_STATUS_LEVEL)" NEXTEST_FINAL_STATUS_LEVEL="$(NEXTEST_FINAL_STATUS_LEVEL)" CARGO_TERM_COLOR="$(CARGO_TERM_COLOR)" CARGO_TERM_PROGRESS_WHEN="$(CARGO_TERM_PROGRESS_WHEN)" CARGO_TERM_PROGRESS_WIDTH="$(CARGO_TERM_PROGRESS_WIDTH)" CARGO_TERM_VERBOSE="$(CARGO_TERM_VERBOSE)" "$(RUST_GATE_BIN)" test

test-fast: ## Compatibility alias for the fast Rust suite.
	@$(ensure_artifact_env)
	@$(MAKE) test-rs

test-slow: ## Run Rust tests labeled as slow.
	@$(ensure_artifact_env)
	@$(MAKE) test-slow-rs

test-slow-rs: ## Run Rust slow suite (tests labeled with slow__ or promoted from the 10s fast lane budget).
	@$(ensure_artifact_env)
	@RS_ARTIFACT_ROOT="$(RS_ARTIFACT_ROOT)" RS_RUN_ID="$(RS_RUN_ID)" RS_TARGET_DIR="$(RS_TARGET_DIR)" RS_NEXTEST_CACHE_DIR="$(RS_NEXTEST_CACHE_DIR)" RS_NEXTEST_CONFIG_HOME="$(RS_NEXTEST_CONFIG_HOME)" RS_PROFRAW_DIR="$(RS_PROFRAW_DIR)" RS_LLVM_PROFILE_FILE="$(RS_LLVM_PROFILE_FILE)" RS_TEST_SLOW_REPORT="$(RS_TEST_SLOW_REPORT)" NEXTEST_CONFIG_FILE="$(NEXTEST_TOML)" NEXTEST_PROFILE_SLOW="$(NEXTEST_PROFILE_SLOW)" NEXTEST_SLOW_EXPR="$(NEXTEST_SLOW_EXPR)" NEXTEST_STATUS_LEVEL="$(NEXTEST_STATUS_LEVEL)" NEXTEST_FINAL_STATUS_LEVEL="$(NEXTEST_FINAL_STATUS_LEVEL)" CARGO_TERM_COLOR="$(CARGO_TERM_COLOR)" CARGO_TERM_PROGRESS_WHEN="$(CARGO_TERM_PROGRESS_WHEN)" CARGO_TERM_PROGRESS_WIDTH="$(CARGO_TERM_PROGRESS_WIDTH)" CARGO_TERM_VERBOSE="$(CARGO_TERM_VERBOSE)" "$(RUST_GATE_BIN)" test-slow

test-all: ## Run the full Rust suite, including ignored tests.
	@$(ensure_artifact_env)
	@$(MAKE) test-all-rs

test-all-rs: ## Run the full Rust suite, including ignored and long-running tests.
	@$(ensure_artifact_env)
	@RS_ARTIFACT_ROOT="$(RS_ARTIFACT_ROOT)" RS_RUN_ID="$(RS_RUN_ID)" RS_TARGET_DIR="$(RS_TARGET_DIR)" RS_NEXTEST_CACHE_DIR="$(RS_NEXTEST_CACHE_DIR)" RS_NEXTEST_CONFIG_HOME="$(RS_NEXTEST_CONFIG_HOME)" RS_PROFRAW_DIR="$(RS_PROFRAW_DIR)" RS_LLVM_PROFILE_FILE="$(RS_LLVM_PROFILE_FILE)" RS_TEST_ALL_REPORT="$(RS_TEST_ALL_REPORT)" NEXTEST_CONFIG_FILE="$(NEXTEST_TOML)" NEXTEST_PROFILE_ALL="$(NEXTEST_PROFILE_ALL)" NEXTEST_STATUS_LEVEL="$(NEXTEST_STATUS_LEVEL)" NEXTEST_FINAL_STATUS_LEVEL="$(NEXTEST_FINAL_STATUS_LEVEL)" CARGO_TERM_COLOR="$(CARGO_TERM_COLOR)" CARGO_TERM_PROGRESS_WHEN="$(CARGO_TERM_PROGRESS_WHEN)" CARGO_TERM_PROGRESS_WIDTH="$(CARGO_TERM_PROGRESS_WIDTH)" CARGO_TERM_VERBOSE="$(CARGO_TERM_VERBOSE)" "$(RUST_GATE_BIN)" test-all

_test:
	@$(ensure_artifact_env)
	@$(MAKE) _dev-dna-bin >/dev/null
	@NEXTEST_CONFIG="$(NEXTEST_CONFIG)" TEST_FEATURES="$(TEST_FEATURES)" NEXTEST_PROFILE="$(NEXTEST_PROFILE)" NEXTEST_TEST_THREADS="$(NEXTEST_TEST_THREADS)" NEXTEST_NO_TESTS="$(NEXTEST_NO_TESTS)" RUN_IGNORED="$(RUN_IGNORED)" $(DEV_DNA_BIN) tooling run ci-test

_test-fast: ## Run fast test suite excluding only slow-labeled tests.
	@$(ensure_artifact_env)
	@$(MAKE) _dev-dna-bin >/dev/null
	@NEXTEST_CONFIG="$(NEXTEST_CONFIG)" TEST_FEATURES="$(TEST_FEATURES)" NEXTEST_PROFILE="$(NEXTEST_PROFILE_FAST)" NEXTEST_TEST_THREADS="$(NEXTEST_TEST_THREADS)" NEXTEST_NO_TESTS="$(NEXTEST_NO_TESTS)" RUN_IGNORED="$(RUN_IGNORED)" NEXTEST_FAST_EXPR="$(NEXTEST_FAST_EXPR)" $(DEV_DNA_BIN) tooling run ci-test

_test-slow: ## Run only slow-labeled tests (functions containing slow__).
	@$(ensure_artifact_env)
	@$(MAKE) _dev-dna-bin >/dev/null
	@NEXTEST_CONFIG="$(NEXTEST_CONFIG)" TEST_FEATURES="$(TEST_FEATURES)" NEXTEST_PROFILE="$(NEXTEST_PROFILE_SLOW)" NEXTEST_TEST_THREADS="$(NEXTEST_TEST_THREADS)" NEXTEST_NO_TESTS="$(NEXTEST_NO_TESTS)" RUN_IGNORED="$(RUN_IGNORED)" $(DEV_DNA_BIN) tooling run ci-test-slow

audit:
	@$(ensure_artifact_env)
	@$(MAKE) audit-rs

audit-rs: ## Run Rust advisory and license audits.
	@$(ensure_artifact_env)
	@RS_ARTIFACT_ROOT="$(RS_ARTIFACT_ROOT)" RS_RUN_ID="$(RS_RUN_ID)" RS_TARGET_DIR="$(RS_TARGET_DIR)" RS_AUDIT_REPORT="$(RS_AUDIT_REPORT)" CARGO_TERM_COLOR="$(CARGO_TERM_COLOR)" CARGO_TERM_PROGRESS_WHEN="$(CARGO_TERM_PROGRESS_WHEN)" CARGO_TERM_PROGRESS_WIDTH="$(CARGO_TERM_PROGRESS_WIDTH)" CARGO_TERM_VERBOSE="$(CARGO_TERM_VERBOSE)" "$(RUST_GATE_BIN)" audit

_audit:
	@$(ensure_artifact_env)
	@cargo run -q -p bijux-dna-dev -- tooling run ci-audit

coverage:
	@$(ensure_artifact_env)
	@$(MAKE) coverage-rs

coverage-rs: ## Run Rust coverage with llvm-cov and emit reports.
	@$(ensure_artifact_env)
	@RS_ARTIFACT_ROOT="$(RS_ARTIFACT_ROOT)" RS_RUN_ID="$(RS_RUN_ID)" RS_NEXTEST_CACHE_DIR="$(RS_NEXTEST_CACHE_DIR)" RS_NEXTEST_CONFIG_HOME="$(RS_NEXTEST_CONFIG_HOME)" RS_PROFRAW_DIR="$(RS_PROFRAW_DIR)" RS_LLVM_PROFILE_FILE="$(RS_LLVM_PROFILE_FILE)" RS_COVERAGE_TARGET_DIR="$(RS_COVERAGE_TARGET_DIR)" RS_COVERAGE_DIR="$(RS_COVERAGE_DIR)" RS_LCOV_FILE="$(RS_LCOV_FILE)" RS_COVERAGE_TEST_REPORT="$(RS_COVERAGE_TEST_REPORT)" RS_COVERAGE_SUMMARY_REPORT="$(RS_COVERAGE_SUMMARY_REPORT)" NEXTEST_CONFIG_FILE="$(NEXTEST_TOML)" NEXTEST_PROFILE_ALL="$(NEXTEST_PROFILE_ALL)" NEXTEST_STATUS_LEVEL="$(NEXTEST_STATUS_LEVEL)" NEXTEST_FINAL_STATUS_LEVEL="$(NEXTEST_FINAL_STATUS_LEVEL)" CARGO_TERM_COLOR="$(CARGO_TERM_COLOR)" CARGO_TERM_PROGRESS_WHEN="$(CARGO_TERM_PROGRESS_WHEN)" CARGO_TERM_PROGRESS_WIDTH="$(CARGO_TERM_PROGRESS_WIDTH)" CARGO_TERM_VERBOSE="$(CARGO_TERM_VERBOSE)" "$(RUST_GATE_BIN)" coverage

coverage-workspace: ## Run the governed coverage control-plane lane.
	@$(ensure_artifact_env)
	@$(MAKE) _coverage

_coverage:
	@$(ensure_artifact_env)
	@NEXTEST_CONFIG="$(NEXTEST_CONFIG)" TEST_FEATURES="$(TEST_FEATURES)" NEXTEST_PROFILE="$(NEXTEST_PROFILE)" NEXTEST_TEST_THREADS="$(NEXTEST_TEST_THREADS)" RUN_IGNORED="$(RUN_IGNORED)" COVERAGE_OUT="$(COVERAGE_OUT)" COVERAGE_BASELINE="$(COVERAGE_BASELINE)" COVERAGE_THRESHOLDS="$(COVERAGE_THRESHOLDS)" cargo run -q -p bijux-dna-dev -- tooling run ci-coverage

doctor:
	@$(ensure_artifact_env)
	@$(MAKE) _doctor

_doctor:
	@$(ensure_artifact_env)
	@cargo run -q -p bijux-dna-dev -- tooling run repo-doctor --fast
	@cargo run -q -p bijux-dna-dev -- checks run check-legacy-automation-removed
	@cargo run -q -p bijux-dna-dev -- checks run check-config-schema
	@cargo run -q -p bijux-dna-dev -- checks run check-nextest-profile-contract
	@cargo run -q -p bijux-dna-dev -- checks run check-runtime-profiles-contract
	@cargo run -q -p bijux-dna-dev -- checks run check-logging-contract
	@cargo run -q -p bijux-dna-dev -- checks run check-hpc-rsync-docs-parity
	@cargo run -q -p bijux-dna-dev -- checks run check-run-directory-layout
	@cargo run -q -p bijux-dna-dev -- checks run check-registry-required-tools-parity
	@cargo run -q -p bijux-dna-dev -- checks run check-domain-tool-parity
	@cargo run -q -p bijux-dna-dev -- checks run check-stage-domain-parity
	@cargo run -q -p bijux-dna-dev -- checks run check-stage-registry-governance
	@cargo run -q -p bijux-dna-dev -- checks run check-enabled-vcf-panel-metadata
	@cargo run -q -p bijux-dna-dev -- checks run check-param-registry-completeness
	@cargo run -q -p bijux-dna-dev -- checks run check-deprecations-enforcement
	@cargo run -q -p bijux-dna-dev -- checks run check-no-raw-cargo-in-makes

_install-ci-tools: ## Install required cargo tools once per CI job.
	@$(ensure_artifact_env)
	@cargo run -q -p bijux-dna-dev -- tooling run ci-install-tools

_domain-gates: _domain-validate _domain-inventory-drift _check-generated-configs _check-generated-config-headers

ci:
	@$(ensure_artifact_env)
	@cargo run -q -p bijux-dna-dev -- tooling run repo-doctor --fast
	@$(MAKE) fmt lint audit test coverage

_check:
	$(MAKE) fmt lint audit coverage

_verify-artifact-env:
	@$(ensure_artifact_env)
	@test -d "$(CARGO_TARGET_DIR)"
	@test -d "$(CARGO_HOME)"
	@test -d "$(TMPDIR)"
	@test "$(ISO_ROOT)" = "$(abspath $(ARTIFACT_ROOT))"

_clean-artifact-scratch:
	@$(call safe_rm,$(ARTIFACT_ROOT)/tmp)
	@mkdir -p "$(ARTIFACT_ROOT)/tmp"

_policy-fast: ## Run fast policy checks (no snapshots)
	@cargo run -q -p bijux-dna-dev -- tooling run cargo-targets policy-fast
	$(MAKE) _domain-gates

_ssot-policy-fast: ## Fast-fail SSOT and registry policy checks.
	cargo run -q -p bijux-dna-dev -- checks run check-ssot-guardrails
	$(MAKE) _domain-gates
	@cargo run -q -p bijux-dna-dev -- tooling run cargo-targets ssot-policy-fast

_test-profile-invariants: ## Run pipeline profile invariant contract tests.
	@cargo run -q -p bijux-dna-dev -- tooling run cargo-targets test-profile-invariants

_registry-lint: ## Run strict tool registry reproducibility policy checks.
	@cargo run -q -p bijux-dna-dev -- tooling run cargo-targets registry-lint

_unit-contract-fast: ## Fast unit/contract checks for critical crates.
	@cargo run -q -p bijux-dna-dev -- tooling run cargo-targets unit-contract-fast

_release-readiness: ## Block merges on experimental tools, unknown metrics schemas, or floating pins.
	$(MAKE) _registry-lint
	@cargo run -q -p bijux-dna-dev -- tooling run cargo-targets release-readiness

_ci-fast: ## Fast CI tier: unit + contract + registry lint + profile invariants.
	$(MAKE) _ssot-policy-fast
	$(MAKE) fmt
	$(MAKE) lint-workspace
	$(MAKE) _unit-contract-fast
	$(MAKE) _release-readiness
	$(MAKE) _fastq-container-readiness
	$(MAKE) _test-profile-invariants
	$(MAKE) _policy-no-raw-cargo

_ci-slow: ## Slow CI tier (manual): heavier integration checks.
	$(MAKE) _install-ci-tools
	$(MAKE) audit
	$(MAKE) coverage-workspace
	$(MAKE) _docs-contract
	$(MAKE) _domain-gates
	$(MAKE) _release-readiness

_quick: ## Quick local gate: fmt + clippy + unit + invariant tests.
	$(MAKE) fmt
	$(MAKE) lint-workspace
	$(MAKE) _test-profile-invariants
	$(MAKE) _registry-lint

_policy-full: ## Run full policy suite
	@cargo run -q -p bijux-dna-dev -- tooling run cargo-targets policy-full
	$(MAKE) _domain-gates

_domain-validate:
	cargo run -q -p bijux-dna-dev -- domain run validate

domain-validate:
	cargo run -q -p bijux-dna-dev -- domain run validate

_domain-coverage:
	@cargo run -q -p bijux-dna-dev -- tooling run cargo-targets domain-coverage

_domain-inventory-drift:
	cargo run -q -p bijux-dna-dev -- domain run inventory-drift

_snapshots:
	@cargo run -q -p bijux-dna-dev -- tooling run cargo-targets snapshots

_snapshots-accept:
	@cargo run -q -p bijux-dna-dev -- tooling run cargo-targets snapshots-accept

_snapshots-review:
	@cargo run -q -p bijux-dna-dev -- tooling run cargo-targets snapshots-review

_fix-snapshots: ## Rebuild and accept workspace snapshots with the CI insta workflow.
	@cargo run -q -p bijux-dna-dev -- tooling run cargo-targets fix-snapshots

_test-triage: ## Group failed tests from a saved nextest log.
	@cargo run -q -p bijux-dna-dev -- test run test-triage "$(ARTIFACTS_DIR)/test-logs/latest.log"

generate-configs:
	@$(ensure_artifact_env)
	@cargo run -q -p bijux-dna-dev -- tooling run generate-configs

_generate-configs:
	@$(MAKE) generate-configs

_check-generated-configs:
	cargo run -q -p bijux-dna-dev -- checks run check-generated-configs

_check-generated-config-headers:
	cargo run -q -p bijux-dna-dev -- checks run check-generated-config-headers

_policy-no-raw-cargo: ## Fail if raw cargo invocations exist in Make/control-plane surfaces.
	cargo run -q -p bijux-dna-dev -- checks run check-no-raw-cargo-in-makes
	cargo run -q -p bijux-dna-dev -- checks run check-no-raw-cargo-in-automation

flake-hunt: ## Run repeated flake hunt for an expression (EXPR required, RUNS optional).
	@$(ensure_artifact_env)
	@if [ -z "$(EXPR)" ]; then echo "EXPR is required, e.g. make flake-hunt EXPR='test(...)' RUNS=20" >&2; exit 2; fi
	@cargo run -q -p bijux-dna-dev -- tooling run flake-hunt --expr "$(EXPR)" --runs "$(or $(RUNS),20)"

realness-gate: ## Run strict realness checks (placeholder artifacts + planner realization).
	@$(ensure_artifact_env)
	@cargo run -q -p bijux-dna-dev -- checks run check-domain-realization
	@cargo run -q -p bijux-dna-dev -- checks run check-no-fake-artifacts

_policy-index: ## Generate policy index under artifacts/.
	@cargo run -q -p bijux-dna-dev -- tooling run generate-policy-index

_policy-only-fast-gate: ## Compile+run policies and critical contract crates only.
	@cargo run -q -p bijux-dna-dev -- tooling run cargo-targets policy-only-fast-gate

gate-essential: ## Fast essential architecture + contract gate for local iteration work.
	@$(ensure_artifact_env)
	@cargo run -q -p bijux-dna-dev -- tooling run cargo-targets essential-integrity

gate-execute: ## Fast governed local execute/dry-run/status/replay gate.
	@$(ensure_artifact_env)
	@cargo run -q -p bijux-dna-dev -- tooling run cargo-targets essential-execute

_control-plane-inventory: ## Generate control-plane inventory under artifacts/
	@cargo run -q -p bijux-dna-dev -- tooling run inventory

_config-inventory: ## Generate config inventory under artifacts/
	@cargo run -q -p bijux-dna-dev -- tooling run config-inventory

_smoke-fastq: ## Quick local FASTQ smoke dry-run.
	@cargo run -q -p bijux-dna-dev -- smoke run run fastq

_smoke-bam: ## Quick local BAM smoke dry-run.
	@cargo run -q -p bijux-dna-dev -- smoke run run bam

local-certification-gate: ## Run local mini-domain certification suite and emit bundle.
	@$(ensure_artifact_env)
	@cargo run -q -p bijux-dna-dev -- tooling run certification-gate

vcf-certification: ## Local-only VCF certification run (sequential VCF stage contract suite).
	@$(ensure_artifact_env)
	@NEXTEST_PROFILE="$(NEXTEST_PROFILE_CERT)" cargo run -q -p bijux-dna-dev -- tooling run cargo-targets vcf-certification

certify-fastq: ## Local FASTQ certification smoke.
	@$(ensure_artifact_env)
	@cargo run -q -p bijux-dna-dev -- tooling run certify-fastq

certify-bam: ## Local BAM certification smoke.
	@$(ensure_artifact_env)
	@cargo run -q -p bijux-dna-dev -- tooling run certify-bam

certify-vcf: ## Local VCF certification suite.
	@$(ensure_artifact_env)
	@cargo run -q -p bijux-dna-dev -- tooling run certify-vcf

certify-all: ## Local cross-domain certification bundle (FASTQ+BAM+VCF downstream mini).
	@$(ensure_artifact_env)
	@cargo run -q -p bijux-dna-dev -- tooling run certify-all

examples-validate:
	@$(MAKE) _examples-validate

_examples-validate:
	cargo run -q -p bijux-dna-dev -- checks run check-examples-structure
	cargo run -q -p bijux-dna-dev -- checks run check-examples-index-ssot
	cargo run -q -p bijux-dna-dev -- checks run check-examples-corpus-manifests
	cargo run -q -p bijux-dna-dev -- checks run check-examples-corpus-checksums
	cargo run -q -p bijux-dna-dev -- checks run check-examples-corpus-layout
	cargo run -q -p bijux-dna-dev -- checks run check-examples-golden
	cargo run -q -p bijux-dna-dev -- checks run check-examples-runner-contract
	cargo run -q -p bijux-dna-dev -- checks run check-examples-cli-snapshot
	cargo run -q -p bijux-dna-dev -- checks run check-examples-notebook-policy
	cargo run -q -p bijux-dna-dev -- checks run check-examples-policy

refresh-assets-toy: ## Regenerate deterministic toy datasets in assets/toy.
	@cargo run -q -p bijux-dna-dev -- assets run refresh-toy

refresh-assets-golden: ## Regenerate deterministic toy-run goldens in assets/golden.
	@cargo run -q -p bijux-dna-dev -- assets run refresh-golden

.PHONY: fmt fmt-rs lint lint-rs lint-workspace lint-rustfmt lint-clippy lint-docs lint-configs lint-fast lint-automation lint-scripts test test-rs test-fast test-slow test-slow-rs test-all test-all-rs audit audit-rs coverage coverage-rs coverage-workspace ci doctor _check _verify-artifact-env \
		_clean-artifact-scratch \
		_domain-gates domain-validate examples-validate \
		_examples-validate \
		_domain-validate _domain-coverage _domain-inventory-drift _generate-configs _check-generated-configs _check-generated-config-headers \
		_test-fast \
		_clippy _clippy-executors _lint _lint-rustfmt _lint-configs _lint-docs _lint-automation _lint-clippy _lint-clippy-executors \
		realness-gate \
		_policy-fast _ssot-policy-fast _policy-full _policy-no-raw-cargo _test-profile-invariants _registry-lint _unit-contract-fast _release-readiness _ci-fast _ci-slow _ci-profile-fast _ci-profile-slow _quick _install-ci-tools release-gate \
		_snapshots _snapshots-accept _snapshots-review _fix-snapshots _test-triage _control-plane-inventory _config-inventory _smoke-fastq _smoke-bam local-certification-gate _test-slow _policy-index _policy-only-fast-gate gate-essential gate-execute \
		certify-fastq certify-bam certify-vcf certify-all \
		refresh-assets-toy refresh-assets-golden flake-hunt

release-gate: ## Minimal publishable gate (docs + lint + registry/container locks).
	@$(ensure_artifact_env)
	@cargo run -q -p bijux-dna-dev -- docs run check-doc-links
	@cargo run -q -p bijux-dna-dev -- checks run check-docs-build-contract
	@cargo run -q -p bijux-dna-dev -- checks run check-tool-registry-lock
	@cargo run -q -p bijux-dna-dev -- containers run check-version-lock
	@cargo run -q -p bijux-dna-dev -- containers run check-version-authority
	@cargo run -q -p bijux-dna-dev -- checks run check-root-layout
	@$(MAKE) certify-vcf

_ci-profile-fast:
	@cargo run -q -p bijux-dna-dev -- tooling run ci-fast

_ci-profile-slow:
	@cargo run -q -p bijux-dna-dev -- tooling run ci-slow
