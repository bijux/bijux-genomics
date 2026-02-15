NEXTEST_PROFILE ?= ci
ARTIFACTS_DIR ?= $(if $(ISO_ROOT),$(ISO_ROOT)/artifacts/isolate/$(or $(MAKECMDGOALS),manual)/$(or $(ISO_RUN_ID),no-runid),artifacts/isolate/$(or $(MAKECMDGOALS),manual)/$(or $(ISO_RUN_ID),local))
NEXTEST_TOML := configs/nextest/nextest.toml
NEXTEST_CONFIG ?= --config-file $(NEXTEST_TOML)
NEXTEST_FAST_EXPR ?= not test(/::slow__/)
NEXTEST_NO_TESTS ?= pass
RUN_IGNORED = --run-ignored all
TEST_FEATURES = --all-features
CARGO_BUILD_JOBS ?= $(JOBS)
NEXTEST_TEST_THREADS ?= $(CARGO_BUILD_JOBS)
LINT_PARALLEL_JOBS ?= $(if $(CARGO_BUILD_JOBS),$(CARGO_BUILD_JOBS),8)
LINT_PARALLEL_COMMANDS_FILE ?= makefiles/lint.parallel.commands.txt
COVERAGE_BASELINE = artifacts/coverage/baseline.json
COVERAGE_THRESHOLDS := configs/coverage/thresholds.toml
COVERAGE_OUT = coverage.json
AUTO_ISO_TAG_PREFIX ?= make

fmt:
	@if [ -n "$$ISO_ROOT" ]; then ./bin/require-isolate >/dev/null; fi
	@if [ -z "$$ISO_ROOT" ]; then \
		tag="$(AUTO_ISO_TAG_PREFIX)-fmt-$$(date -u +%Y%m%dT%H%M%SZ)-$$PPID"; \
		ISO_TAG="$$tag" ./bin/isolate --tag "$$tag" $(MAKE) _fmt; \
	else \
		$(MAKE) _fmt; \
	fi

_fmt:
	@./bin/require-isolate >/dev/null
	@./scripts/run.sh tooling ci-fmt

lint:
	@if [ -n "$$ISO_ROOT" ]; then ./bin/require-isolate >/dev/null; fi
	@if [ -z "$$ISO_ROOT" ]; then \
		tag="$(AUTO_ISO_TAG_PREFIX)-lint-$$(date -u +%Y%m%dT%H%M%SZ)-$$PPID"; \
		ISO_TAG="$$tag" ./bin/isolate --tag "$$tag" $(MAKE) _lint; \
	else \
		$(MAKE) _lint; \
	fi

_lint:
	@./bin/require-isolate >/dev/null
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
	@CARGO_BUILD_JOBS="$(CARGO_BUILD_JOBS)" ./scripts/run.sh tooling ci-clippy

_clippy: ## Run workspace clippy only (no script gates).
	@./bin/require-isolate >/dev/null
	@CARGO_BUILD_JOBS="$(CARGO_BUILD_JOBS)" ./scripts/run.sh tooling ci-clippy

test:
	@if [ -n "$$ISO_ROOT" ]; then ./bin/require-isolate >/dev/null; fi
	@if [ -z "$$ISO_ROOT" ]; then \
		tag="$(AUTO_ISO_TAG_PREFIX)-test-$$(date -u +%Y%m%dT%H%M%SZ)-$$PPID"; \
		ISO_TAG="$$tag" ./bin/isolate --tag "$$tag" $(MAKE) _test; \
	else \
		$(MAKE) _test; \
	fi

test-fast:
	@if [ -n "$$ISO_ROOT" ]; then ./bin/require-isolate >/dev/null; fi
	@if [ -z "$$ISO_ROOT" ]; then \
		tag="$(AUTO_ISO_TAG_PREFIX)-test-fast-$$(date -u +%Y%m%dT%H%M%SZ)-$$PPID"; \
		ISO_TAG="$$tag" ./bin/isolate --tag "$$tag" $(MAKE) _test-fast; \
	else \
		$(MAKE) _test-fast; \
	fi

_test:
	@./bin/require-isolate >/dev/null
	@NEXTEST_CONFIG="$(NEXTEST_CONFIG)" TEST_FEATURES="$(TEST_FEATURES)" NEXTEST_PROFILE="$(NEXTEST_PROFILE)" NEXTEST_TEST_THREADS="$(NEXTEST_TEST_THREADS)" NEXTEST_NO_TESTS="$(NEXTEST_NO_TESTS)" RUN_IGNORED="$(RUN_IGNORED)" NEXTEST_FAST_EXPR="$(NEXTEST_FAST_EXPR)" ./scripts/run.sh tooling ci-test

_test-fast: ## Run fast test suite excluding slow-labeled tests and heavyweight policy package.
	@./bin/require-isolate >/dev/null
	@NEXTEST_CONFIG="$(NEXTEST_CONFIG)" TEST_FEATURES="$(TEST_FEATURES)" NEXTEST_PROFILE="$(NEXTEST_PROFILE)" NEXTEST_TEST_THREADS="$(NEXTEST_TEST_THREADS)" NEXTEST_NO_TESTS="$(NEXTEST_NO_TESTS)" RUN_IGNORED="$(RUN_IGNORED)" NEXTEST_FAST_EXPR="not test(/::slow__/) and not package(bijux-dna-policies)" ./scripts/run.sh tooling ci-test

_test-slow: ## Run only slow-labeled tests (functions containing slow__).
	@NEXTEST_CONFIG="$(NEXTEST_CONFIG)" TEST_FEATURES="$(TEST_FEATURES)" NEXTEST_PROFILE="$(NEXTEST_PROFILE)" NEXTEST_TEST_THREADS="$(NEXTEST_TEST_THREADS)" NEXTEST_NO_TESTS="$(NEXTEST_NO_TESTS)" RUN_IGNORED="$(RUN_IGNORED)" ./scripts/run.sh tooling ci-test-slow

audit:
	@if [ -n "$$ISO_ROOT" ]; then ./bin/require-isolate >/dev/null; fi
	@if [ -z "$$ISO_ROOT" ]; then \
		tag="$(AUTO_ISO_TAG_PREFIX)-audit-$$(date -u +%Y%m%dT%H%M%SZ)-$$PPID"; \
		ISO_TAG="$$tag" ./bin/isolate --tag "$$tag" $(MAKE) _audit; \
	else \
		$(MAKE) _audit; \
	fi

_audit:
	@./bin/require-isolate >/dev/null
	@./scripts/run.sh tooling ci-audit

coverage:
	@if [ -n "$$ISO_ROOT" ]; then ./bin/require-isolate >/dev/null; fi
	@if [ -z "$$ISO_ROOT" ]; then \
		tag="$(AUTO_ISO_TAG_PREFIX)-coverage-$$(date -u +%Y%m%dT%H%M%SZ)-$$PPID"; \
		ISO_TAG="$$tag" ./bin/isolate --tag "$$tag" $(MAKE) _coverage; \
	else \
		$(MAKE) _coverage; \
	fi

_coverage:
	@./bin/require-isolate >/dev/null
	@NEXTEST_CONFIG="$(NEXTEST_CONFIG)" TEST_FEATURES="$(TEST_FEATURES)" NEXTEST_PROFILE="$(NEXTEST_PROFILE)" NEXTEST_TEST_THREADS="$(NEXTEST_TEST_THREADS)" RUN_IGNORED="$(RUN_IGNORED)" COVERAGE_OUT="$(COVERAGE_OUT)" COVERAGE_BASELINE="$(COVERAGE_BASELINE)" COVERAGE_THRESHOLDS="$(COVERAGE_THRESHOLDS)" ./scripts/run.sh tooling ci-coverage

doctor:
	@if [ -n "$$ISO_ROOT" ]; then ./bin/require-isolate >/dev/null; fi
	@if [ -z "$$ISO_ROOT" ]; then \
		tag="$(AUTO_ISO_TAG_PREFIX)-doctor-$$(date -u +%Y%m%dT%H%M%SZ)-$$PPID"; \
		ISO_TAG="$$tag" ./bin/isolate --tag "$$tag" $(MAKE) _doctor; \
	else \
		$(MAKE) _doctor; \
	fi

_doctor:
	@./bin/require-isolate >/dev/null
	@./scripts/run.sh tooling repo-doctor --fast
	@./scripts/run.sh checks check-supported-scripts
	@./scripts/run.sh checks check-config-schema
	@./scripts/run.sh checks check-nextest-profile-contract
	@./scripts/run.sh checks check-runtime-profiles-contract
	@./scripts/run.sh checks check-logging-contract
	@./scripts/run.sh checks check-hpc-rsync-docs-parity
	@./scripts/run.sh checks check-registry-required-tools-parity
	@./scripts/run.sh checks check-domain-tool-parity
	@./scripts/run.sh checks check-stage-domain-parity
	@./scripts/run.sh checks check-param-registry-completeness
	@./scripts/run.sh checks check-deprecations-enforcement
	@./scripts/run.sh checks check-no-raw-cargo-in-makefiles

_install-ci-tools: ## Install required cargo tools once per CI job.
	@./scripts/run.sh tooling ci-install-tools

_domain-gates: _domain-validate _domain-inventory-drift _check-generated-configs _check-generated-config-headers

ci:
	@./bin/isolate sh -ceu 'export CARGO_TARGET_DIR="$$ISO_ROOT/target-ci"; ./scripts/run.sh tooling repo-doctor --fast; $(MAKE) fmt lint audit test coverage'

_check:
	$(MAKE) fmt lint audit coverage

_verify-parallel-isolation:
	@ISO_TAG=verify-a ./bin/isolate sh -ceu 'echo "$$ISO_ROOT" > "$$ISO_ROOT/.verify_path"'
	@ISO_TAG=verify-b ./bin/isolate sh -ceu 'echo "$$ISO_ROOT" > "$$ISO_ROOT/.verify_path"'
	@a_root="$$(ISO_TAG=verify-a ./bin/isolate --print-root)"; b_root="$$(ISO_TAG=verify-b ./bin/isolate --print-root)"; test "$$(cat "$$a_root/.verify_path")" != "$$(cat "$$b_root/.verify_path")"
	@ISO_TAG=verify-a ./bin/isolate sh -ceu 'rm -f "$$ISO_ROOT/.verify_path"'
	@ISO_TAG=verify-b ./bin/isolate sh -ceu 'rm -f "$$ISO_ROOT/.verify_path"'

_clean-isolates:
	@rm -rf artifacts/isolates/*

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
	$(MAKE) _docs-isolate
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
	@./scripts/run.sh tooling generate-configs

_generate-configs:
	@$(MAKE) generate-configs

_check-generated-configs:
	./scripts/run.sh checks check-generated-configs

_check-generated-config-headers:
	./scripts/run.sh checks check-generated-config-headers

_policy-no-raw-cargo: ## Fail if raw cargo invocations exist in Make/scripts.
	./scripts/run.sh checks check-no-raw-cargo-in-makefiles
	./scripts/run.sh checks check-no-raw-cargo-in-scripts

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

.PHONY: fmt lint test test-fast audit coverage ci doctor _check _verify-parallel-isolation \
		_clean-isolates \
		_domain-gates domain-validate examples-validate \
		_examples-validate \
		_domain-validate _domain-coverage _domain-inventory-drift _generate-configs _check-generated-configs _check-generated-config-headers \
		_test-fast \
		_clippy \
		_policy-fast _ssot-policy-fast _policy-full _policy-no-raw-cargo _test-profile-invariants _registry-lint _unit-contract-fast _release-readiness _ci-fast _ci-slow _ci-profile-fast _ci-profile-slow _quick _install-ci-tools release-gate \
		_snapshots _snapshots-accept _snapshots-review _fix-snapshots _test-triage _scripts-inventory _config-inventory _smoke-fastq _smoke-bam _test-slow _policy-index _policy-only-fast-gate \
		refresh-assets-toy refresh-assets-golden
release-gate: ## Minimal publishable gate (docs + lint + registry/container locks).
	@./bin/require-isolate >/dev/null
	@./scripts/run.sh docs check-doc-links
	@./scripts/run.sh checks check-docs-build-contract
	@./scripts/run.sh checks check-tool-registry-lock
	@./scripts/run.sh containers check-version-lock
	@./scripts/run.sh containers check-version-authority
	@./scripts/run.sh checks check-root-layout

_ci-profile-fast:
	@./scripts/run.sh tooling ci-fast

_ci-profile-slow:
	@./scripts/run.sh tooling ci-slow
