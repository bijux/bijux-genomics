NEXTEST_PROFILE ?= ci
NEXTEST_PROFILE_FAST ?= fast-unit
NEXTEST_PROFILE_SLOW ?= slow-integration
NEXTEST_PROFILE_CERT ?= certification
ARTIFACTS_DIR ?= $(ARTIFACT_ROOT)/make/$(or $(MAKECMDGOALS),manual)
NEXTEST_TOML := configs/rust/nextest.toml
NEXTEST_CONFIG ?= --config-file $(NEXTEST_TOML)
NEXTEST_FAST_EXPR ?= not test(/::slow__/)
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

fmt:
	@$(ensure_artifact_env)
	@$(MAKE) _fmt

_fmt:
	@$(ensure_artifact_env)
	@./scripts/run.sh tooling ci-fmt

lint:
	@$(ensure_artifact_env)
	@$(MAKE) _lint

_lint:
	@$(MAKE) _lint-rustfmt
	@$(MAKE) _lint-configs
	@$(MAKE) _lint-docs
	@$(MAKE) _lint-scripts
	@$(MAKE) _lint-clippy

_lint-rustfmt:
	@$(ensure_artifact_env)
	@./scripts/run.sh tooling ci-fmt

_lint-configs:
	@$(ensure_artifact_env)
	@./scripts/run.sh checks check-config-schema
	@./scripts/run.sh checks check-config-layout
	@./scripts/run.sh checks check-generated-configs
	@./scripts/run.sh checks check-generated-config-headers

_lint-docs:
	@$(ensure_artifact_env)
	@./scripts/run.sh docs check-doc-links
	@./scripts/run.sh checks check-docs-build-contract

_lint-scripts:
	@$(ensure_artifact_env)
	./scripts/run.sh tooling repo-doctor --fast
	@rm -rf "$(ARTIFACTS_DIR)/lint-parallel"
	@mkdir -p "$(ARTIFACTS_DIR)/lint-parallel"
	@cp "$(LINT_PARALLEL_COMMANDS_FILE)" "$(ARTIFACTS_DIR)/lint-parallel/commands.txt"
	@echo "Running lint script gates in parallel (jobs=$(LINT_PARALLEL_JOBS)); logs: $(ARTIFACTS_DIR)/lint-parallel"
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

lint-scripts: ## Run repo-doctor + script/container lint checks (parallelized), without clippy.
	@$(ensure_artifact_env)
	@$(MAKE) _lint-scripts

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
	@./scripts/run.sh tooling lint-fast

_lint-clippy:
	@$(ensure_artifact_env)
	@CARGO_BUILD_JOBS="$(CARGO_BUILD_JOBS)" ./scripts/run.sh tooling ci-clippy

_lint-clippy-executors:
	@$(ensure_artifact_env)
	@CARGO_BUILD_JOBS="$(CARGO_BUILD_JOBS)" ./scripts/run.sh tooling ci-clippy-executors

_clippy: ## Run workspace clippy only (no script gates).
	@$(MAKE) _lint-clippy

_clippy-executors: ## Run deny-warnings clippy for runner/executor crates.
	@$(MAKE) _lint-clippy-executors

test:
	@$(ensure_artifact_env)
	@$(MAKE) _test

test-fast:
	@$(ensure_artifact_env)
	@$(MAKE) _test-fast

_test:
	@$(ensure_artifact_env)
	@NEXTEST_CONFIG="$(NEXTEST_CONFIG)" TEST_FEATURES="$(TEST_FEATURES)" NEXTEST_PROFILE="$(NEXTEST_PROFILE)" NEXTEST_TEST_THREADS="$(NEXTEST_TEST_THREADS)" NEXTEST_NO_TESTS="$(NEXTEST_NO_TESTS)" RUN_IGNORED="$(RUN_IGNORED)" ./scripts/run.sh tooling ci-test

_test-fast: ## Run fast test suite excluding only slow-labeled tests.
	@$(ensure_artifact_env)
	@NEXTEST_CONFIG="$(NEXTEST_CONFIG)" TEST_FEATURES="$(TEST_FEATURES)" NEXTEST_PROFILE="$(NEXTEST_PROFILE_FAST)" NEXTEST_TEST_THREADS="$(NEXTEST_TEST_THREADS)" NEXTEST_NO_TESTS="$(NEXTEST_NO_TESTS)" RUN_IGNORED="$(RUN_IGNORED)" NEXTEST_FAST_EXPR="$(NEXTEST_FAST_EXPR)" ./scripts/run.sh tooling ci-test

_test-slow: ## Run only slow-labeled tests (functions containing slow__).
	@$(ensure_artifact_env)
	@NEXTEST_CONFIG="$(NEXTEST_CONFIG)" TEST_FEATURES="$(TEST_FEATURES)" NEXTEST_PROFILE="$(NEXTEST_PROFILE_SLOW)" NEXTEST_TEST_THREADS="$(NEXTEST_TEST_THREADS)" NEXTEST_NO_TESTS="$(NEXTEST_NO_TESTS)" RUN_IGNORED="$(RUN_IGNORED)" ./scripts/run.sh tooling ci-test-slow

audit:
	@$(ensure_artifact_env)
	@$(MAKE) _audit

_audit:
	@$(ensure_artifact_env)
	@./scripts/run.sh tooling ci-audit

coverage:
	@$(ensure_artifact_env)
	@$(MAKE) _coverage

_coverage:
	@$(ensure_artifact_env)
	@NEXTEST_CONFIG="$(NEXTEST_CONFIG)" TEST_FEATURES="$(TEST_FEATURES)" NEXTEST_PROFILE="$(NEXTEST_PROFILE)" NEXTEST_TEST_THREADS="$(NEXTEST_TEST_THREADS)" RUN_IGNORED="$(RUN_IGNORED)" COVERAGE_OUT="$(COVERAGE_OUT)" COVERAGE_BASELINE="$(COVERAGE_BASELINE)" COVERAGE_THRESHOLDS="$(COVERAGE_THRESHOLDS)" ./scripts/run.sh tooling ci-coverage

doctor:
	@$(ensure_artifact_env)
	@$(MAKE) _doctor

_doctor:
	@$(ensure_artifact_env)
	@./scripts/run.sh tooling repo-doctor --fast
	@./scripts/run.sh checks check-supported-scripts
	@./scripts/run.sh checks check-config-schema
	@./scripts/run.sh checks check-nextest-profile-contract
	@./scripts/run.sh checks check-runtime-profiles-contract
	@./scripts/run.sh checks check-logging-contract
	@./scripts/run.sh checks check-hpc-rsync-docs-parity
	@./scripts/run.sh checks check-run-directory-layout
	@./scripts/run.sh checks check-registry-required-tools-parity
	@./scripts/run.sh checks check-domain-tool-parity
	@./scripts/run.sh checks check-stage-domain-parity
	@./scripts/run.sh checks check-stage-registry-governance
	@./scripts/run.sh checks check-enabled-vcf-panel-metadata
	@./scripts/run.sh checks check-param-registry-completeness
	@./scripts/run.sh checks check-deprecations-enforcement
	@./scripts/run.sh checks check-no-raw-cargo-in-makes

_install-ci-tools: ## Install required cargo tools once per CI job.
	@$(ensure_artifact_env)
	@./scripts/run.sh tooling ci-install-tools

_domain-gates: _domain-validate _domain-inventory-drift _check-generated-configs _check-generated-config-headers

ci:
	@$(ensure_artifact_env)
	@./scripts/run.sh tooling repo-doctor --fast
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
	@./scripts/run.sh tooling cargo-targets policy-fast
	$(MAKE) _domain-gates

_ssot-policy-fast: ## Fast-fail SSOT and registry policy checks.
	./scripts/run.sh checks check-ssot-guardrails
	$(MAKE) _domain-gates
	@./scripts/run.sh tooling cargo-targets ssot-policy-fast

_test-profile-invariants: ## Run pipeline profile invariant contract tests.
	@./scripts/run.sh tooling cargo-targets test-profile-invariants

_registry-lint: ## Run strict tool registry reproducibility policy checks.
	@./scripts/run.sh tooling cargo-targets registry-lint

_unit-contract-fast: ## Fast unit/contract checks for critical crates.
	@./scripts/run.sh tooling cargo-targets unit-contract-fast

_release-readiness: ## Block merges on experimental tools, unknown metrics schemas, or floating pins.
	$(MAKE) _registry-lint
	@./scripts/run.sh tooling cargo-targets release-readiness

_ci-fast: ## Fast CI tier: unit + contract + registry lint + profile invariants.
	$(MAKE) _ssot-policy-fast
	$(MAKE) fmt
	$(MAKE) lint
	$(MAKE) _unit-contract-fast
	$(MAKE) _release-readiness
	$(MAKE) _test-profile-invariants
	$(MAKE) _policy-no-raw-cargo

_ci-slow: ## Slow CI tier (manual): heavier integration checks.
	$(MAKE) _install-ci-tools
	$(MAKE) audit
	$(MAKE) coverage
	$(MAKE) _docs-contract
	$(MAKE) _domain-gates
	$(MAKE) _release-readiness

_quick: ## Quick local gate: fmt + clippy + unit + invariant tests.
	$(MAKE) fmt
	$(MAKE) lint
	$(MAKE) _test-profile-invariants
	$(MAKE) _registry-lint

_policy-full: ## Run full policy suite
	@./scripts/run.sh tooling cargo-targets policy-full
	$(MAKE) _domain-gates

_domain-validate:
	./scripts/run.sh domain validate

domain-validate:
	./scripts/run.sh domain validate

_domain-coverage:
	@./scripts/run.sh tooling cargo-targets domain-coverage

_domain-inventory-drift:
	./scripts/run.sh domain inventory-drift

_snapshots:
	@./scripts/run.sh tooling cargo-targets snapshots

_snapshots-accept:
	@./scripts/run.sh tooling cargo-targets snapshots-accept

_snapshots-review:
	@./scripts/run.sh tooling cargo-targets snapshots-review

_fix-snapshots: ## Rebuild and accept workspace snapshots with the CI insta workflow.
	@./scripts/run.sh tooling cargo-targets fix-snapshots

_test-triage: ## Group failed tests from a saved nextest log.
	@./scripts/run.sh test test-triage "$(ARTIFACTS_DIR)/test-logs/latest.log"

generate-configs:
	@$(ensure_artifact_env)
	@./scripts/run.sh tooling generate-configs

_generate-configs:
	@$(MAKE) generate-configs

_check-generated-configs:
	./scripts/run.sh checks check-generated-configs

_check-generated-config-headers:
	./scripts/run.sh checks check-generated-config-headers

_policy-no-raw-cargo: ## Fail if raw cargo invocations exist in Make/scripts.
	./scripts/run.sh checks check-no-raw-cargo-in-makes
	./scripts/run.sh checks check-no-raw-cargo-in-scripts

flake-hunt: ## Run repeated flake hunt for an expression (EXPR required, RUNS optional).
	@$(ensure_artifact_env)
	@if [ -z "$(EXPR)" ]; then echo "EXPR is required, e.g. make flake-hunt EXPR='test(...)' RUNS=20" >&2; exit 2; fi
	@./scripts/run.sh tooling flake-hunt --expr "$(EXPR)" --runs "$(or $(RUNS),20)"

realness-gate: ## Run strict realness checks (placeholder artifacts + planner realization).
	@$(ensure_artifact_env)
	@./scripts/run.sh checks check-domain-realization
	@./scripts/run.sh checks check-no-fake-artifacts

_policy-index: ## Generate policy index under artifacts/.
	@./scripts/run.sh tooling generate-policy-index

_policy-only-fast-gate: ## Compile+run policies and critical contract crates only.
	@./scripts/run.sh tooling cargo-targets policy-only-fast-gate

_scripts-inventory: ## Generate scripts inventory under artifacts/
	@./scripts/run.sh tooling inventory

_config-inventory: ## Generate config inventory under artifacts/
	@./scripts/run.sh tooling config-inventory

_smoke-fastq: ## Quick local FASTQ smoke dry-run.
	@./scripts/run.sh smoke run fastq

_smoke-bam: ## Quick local BAM smoke dry-run.
	@./scripts/run.sh smoke run bam

local-certification-gate: ## Run local mini-domain certification suite and emit bundle.
	@$(ensure_artifact_env)
	@./scripts/run.sh tooling certification-gate

vcf-certification: ## Local-only VCF certification run (sequential VCF stage contract suite).
	@$(ensure_artifact_env)
	@NEXTEST_PROFILE="$(NEXTEST_PROFILE_CERT)" ./scripts/run.sh tooling cargo-targets vcf-certification

certify-fastq: ## Local FASTQ certification smoke.
	@$(ensure_artifact_env)
	@./scripts/run.sh tooling certify-fastq

certify-bam: ## Local BAM certification smoke.
	@$(ensure_artifact_env)
	@./scripts/run.sh tooling certify-bam

certify-vcf: ## Local VCF certification suite.
	@$(ensure_artifact_env)
	@./scripts/run.sh tooling certify-vcf

certify-all: ## Local cross-domain certification bundle (FASTQ+BAM+VCF downstream mini).
	@$(ensure_artifact_env)
	@./scripts/run.sh tooling certify-all

examples-validate:
	@$(MAKE) _examples-validate

_examples-validate:
	./scripts/run.sh checks check-examples-structure
	./scripts/run.sh checks check-examples-index-ssot
	./scripts/run.sh checks check-examples-corpus-manifests
	./scripts/run.sh checks check-examples-corpus-checksums
	./scripts/run.sh checks check-examples-corpus-layout
	./scripts/run.sh checks check-examples-golden
	./scripts/run.sh checks check-examples-runner-contract
	./scripts/run.sh checks check-examples-cli-snapshot
	./scripts/run.sh checks check-examples-notebook-policy
	./scripts/run.sh checks check-examples-policy

refresh-assets-toy: ## Regenerate deterministic toy datasets in assets/toy.
	@./scripts/run.sh assets refresh-toy

refresh-assets-golden: ## Regenerate deterministic toy-run goldens in assets/golden.
	@./scripts/run.sh assets refresh-golden

.PHONY: fmt lint lint-rustfmt lint-clippy lint-docs lint-configs lint-fast lint-scripts test test-fast audit coverage ci doctor _check _verify-artifact-env \
		_clean-artifact-scratch \
		_domain-gates domain-validate examples-validate \
		_examples-validate \
		_domain-validate _domain-coverage _domain-inventory-drift _generate-configs _check-generated-configs _check-generated-config-headers \
		_test-fast \
		_clippy _clippy-executors _lint _lint-rustfmt _lint-configs _lint-docs _lint-scripts _lint-clippy _lint-clippy-executors \
		realness-gate \
		_policy-fast _ssot-policy-fast _policy-full _policy-no-raw-cargo _test-profile-invariants _registry-lint _unit-contract-fast _release-readiness _ci-fast _ci-slow _ci-profile-fast _ci-profile-slow _quick _install-ci-tools release-gate \
		_snapshots _snapshots-accept _snapshots-review _fix-snapshots _test-triage _scripts-inventory _config-inventory _smoke-fastq _smoke-bam local-certification-gate _test-slow _policy-index _policy-only-fast-gate \
		certify-fastq certify-bam certify-vcf certify-all \
		refresh-assets-toy refresh-assets-golden flake-hunt

release-gate: ## Minimal publishable gate (docs + lint + registry/container locks).
	@$(ensure_artifact_env)
	@./scripts/run.sh docs check-doc-links
	@./scripts/run.sh checks check-docs-build-contract
	@./scripts/run.sh checks check-tool-registry-lock
	@./scripts/run.sh containers check-version-lock
	@./scripts/run.sh containers check-version-authority
	@./scripts/run.sh checks check-root-layout
	@$(MAKE) certify-vcf

_ci-profile-fast:
	@./scripts/run.sh tooling ci-fast

_ci-profile-slow:
	@./scripts/run.sh tooling ci-slow
