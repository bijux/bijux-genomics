SHELL := /bin/sh

##@ Code Formatting

fmt-check: ## Check code formatting
	cargo fmt --all -- --check

fmt: ## Format all code
	cargo fmt --all

##@ Linting

clippy: ## Run Clippy on core crates only (fast)
	CARGO_BUILD_JOBS=$(JOBS) cargo clippy \
		-p bijux-core -p bijux-engine -p bijux-api -p bijux \
		--lib --bins --no-deps -- -D warnings

lint: fmt-check clippy ## Run standard lint (fmt-check + core clippy)

lint-full: fmt-check ## Run exhaustive Clippy on entire workspace
	cargo clippy --workspace --all-targets --all-features -- -D warnings

lint-fast: ## Run quick workspace lint
	@echo "Checking formatting..."
	cargo fmt --all -- --check
	@echo "Running Clippy (workspace)..."
	CARGO_BUILD_JOBS=$(JOBS) cargo clippy --workspace --all-targets -- -D warnings

lint-isolate: ## Run standard lint with isolated target dir
	CARGO_TARGET_DIR=target-isolate $(MAKE) lint

lint-full-isolate: ## Run full lint with isolated target dir
	CARGO_TARGET_DIR=target-isolate $(MAKE) lint-full

##@ Testing

define NEXTEST_OR_TEST
@if command -v cargo-nextest >/dev/null 2>&1; then \
	echo "$(1)"; \
	CARGO_BUILD_JOBS=$(JOBS) cargo nextest run --workspace $(2) --no-fail-fast --jobs $(NEXTEST_JOBS); \
else \
	echo "cargo-nextest not installed; falling back to cargo test"; \
	CARGO_BUILD_JOBS=$(JOBS) cargo test --workspace $(3) --no-fail-fast -- --color always; \
fi
endef

test: ## Run standard tests (default features)
	$(call NEXTEST_OR_TEST,Running standard tests with nextest...,"", "")

test-ci: ## Run fast CI tests with nextest profile
	@if command -v cargo-nextest >/dev/null 2>&1; then \
		echo "Running CI tests with nextest profile..."; \
		CARGO_BUILD_JOBS=$(JOBS) cargo nextest run --workspace --profile ci --no-fail-fast --jobs $(NEXTEST_JOBS); \
	else \
		echo "cargo-nextest not installed; falling back to cargo test"; \
		CARGO_BUILD_JOBS=$(JOBS) cargo test --workspace --no-fail-fast -- --color always; \
	fi

test-full: ## Run tests with all features
	$(call NEXTEST_OR_TEST,Running tests with all features...,--all-features,--all-features)

test-fast: ## Run fast full-feature tests (alias for test-full)
	$(call NEXTEST_OR_TEST,Running fast tests with all features...,--all-features,--all-features)

##@ Isolated Target Dir

test-isolate: ## Run standard tests with isolated target dir
	CARGO_TARGET_DIR=target-isolate $(MAKE) test

test-full-isolate: ## Run full-feature tests with isolated target dir
	CARGO_TARGET_DIR=target-isolate $(MAKE) test-full

test-siolate: test-isolate ## Back-compat typo: run standard tests in isolated target dir

test-fukll-siolate: test-full-isolate ## Back-compat typo: run full tests in isolated target dir

test-slow: ## Run only slow tests
	@if command -v cargo-nextest >/dev/null 2>&1; then \
		echo "Running slow tests with nextest..."; \
		CARGO_BUILD_JOBS=$(JOBS) cargo nextest run --workspace --all-features \
			--run-ignored ignored-only -E 'test(/_slow_/)' --jobs $(NEXTEST_JOBS); \
	else \
		echo "cargo-nextest not installed; running all ignored tests as fallback"; \
		CARGO_BUILD_JOBS=$(JOBS) cargo test --workspace --all-features --ignored -- --color always; \
	fi

test-e2e: ## Run end-to-end lab harness (requires CORPUS_ROOT)
	@echo "Use 'make lab-fastq' with CORPUS_ROOT set (see scripts/lab/README.md)."
	@exit 1

test-science: ## Run science-specific tests
	@if command -v cargo-nextest >/dev/null 2>&1; then \
		echo "Running science tests with nextest..."; \
		CARGO_BUILD_JOBS=$(JOBS) cargo nextest run --workspace --all-features \
			--run-ignored ignored-only -E 'test(/_science_/)' --jobs $(NEXTEST_JOBS); \
	else \
		echo "cargo-nextest not installed; running all ignored tests as fallback"; \
		CARGO_BUILD_JOBS=$(JOBS) cargo test --workspace --all-features --ignored -- --color always; \
	fi

##@ Verification & Quality

msrv: ## Verify minimum supported Rust version
	cargo check --workspace --all-targets

guardrails: ## Run architectural guardrail tests
	@if command -v cargo-nextest >/dev/null 2>&1; then \
		echo "Running guardrail tests with nextest..."; \
		CARGO_BUILD_JOBS=$(JOBS) cargo nextest run --workspace --all-features \
			-E 'test(/(no_deep_modules_in_src|file_loc_budget|no_giant_file|no_garbage_module_names|owner_guardrail|public_api_is_small|no_cross_layer_calls|no_new_top_level_modules_without_owner)/)' \
			--jobs $(NEXTEST_JOBS); \
	else \
		echo "cargo-nextest not installed; running partial guardrails with cargo test"; \
		CARGO_BUILD_JOBS=$(JOBS) cargo test --workspace -- \
			--color always no_deep_modules_in_src file_loc_budget no_giant_file; \
	fi

structure-check: ## Run repository policy snapshot tests
	cargo test -p bijux-policies --test workspace --test policy_snapshot

policies: ## Run policy test suite
	cargo test -p bijux-policies

coverage: ## Generate test coverage report (prefers nextest)
	@if command -v cargo-llvm-cov >/dev/null 2>&1; then \
		if command -v cargo-nextest >/dev/null 2>&1; then \
			echo "Generating coverage with cargo-llvm-cov + nextest..."; \
			CARGO_BUILD_JOBS=$(JOBS) cargo llvm-cov nextest run --workspace --all-features --html --no-fail-fast --jobs $(NEXTEST_JOBS); \
		else \
			echo "Generating coverage with cargo-llvm-cov..."; \
			CARGO_BUILD_JOBS=$(JOBS) cargo llvm-cov --workspace --all-features --html; \
		fi; \
		echo "HTML report: target/llvm-cov/html/index.html"; \
	else \
		echo "cargo-llvm-cov not installed; skipping coverage"; \
	fi

audit: ## Run security and dependency audits
	@if command -v cargo-audit >/dev/null 2>&1; then \
		if [ -f audit-allowlist.toml ]; then \
			cargo audit --file audit-allowlist.toml; \
		else \
			cargo audit; \
		fi; \
	else \
		echo "cargo-audit not installed; skipping advisory check"; \
	fi
	@if command -v cargo-deny >/dev/null 2>&1; then \
		cargo deny check; \
	else \
		echo "cargo-deny not installed; skipping deny check"; \
	fi
	@if command -v cargo-machete >/dev/null 2>&1; then \
		cargo machete; \
	else \
		echo "cargo-machete not installed; skipping unused dependency check"; \
	fi

guardrails-isolate: ## Run guardrail tests with isolated target dir
	CARGO_TARGET_DIR=target-isolate $(MAKE) guardrails

structure-check-isolate: ## Run policy snapshot tests with isolated target dir
	CARGO_TARGET_DIR=target-isolate $(MAKE) structure-check

coverage-isolate: ## Generate coverage with isolated target dir
	CARGO_TARGET_DIR=target-isolate $(MAKE) coverage

audit-isolate: ## Run audits with isolated target dir
	CARGO_TARGET_DIR=target-isolate $(MAKE) audit

clean-isolate: ## Remove isolated target dir
	rm -rf target-isolate

isolate: ## Run full CI essentials in isolated target dir
	CARGO_TARGET_DIR=target-isolate $(MAKE) lint-full test-full guardrails structure-check audit coverage

.PHONY: fmt fmt-check clippy lint lint-full lint-fast \
        test test-full test-fast test-slow test-e2e test-science \
        test-isolate test-full-isolate test-siolate test-fukll-siolate \
        lint-isolate lint-full-isolate guardrails-isolate structure-check-isolate \
        coverage-isolate audit-isolate clean-isolate isolate \
        msrv guardrails structure-check coverage audit
