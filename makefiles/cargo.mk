NEXTEST_PROFILE ?= ci
NEXTEST_CONFIG ?= --config-file nextest.toml
RUN_IGNORED = --run-ignored all
TEST_FEATURES = --all-features
CARGO_BUILD_JOBS ?= $(JOBS)
NEXTEST_TEST_THREADS ?= $(CARGO_BUILD_JOBS)
COVERAGE_BASELINE = artifacts/coverage/baseline.json
COVERAGE_THRESHOLDS = configs/coverage.toml
COVERAGE_OUT = coverage.json

define RUN_IN_ISOLATE
	@./bin/isolate sh -ceu '$(1)'
endef

fmt:
	$(call RUN_IN_ISOLATE,./bin/require-isolate >/dev/null; cargo fmt --all -- --check)

lint:
	./scripts/check-artifacts-tracked.sh
	./scripts/check-no-target-paths-in-tests.sh
	$(call RUN_IN_ISOLATE,./bin/require-isolate >/dev/null; CARGO_BUILD_JOBS=$(CARGO_BUILD_JOBS) cargo clippy --workspace --all-targets --all-features -- -D warnings)

test:
	$(call RUN_IN_ISOLATE,./bin/require-isolate >/dev/null; ./scripts/check-isolation-contract.sh; ./scripts/check-ssot-guardrails.sh; export TZ=UTC LC_ALL=C TEST_TARGET_DIR="$$ISO_ROOT/target-test" COV_TARGET_DIR="$$ISO_ROOT/target-cov" TEST_TMP_DIR="$$ISO_ROOT/tmp-test" COV_TMP_DIR="$$ISO_ROOT/tmp-cov" TEST_PROFRAW_DIR="$$ISO_ROOT/profraw-test" COV_PROFRAW_DIR="$$ISO_ROOT/profraw-cov" CARGO_TARGET_DIR="$$ISO_ROOT/target-test"; if command -v sccache >/dev/null 2>&1; then export RUSTC_WRAPPER="$$(command -v sccache)"; fi; cargo install cargo-nextest --locked >/dev/null 2>&1 || true; cargo nextest run $(NEXTEST_CONFIG) --workspace $(TEST_FEATURES) --profile $(NEXTEST_PROFILE) --test-threads $(NEXTEST_TEST_THREADS) $(RUN_IGNORED); ./scripts/check-isolation-contract.sh)

audit:
	$(call RUN_IN_ISOLATE,./bin/require-isolate >/dev/null; cargo install cargo-deny --locked >/dev/null 2>&1 || true; cargo deny check)

coverage:
	$(call RUN_IN_ISOLATE,./bin/require-isolate >/dev/null; export TZ=UTC LC_ALL=C TEST_TARGET_DIR="$$ISO_ROOT/target-test" COV_TARGET_DIR="$$ISO_ROOT/target-cov" TEST_TMP_DIR="$$ISO_ROOT/tmp-test" COV_TMP_DIR="$$ISO_ROOT/tmp-cov" TEST_PROFRAW_DIR="$$ISO_ROOT/profraw-test" COV_PROFRAW_DIR="$$ISO_ROOT/profraw-cov" CARGO_TARGET_DIR="$$ISO_ROOT/target-cov"; if command -v sccache >/dev/null 2>&1; then export RUSTC_WRAPPER="$$(command -v sccache)"; fi; cargo install cargo-llvm-cov --locked >/dev/null 2>&1 || true; cargo install cargo-nextest --locked >/dev/null 2>&1 || true; cargo llvm-cov clean; rm -rf "$$ISO_ROOT/coverage"; mkdir -p "$$ISO_ROOT/coverage"; cargo llvm-cov nextest --no-report --no-cfg-coverage $(NEXTEST_CONFIG) --workspace $(TEST_FEATURES) --profile $(NEXTEST_PROFILE) --test-threads $(NEXTEST_TEST_THREADS) $(RUN_IGNORED); cargo llvm-cov report --json --output-path "$$ISO_ROOT/coverage/$(COVERAGE_OUT)"; cargo llvm-cov report --html --output-dir "$$ISO_ROOT/coverage"; test -f "$$ISO_ROOT/coverage/$(COVERAGE_OUT)"; test -f "$$ISO_ROOT/coverage/index.html"; if [ -f $(COVERAGE_BASELINE) ]; then python3 scripts/coverage_summary.py "$$ISO_ROOT/coverage/$(COVERAGE_OUT)" --baseline $(COVERAGE_BASELINE) --check-thresholds $(COVERAGE_THRESHOLDS); else python3 scripts/coverage_summary.py "$$ISO_ROOT/coverage/$(COVERAGE_OUT)" --check-thresholds $(COVERAGE_THRESHOLDS); fi)

domain-gates: domain-validate domain-inventory-drift check-generated-configs check-generated-config-headers

domain-gates-isolate:
	@./bin/isolate $(MAKE) domain-gates
	@./scripts/check-root-pollution.sh

fmt-isolate:
	@./bin/isolate $(MAKE) fmt
	@./scripts/check-root-pollution.sh

lint-isolate:
	@./bin/isolate $(MAKE) lint
	@./scripts/check-root-pollution.sh

test-isolate:
	@./bin/isolate $(MAKE) test
	@./scripts/check-root-pollution.sh

audit-isolate:
	@./bin/isolate $(MAKE) audit
	@./scripts/check-root-pollution.sh

coverage-isolate:
	@./bin/isolate $(MAKE) coverage
	@./scripts/check-root-pollution.sh

ci:
	$(MAKE) fmt-isolate
	$(MAKE) domain-gates-isolate
	$(MAKE) lint-isolate
	$(MAKE) audit-isolate
	$(MAKE) test-isolate
	$(MAKE) docs-isolate
	./scripts/check-root-pollution.sh

check:
	$(MAKE) fmt lint audit coverage

ci-isolate: fmt-isolate domain-gates-isolate lint-isolate audit-isolate test-isolate docs-isolate
	@./scripts/check-root-pollution.sh

test-coverage-isolate-parallel:
	$(MAKE) -j2 test-isolate coverage-isolate

ci-local:
	$(MAKE) -j2 test coverage

verify-parallel-isolation:
	@ISO_TAG=verify-a ./bin/isolate sh -ceu 'echo "$$ISO_ROOT" > artifacts/isolates/.verify_a_path'
	@ISO_TAG=verify-b ./bin/isolate sh -ceu 'echo "$$ISO_ROOT" > artifacts/isolates/.verify_b_path'
	@test "$$(cat artifacts/isolates/.verify_a_path)" != "$$(cat artifacts/isolates/.verify_b_path)"
	@rm -f artifacts/isolates/.verify_a_path artifacts/isolates/.verify_b_path

test-and-coverage: verify-parallel-isolation test coverage

test-coverage-parallel: test-and-coverage

clean-isolates:
	@rm -rf artifacts/isolates/*

policy-fast: ## Run fast policy checks (no snapshots)
	@./bin/isolate cargo test -p bijux-dna-policies --test dependency_graph --test purity_scans --test core_layering --test domain_dependency_policy --test ci_tools_policy --test dev_deps_policy --test heavy_deps_policy
	$(MAKE) domain-gates

ssot-policy-fast: ## Fast-fail SSOT and registry policy checks.
	./scripts/check-ssot-guardrails.sh
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
	$(MAKE) fmt-isolate
	$(MAKE) lint-isolate
	$(MAKE) unit-contract-fast
	$(MAKE) release-readiness
	$(MAKE) test-profile-invariants
	$(MAKE) policy-no-raw-cargo

ci-slow: ## Slow CI tier (manual): heavier integration checks.
	$(MAKE) audit-isolate
	$(MAKE) coverage-isolate
	$(MAKE) docs-isolate
	$(MAKE) domain-gates-isolate
	$(MAKE) release-readiness

quick: ## Quick local gate: fmt + clippy + unit + invariant tests.
	$(MAKE) fmt-isolate
	$(MAKE) lint-isolate
	$(MAKE) test-profile-invariants
	$(MAKE) registry-lint

policy-full: ## Run full policy suite
	@./bin/isolate cargo test -p bijux-dna-policies
	$(MAKE) domain-gates

domain-validate:
	./scripts/domain-validate.sh

domain-coverage:
	@./bin/isolate cargo run -p bijux-dna-cli --bin bijux-dna -- domain coverage --domain-dir domain

domain-inventory-drift:
	./scripts/domain-inventory-drift.sh

snapshots:
	@./bin/isolate cargo insta test --workspace

snapshots-accept:
	@./bin/isolate cargo insta accept --workspace

snapshots-review:
	@./bin/isolate cargo insta review

fix-snapshots: ## Rebuild and accept workspace snapshots with the CI insta workflow.
	@./bin/isolate cargo insta test --workspace
	@./bin/isolate cargo insta accept --workspace

generate-configs:
	@./bin/isolate cargo run -p bijux-dna-domain-compiler --bin compile_domain_configs -- --domain-dir domain --configs-dir configs

check-generated-configs:
	./scripts/check-generated-configs.sh

check-generated-config-headers:
	./scripts/check-generated-config-headers.sh

policy-no-raw-cargo: ## Fail if raw cargo invocations exist in Make/scripts.
	./scripts/check-no-raw-cargo-in-makefiles.sh
	./scripts/check-no-raw-cargo-in-scripts.sh

.PHONY: fmt lint test audit coverage ci check ci-local test-coverage-parallel verify-parallel-isolation \
		test-and-coverage \
		test-coverage-isolate-parallel \
		fmt-isolate lint-isolate test-isolate audit-isolate coverage-isolate ci-isolate clean-isolates \
		domain-gates domain-gates-isolate \
		domain-validate domain-coverage domain-inventory-drift generate-configs check-generated-configs check-generated-config-headers \
		policy-fast ssot-policy-fast policy-full policy-no-raw-cargo test-profile-invariants registry-lint unit-contract-fast release-readiness ci-fast ci-slow quick \
		snapshots snapshots-accept snapshots-review fix-snapshots
