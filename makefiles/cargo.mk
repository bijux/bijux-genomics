SHELL := /bin/sh

##@ Code Formatting

fmt: ## Format all code
	cargo fmt --all

##@ Linting

lint: ## Run standard lint (fmt-check + core clippy)
	cargo fmt --all -- --check
	CARGO_BUILD_JOBS=$(JOBS) cargo clippy \
		-p bijux-core -p bijux-engine -p bijux-api -p bijux \
		--lib --bins --no-deps -- -D warnings

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

##@ Verification & Quality

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

coverage: ## Generate test coverage report (prefers nextest)
	@if command -v cargo-llvm-cov >/dev/null 2>&1; then \
		if command -v cargo-nextest >/dev/null 2>&1; then \
			echo "Generating coverage with cargo-llvm-cov + nextest..."; \
			CARGO_BUILD_JOBS=$(JOBS) cargo llvm-cov nextest run --workspace --no-fail-fast --jobs $(NEXTEST_JOBS); \
			CARGO_BUILD_JOBS=$(JOBS) cargo llvm-cov report --json > target/llvm-cov/coverage.json; \
			python3 scripts/coverage_summary.py target/llvm-cov/coverage.json; \
		else \
			echo "Generating coverage with cargo-llvm-cov..."; \
			CARGO_BUILD_JOBS=$(JOBS) cargo llvm-cov --workspace; \
			CARGO_BUILD_JOBS=$(JOBS) cargo llvm-cov report --json > target/llvm-cov/coverage.json; \
			python3 scripts/coverage_summary.py target/llvm-cov/coverage.json; \
		fi; \
	else \
		echo "cargo-llvm-cov not installed; skipping coverage"; \
	fi

coverage-html: ## Generate HTML coverage report
	@if command -v cargo-llvm-cov >/dev/null 2>&1; then \
		if command -v cargo-nextest >/dev/null 2>&1; then \
			echo "Generating coverage HTML with cargo-llvm-cov + nextest..."; \
			CARGO_BUILD_JOBS=$(JOBS) cargo llvm-cov nextest run --workspace --no-fail-fast --jobs $(NEXTEST_JOBS); \
			CARGO_BUILD_JOBS=$(JOBS) cargo llvm-cov report --html; \
		else \
			echo "Generating coverage HTML with cargo-llvm-cov..."; \
			CARGO_BUILD_JOBS=$(JOBS) cargo llvm-cov --workspace; \
			CARGO_BUILD_JOBS=$(JOBS) cargo llvm-cov report --html; \
		fi; \
		echo "HTML report: target/llvm-cov/html/index.html"; \
	else \
		echo "cargo-llvm-cov not installed; skipping coverage"; \
	fi

##@ Isolated Target Dir

fmt-isolate: ## Format all code with isolated target dir
	CARGO_TARGET_DIR=target-isolate $(MAKE) fmt

lint-isolate: ## Run standard lint with isolated target dir
	CARGO_TARGET_DIR=target-isolate $(MAKE) lint

test-isolate: ## Run standard tests with isolated target dir
	CARGO_TARGET_DIR=target-isolate $(MAKE) test

audit-isolate: ## Run audits with isolated target dir
	CARGO_TARGET_DIR=target-isolate $(MAKE) audit

coverage-isolate: ## Generate coverage with isolated target dir
	CARGO_TARGET_DIR=target-isolate $(MAKE) coverage

##@ CI

ci: ## Run CI gates without redundancy
	$(MAKE) lint test audit coverage

ci-isolate: ## Run CI gates in isolated target dir
	CARGO_TARGET_DIR=target-isolate $(MAKE) ci

.PHONY: fmt lint test audit coverage \
        fmt-isolate lint-isolate test-isolate audit-isolate coverage-isolate \
        coverage-html ci ci-isolate
