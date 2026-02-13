NEXTEST_PROFILE ?= ci
NEXTEST_TOML := configs/nextest/nextest.toml
NEXTEST_CONFIG ?= --config-file $(NEXTEST_TOML)
NEXTEST_FAST_EXPR ?= not test(/::slow__/)
NEXTEST_NO_TESTS ?= pass
RUN_IGNORED = --run-ignored all
TEST_FEATURES = --all-features
CARGO_BUILD_JOBS ?= $(JOBS)
NEXTEST_TEST_THREADS ?= $(CARGO_BUILD_JOBS)
COVERAGE_BASELINE = artifacts/coverage/baseline.json
COVERAGE_THRESHOLDS := configs/coverage/thresholds.toml
COVERAGE_OUT = coverage.json

fmt:
	@./scripts/run.sh tooling ci-fmt

lint:
	./scripts/run.sh checks check-supported-scripts
	./scripts/run.sh checks check-config-layout
	./scripts/run.sh checks check-config-filenames
	./scripts/run.sh checks check-config-headers
	./scripts/run.sh checks check-config-owners
	./scripts/run.sh checks check-species-aliases
	./scripts/run.sh checks check-bench-knobs
	./scripts/run.sh checks check-registry-split
	./scripts/run.sh checks check-tool-registry-lock
	./scripts/run.sh docs check-domain-doc-references
	./scripts/run.sh docs check-doc-links
	./scripts/run.sh docs check-docs-graph
	./scripts/run.sh docs check-doc-root-layout
	./scripts/run.sh docs check-doc-depth
	./scripts/run.sh docs check-no-placeholder-language
	./scripts/run.sh docs check-generated-docs
	./scripts/run.sh docs check-doc-assets
	./scripts/run.sh tooling check-config-paths
	./scripts/run.sh tooling check-config-snapshot
	./scripts/run.sh checks check-root-layout
	./scripts/run.sh checks check-artifacts-tracked
	./scripts/run.sh checks check-no-target-paths-in-tests
	./scripts/run.sh checks check-no-user-path-literals
	./scripts/run.sh checks check-script-writes
	./scripts/run.sh checks check-assets-drift
	./scripts/run.sh checks check-assets-reference-schema
	./scripts/run.sh checks tree-intent
	./scripts/run.sh checks check-readme-links
	./scripts/run.sh checks check-ci-shell-scripts
	./scripts/run.sh checks check-lib-api
	./scripts/run.sh checks check-isolation-contract
	./scripts/run.sh checks check-shell-portability
	./scripts/run.sh checks check-output-roots
	./scripts/run.sh checks check-script-arg-style
	./scripts/run.sh checks check-no-orphan-scripts
	./scripts/run.sh checks check-no-raw-cargo-in-makefiles
	./scripts/run.sh checks check-no-raw-cargo-in-scripts
	@CARGO_BUILD_JOBS="$(CARGO_BUILD_JOBS)" ./scripts/run.sh tooling ci-clippy

test:
	@NEXTEST_CONFIG="$(NEXTEST_CONFIG)" TEST_FEATURES="$(TEST_FEATURES)" NEXTEST_PROFILE="$(NEXTEST_PROFILE)" NEXTEST_TEST_THREADS="$(NEXTEST_TEST_THREADS)" NEXTEST_NO_TESTS="$(NEXTEST_NO_TESTS)" RUN_IGNORED="$(RUN_IGNORED)" NEXTEST_FAST_EXPR="$(NEXTEST_FAST_EXPR)" ./scripts/run.sh tooling ci-test

test-slow: ## Run only slow-labeled tests (functions containing slow__).
	@NEXTEST_CONFIG="$(NEXTEST_CONFIG)" TEST_FEATURES="$(TEST_FEATURES)" NEXTEST_PROFILE="$(NEXTEST_PROFILE)" NEXTEST_TEST_THREADS="$(NEXTEST_TEST_THREADS)" NEXTEST_NO_TESTS="$(NEXTEST_NO_TESTS)" RUN_IGNORED="$(RUN_IGNORED)" ./scripts/run.sh tooling ci-test-slow

audit:
	@./scripts/run.sh tooling ci-audit

coverage:
	@NEXTEST_CONFIG="$(NEXTEST_CONFIG)" TEST_FEATURES="$(TEST_FEATURES)" NEXTEST_PROFILE="$(NEXTEST_PROFILE)" NEXTEST_TEST_THREADS="$(NEXTEST_TEST_THREADS)" RUN_IGNORED="$(RUN_IGNORED)" COVERAGE_OUT="$(COVERAGE_OUT)" COVERAGE_BASELINE="$(COVERAGE_BASELINE)" COVERAGE_THRESHOLDS="$(COVERAGE_THRESHOLDS)" ./scripts/run.sh tooling ci-coverage

install-ci-tools: ## Install required cargo tools once per CI job.
	@./scripts/run.sh tooling ci-install-tools

domain-gates: domain-validate domain-inventory-drift check-generated-configs check-generated-config-headers

ci:
	$(MAKE) fmt
	$(MAKE) lint
	$(MAKE) audit
	$(MAKE) test
	$(MAKE) coverage

check:
	$(MAKE) fmt lint audit coverage

verify-parallel-isolation:
	@ISO_TAG=verify-a ./bin/isolate sh -ceu 'echo "$$ISO_ROOT" > artifacts/isolates/.verify_a_path'
	@ISO_TAG=verify-b ./bin/isolate sh -ceu 'echo "$$ISO_ROOT" > artifacts/isolates/.verify_b_path'
	@test "$$(cat artifacts/isolates/.verify_a_path)" != "$$(cat artifacts/isolates/.verify_b_path)"
	@rm -f artifacts/isolates/.verify_a_path artifacts/isolates/.verify_b_path

clean-isolates:
	@rm -rf artifacts/isolates/*

policy-fast: ## Run fast policy checks (no snapshots)
	@./scripts/run.sh tooling cargo-targets policy-fast
	$(MAKE) domain-gates

ssot-policy-fast: ## Fast-fail SSOT and registry policy checks.
	./scripts/run.sh checks check-ssot-guardrails
	$(MAKE) domain-gates
	@./scripts/run.sh tooling cargo-targets ssot-policy-fast

test-profile-invariants: ## Run pipeline profile invariant contract tests.
	@./scripts/run.sh tooling cargo-targets test-profile-invariants

registry-lint: ## Run strict tool registry reproducibility policy checks.
	@./scripts/run.sh tooling cargo-targets registry-lint

unit-contract-fast: ## Fast unit/contract checks for critical crates.
	@./scripts/run.sh tooling cargo-targets unit-contract-fast

release-readiness: ## Block merges on experimental tools, unknown metrics schemas, or floating pins.
	$(MAKE) registry-lint
	@./scripts/run.sh tooling cargo-targets release-readiness

ci-fast: ## Fast CI tier: unit + contract + registry lint + profile invariants.
	$(MAKE) ssot-policy-fast
	$(MAKE) fmt
	$(MAKE) lint
	$(MAKE) unit-contract-fast
	$(MAKE) release-readiness
	$(MAKE) test-profile-invariants
	$(MAKE) policy-no-raw-cargo

ci-slow: ## Slow CI tier (manual): heavier integration checks.
	$(MAKE) install-ci-tools
	$(MAKE) audit
	$(MAKE) coverage
	$(MAKE) docs-isolate
	$(MAKE) domain-gates
	$(MAKE) release-readiness

quick: ## Quick local gate: fmt + clippy + unit + invariant tests.
	$(MAKE) fmt
	$(MAKE) lint
	$(MAKE) test-profile-invariants
	$(MAKE) registry-lint

policy-full: ## Run full policy suite
	@./scripts/run.sh tooling cargo-targets policy-full
	$(MAKE) domain-gates

domain-validate:
	./scripts/run.sh domain validate

domain-coverage:
	@./scripts/run.sh tooling cargo-targets domain-coverage

domain-inventory-drift:
	./scripts/run.sh domain inventory-drift

snapshots:
	@./scripts/run.sh tooling cargo-targets snapshots

snapshots-accept:
	@./scripts/run.sh tooling cargo-targets snapshots-accept

snapshots-review:
	@./scripts/run.sh tooling cargo-targets snapshots-review

fix-snapshots: ## Rebuild and accept workspace snapshots with the CI insta workflow.
	@./scripts/run.sh tooling cargo-targets fix-snapshots

test-triage: ## Group failed tests from a saved nextest log.
	@./scripts/run.sh test test-triage artifacts/test-logs/latest.log

generate-configs:
	@./scripts/run.sh tooling generate-configs

check-generated-configs:
	./scripts/run.sh checks check-generated-configs

check-generated-config-headers:
	./scripts/run.sh checks check-generated-config-headers

policy-no-raw-cargo: ## Fail if raw cargo invocations exist in Make/scripts.
	./scripts/run.sh checks check-no-raw-cargo-in-makefiles
	./scripts/run.sh checks check-no-raw-cargo-in-scripts

policy-index: ## Generate policy index under artifacts/.
	@./scripts/run.sh tooling generate-policy-index

policy-only-fast-gate: ## Compile+run policies and critical contract crates only.
	@./scripts/run.sh tooling cargo-targets policy-only-fast-gate

scripts-inventory: ## Generate scripts inventory under artifacts/
	@./scripts/run.sh tooling inventory

config-inventory: ## Generate config inventory under artifacts/
	@./scripts/run.sh tooling config-inventory

smoke-fastq: ## Quick local FASTQ smoke dry-run.
	@./scripts/run.sh smoke run fastq

smoke-bam: ## Quick local BAM smoke dry-run.
	@./scripts/run.sh smoke run bam

refresh-assets-toy: ## Regenerate deterministic toy datasets in assets/toy.
	@./scripts/run.sh assets refresh-toy

refresh-assets-golden: ## Regenerate deterministic toy-run goldens in assets/golden.
	@./scripts/run.sh assets refresh-golden

.PHONY: fmt lint test audit coverage ci check verify-parallel-isolation \
		clean-isolates \
		domain-gates \
		domain-validate domain-coverage domain-inventory-drift generate-configs check-generated-configs check-generated-config-headers \
		policy-fast ssot-policy-fast policy-full policy-no-raw-cargo test-profile-invariants registry-lint unit-contract-fast release-readiness ci-fast ci-slow quick install-ci-tools \
		snapshots snapshots-accept snapshots-review fix-snapshots test-triage scripts-inventory config-inventory smoke-fastq smoke-bam test-slow policy-index policy-only-fast-gate \
		refresh-assets-toy refresh-assets-golden
