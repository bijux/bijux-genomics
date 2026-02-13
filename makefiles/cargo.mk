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

REQUIRED_CARGO_TOOLS = cargo-nextest cargo-llvm-cov cargo-deny

define RUN_IN_ISOLATE
	@./bin/isolate sh -ceu '$(1)'
endef

define REQUIRE_TOOL
	command -v $(1) >/dev/null 2>&1 || { echo "missing required tool: $(1)"; echo "install once: cargo install $(1) --locked"; exit 1; }
endef

fmt:
	$(call RUN_IN_ISOLATE,./bin/require-isolate >/dev/null; cargo fmt --all -- --check)

lint:
	./scripts/checks/check-supported-scripts.sh
	./scripts/checks/check-config-layout.sh
	./scripts/checks/check-config-filenames.sh
	./scripts/checks/check-config-contract-headers.sh
	./scripts/docs/check-domain-doc-references.sh
	./scripts/docs/check-doc-links.sh
	./scripts/docs/check-generated-docs.sh
	./scripts/docs/check-doc-assets.sh
	./scripts/tooling/check-config-paths.sh
	./scripts/tooling/check-config-snapshot.sh
	./scripts/checks/check-root-layout.sh
	./scripts/checks/check-artifacts-tracked.sh
	./scripts/checks/check-no-target-paths-in-tests.sh
	./scripts/checks/check-no-user-path-literals.sh
	./scripts/checks/check-script-writes.sh
	./scripts/checks/check-assets-drift.sh
	./scripts/checks/tree-intent.sh
	./scripts/checks/check-readme-links.sh
	./scripts/checks/check-ci-shell-scripts.sh
	$(call RUN_IN_ISOLATE,./bin/require-isolate >/dev/null; CARGO_BUILD_JOBS=$(CARGO_BUILD_JOBS) cargo clippy --workspace --all-targets --all-features -- -D warnings)

test:
	$(call RUN_IN_ISOLATE,./bin/require-isolate >/dev/null; ./scripts/checks/check-isolation-contract.sh; ./scripts/checks/check-ssot-guardrails.sh; $(call REQUIRE_TOOL,cargo-nextest); export TZ=UTC LC_ALL=C TEST_TARGET_DIR="$$ISO_ROOT/target-test" COV_TARGET_DIR="$$ISO_ROOT/target-cov" TEST_TMP_DIR="$$ISO_ROOT/tmp-test" COV_TMP_DIR="$$ISO_ROOT/tmp-cov" TEST_PROFRAW_DIR="$$ISO_ROOT/profraw-test" COV_PROFRAW_DIR="$$ISO_ROOT/profraw-cov" CARGO_TARGET_DIR="$$ISO_ROOT/target-test"; if command -v sccache >/dev/null 2>&1; then export RUSTC_WRAPPER="$$(command -v sccache)"; fi; cargo nextest run $(NEXTEST_CONFIG) --workspace $(TEST_FEATURES) --profile $(NEXTEST_PROFILE) --test-threads $(NEXTEST_TEST_THREADS) --no-tests $(NEXTEST_NO_TESTS) $(RUN_IGNORED) -E "$(NEXTEST_FAST_EXPR)"; ./scripts/checks/check-isolation-contract.sh)

test-slow: ## Run only slow-labeled tests (functions containing slow__).
	$(call RUN_IN_ISOLATE,./bin/require-isolate >/dev/null; $(call REQUIRE_TOOL,cargo-nextest); export TZ=UTC LC_ALL=C CARGO_TARGET_DIR="$$ISO_ROOT/target-test"; if command -v sccache >/dev/null 2>&1; then export RUSTC_WRAPPER="$$(command -v sccache)"; fi; cargo nextest run $(NEXTEST_CONFIG) --workspace $(TEST_FEATURES) --profile $(NEXTEST_PROFILE) --test-threads $(NEXTEST_TEST_THREADS) --no-tests $(NEXTEST_NO_TESTS) $(RUN_IGNORED) -E "test(/::slow__/)")

audit:
	$(call RUN_IN_ISOLATE,./bin/require-isolate >/dev/null; $(call REQUIRE_TOOL,cargo-deny); cargo deny check)

coverage:
	$(call RUN_IN_ISOLATE,./bin/require-isolate >/dev/null; $(call REQUIRE_TOOL,cargo-llvm-cov); $(call REQUIRE_TOOL,cargo-nextest); export TZ=UTC LC_ALL=C TEST_TARGET_DIR="$$ISO_ROOT/target-test" COV_TARGET_DIR="$$ISO_ROOT/target-cov" TEST_TMP_DIR="$$ISO_ROOT/tmp-test" COV_TMP_DIR="$$ISO_ROOT/tmp-cov" TEST_PROFRAW_DIR="$$ISO_ROOT/profraw-test" COV_PROFRAW_DIR="$$ISO_ROOT/profraw-cov" CARGO_TARGET_DIR="$$ISO_ROOT/target-test"; if command -v sccache >/dev/null 2>&1; then export RUSTC_WRAPPER="$$(command -v sccache)"; fi; cargo llvm-cov clean; rm -rf "$$ISO_ROOT/coverage"; mkdir -p "$$ISO_ROOT/coverage"; cargo llvm-cov nextest --no-report --no-cfg-coverage $(NEXTEST_CONFIG) --workspace $(TEST_FEATURES) --profile $(NEXTEST_PROFILE) --test-threads $(NEXTEST_TEST_THREADS) $(RUN_IGNORED); cargo llvm-cov report --json --output-path "$$ISO_ROOT/coverage/$(COVERAGE_OUT)"; cargo llvm-cov report --html --output-dir "$$ISO_ROOT/coverage"; test -f "$$ISO_ROOT/coverage/$(COVERAGE_OUT)"; test -f "$$ISO_ROOT/coverage/index.html"; if [ -f $(COVERAGE_BASELINE) ]; then python3 scripts/tooling/coverage_summary.sh "$$ISO_ROOT/coverage/$(COVERAGE_OUT)" --baseline $(COVERAGE_BASELINE) --check-thresholds $(COVERAGE_THRESHOLDS); else python3 scripts/tooling/coverage_summary.sh "$$ISO_ROOT/coverage/$(COVERAGE_OUT)" --check-thresholds $(COVERAGE_THRESHOLDS); fi)

install-ci-tools: ## Install required cargo tools once per CI job.
	$(call RUN_IN_ISOLATE,./bin/require-isolate >/dev/null; cargo install --locked cargo-nextest cargo-llvm-cov cargo-deny)

domain-gates: domain-validate domain-inventory-drift check-generated-configs check-generated-config-headers

ci:
	$(MAKE) install-ci-tools
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
	@./bin/isolate cargo test -p bijux-dna-policies --test dependency_graph --test purity_scans --test core_layering --test domain_dependency_policy --test ci_tools_policy --test dev_deps_policy --test heavy_deps_policy
	$(MAKE) domain-gates

ssot-policy-fast: ## Fast-fail SSOT and registry policy checks.
	./scripts/checks/check-ssot-guardrails.sh
	$(MAKE) domain-gates
	$(call RUN_IN_ISOLATE,./bin/require-isolate >/dev/null; export TZ=UTC LC_ALL=C CARGO_TARGET_DIR="$$ISO_ROOT/target-test"; if command -v sccache >/dev/null 2>&1; then export RUSTC_WRAPPER="$$(command -v sccache)"; fi; cargo test -p bijux-dna-policies --test contracts policy_test_names_are_consistent -- --nocapture; cargo test -p bijux-dna-policies --test contracts supported_stages_and_tools_are_complete -- --nocapture; cargo test -p bijux-dna-policies --test contracts each_tool_has_exactly_one_domain_and_stage_binding -- --nocapture)

test-profile-invariants: ## Run pipeline profile invariant contract tests.
	$(call RUN_IN_ISOLATE,./bin/require-isolate >/dev/null; export TZ=UTC LC_ALL=C CARGO_TARGET_DIR="$$ISO_ROOT/target-test"; if command -v sccache >/dev/null 2>&1; then export RUSTC_WRAPPER="$$(command -v sccache)"; fi; cargo test -p bijux-dna-pipelines --test invariant_fast -- --nocapture)

registry-lint: ## Run strict tool registry reproducibility policy checks.
	$(call RUN_IN_ISOLATE,./bin/require-isolate >/dev/null; export TZ=UTC LC_ALL=C CARGO_TARGET_DIR="$$ISO_ROOT/target-test"; if command -v sccache >/dev/null 2>&1; then export RUSTC_WRAPPER="$$(command -v sccache)"; fi; cargo test -p bijux-dna-policies --test contracts production_registry_is_pinned_and_non_floating -- --nocapture; cargo test -p bijux-dna-policies --test contracts profiles_only_use_valid_production_tools -- --nocapture)

unit-contract-fast: ## Fast unit/contract checks for critical crates.
	$(call RUN_IN_ISOLATE,./bin/require-isolate >/dev/null; export TZ=UTC LC_ALL=C CARGO_TARGET_DIR="$$ISO_ROOT/target-test"; if command -v sccache >/dev/null 2>&1; then export RUSTC_WRAPPER="$$(command -v sccache)"; fi; cargo test -p bijux-dna-runner --lib -- --nocapture; cargo test -p bijux-dna-planner-fastq --lib -- --nocapture; cargo test -p bijux-dna-planner-bam --lib -- --nocapture; cargo test -p bijux-dna-stages-fastq --lib -- --nocapture; cargo test -p bijux-dna-stages-bam --lib -- --nocapture; cargo test -p bijux-dna-api --lib -- --nocapture)

release-readiness: ## Block merges on experimental tools, unknown metrics schemas, or floating pins.
	$(MAKE) registry-lint
	$(call RUN_IN_ISOLATE,./bin/require-isolate >/dev/null; export TZ=UTC LC_ALL=C CARGO_TARGET_DIR="$$ISO_ROOT/target-test"; if command -v sccache >/dev/null 2>&1; then export RUSTC_WRAPPER="$$(command -v sccache)"; fi; cargo test -p bijux-dna-policies --test contracts profiles_release_readiness_gate -- --nocapture; cargo test -p bijux-dna-policies --test contracts reference_adna_profile_uses_production_tools_only -- --nocapture)

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
	@./bin/isolate cargo test -p bijux-dna-policies
	$(MAKE) domain-gates

domain-validate:
	./scripts/domain/validate.sh

domain-coverage:
	@./bin/isolate cargo run -p bijux-dna --bin bijux -- dna domain coverage --domain-dir domain

domain-inventory-drift:
	./scripts/domain/inventory-drift.sh

snapshots:
	@./bin/isolate cargo insta test --workspace

snapshots-accept:
	@./bin/isolate cargo insta accept --workspace

snapshots-review:
	@./bin/isolate cargo insta review

fix-snapshots: ## Rebuild and accept workspace snapshots with the CI insta workflow.
	@./bin/isolate cargo insta test --workspace
	@./bin/isolate cargo insta accept --workspace

test-triage: ## Group failed tests from a saved nextest log.
	@./scripts/test/test-triage.sh artifacts/test-logs/latest.log

generate-configs:
	@./scripts/tooling/generate-configs.sh

check-generated-configs:
	./scripts/checks/check-generated-configs.sh

check-generated-config-headers:
	./scripts/checks/check-generated-config-headers.sh

policy-no-raw-cargo: ## Fail if raw cargo invocations exist in Make/scripts.
	./scripts/checks/check-no-raw-cargo-in-makefiles.sh
	./scripts/checks/check-no-raw-cargo-in-scripts.sh

policy-index: ## Generate policy index under artifacts/.
	@./scripts/tooling/generate-policy-index.sh

policy-only-fast-gate: ## Compile+run policies and critical contract crates only.
	$(call RUN_IN_ISOLATE,./bin/require-isolate >/dev/null; export TZ=UTC LC_ALL=C CARGO_TARGET_DIR="$$ISO_ROOT/target-test"; cargo test -p bijux-dna-policies --test contracts --test boundaries --test determinism -- --nocapture; cargo test -p bijux-dna-core --test contracts -- --nocapture; cargo test -p bijux-dna-pipelines --test contracts -- --nocapture; cargo test -p bijux-dna-runtime --test contracts -- --nocapture)

scripts-inventory: ## Generate scripts inventory under artifacts/
	@./scripts/tooling/inventory.sh

config-inventory: ## Generate config inventory under artifacts/
	@./scripts/tooling/config-inventory.sh

smoke-fastq: ## Quick local FASTQ smoke dry-run.
	@./scripts/smoke/run.sh fastq

smoke-bam: ## Quick local BAM smoke dry-run.
	@./scripts/smoke/run.sh bam

.PHONY: fmt lint test audit coverage ci check verify-parallel-isolation \
		clean-isolates \
		domain-gates \
		domain-validate domain-coverage domain-inventory-drift generate-configs check-generated-configs check-generated-config-headers \
		policy-fast ssot-policy-fast policy-full policy-no-raw-cargo test-profile-invariants registry-lint unit-contract-fast release-readiness ci-fast ci-slow quick install-ci-tools \
		snapshots snapshots-accept snapshots-review fix-snapshots test-triage scripts-inventory config-inventory smoke-fastq smoke-bam test-slow policy-index policy-only-fast-gate
