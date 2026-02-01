SHELL 			:= /bin/sh
PLATFORM 		?= docker-mac-arm64
JOBS 			?= 8
NEXTEST_JOBS 	?= $(JOBS)
TOOLS_TRIM 		?= fastp,cutadapt,bbduk,adapterremoval,trimmomatic,trim_galore,atropos
TOOLS_VALIDATE 	?= seqtk,fastqc,fastqvalidator,fastqvalidator_official,fqtools
TOOLS_FILTER 	?= prinseq,fastp,seqkit
TOOLS_MERGE 	?= pear,vsearch,bbmerge,flash2
TOOLS_CORRECT 	?= rcorrector
TOOLS_QC_POST 	?= fastqc,multiqc
TOOLS_UMI 		?= umi_tools
TOOLS_STATS 	?= seqkit_stats
TOOLS_SCREEN 	?= kraken2,centrifuge,metaphlan,kaiju,fastq_screen

EXTRA_GOALS := $(filter-out bench-all benchmark-validate benchmark-trim benchmark-merge benchmark-correct benchmark-filter benchmark-stats benchmark-qc-post benchmark-umi benchmark-screen benchmark-preprocess image-qa build-images test-images test-images-trim test-images-validate test-images-filter test-images-merge lint security test coverage test-fast test-slow test-e2e guardrails mac-ci,$(MAKECMDGOALS))
EXTRA_FASTQ_ROOTS := $(EXTRA_GOALS)
FASTQ_ROOT_OVERRIDE ?= $(EXTRA_FASTQ_ROOTS)

.PHONY: build-images test-images image-qa bench-all benchmark-trim benchmark-validate benchmark-filter benchmark-merge \
	benchmark-correct benchmark-qc-post benchmark-umi benchmark-stats benchmark-screen benchmark-preprocess \
	test-images-trim test-images-validate test-images-filter test-images-merge lint quality security test \
	test-fast test-slow test-e2e guardrails mac-ci mac-ci-fast mac-ci-full lint-fast test-full

test:
	@if command -v cargo-nextest >/dev/null 2>&1; then \
       echo "Running tests with nextest..."; \
       CARGO_BUILD_JOBS=$(JOBS) CARGO_INCREMENTAL=1 cargo nextest run --workspace --no-fail-fast --jobs $(NEXTEST_JOBS); \
    else \
       echo "cargo-nextest not installed; falling back to cargo test"; \
       CARGO_BUILD_JOBS=$(JOBS) CARGO_INCREMENTAL=1 cargo test --workspace -- --color always; \
    fi

test-full:
	@if command -v cargo-nextest >/dev/null 2>&1; then \
       echo "Running full tests with nextest..."; \
       CARGO_BUILD_JOBS=$(JOBS) CARGO_INCREMENTAL=1 cargo nextest run --workspace --all-features --no-fail-fast --jobs $(NEXTEST_JOBS); \
    else \
       echo "cargo-nextest not installed; falling back to cargo test"; \
       CARGO_BUILD_JOBS=$(JOBS) CARGO_INCREMENTAL=1 cargo test --workspace --all-features -- --color always; \
    fi

test-fast:
	@if command -v cargo-nextest >/dev/null 2>&1; then \
       echo "Running fast tests with nextest..."; \
       CARGO_BUILD_JOBS=$(JOBS) CARGO_INCREMENTAL=1 cargo nextest run --workspace --all-features --no-fail-fast --jobs $(NEXTEST_JOBS); \
    else \
       echo "cargo-nextest not installed; falling back to cargo test"; \
       CARGO_BUILD_JOBS=$(JOBS) CARGO_INCREMENTAL=1 cargo test --workspace -- --color always; \
    fi

test-slow:
	@if command -v cargo-nextest >/dev/null 2>&1; then \
       echo "Running slow tests with nextest..."; \
       CARGO_BUILD_JOBS=$(JOBS) CARGO_INCREMENTAL=1 cargo nextest run --workspace --all-features --run-ignored ignored-only -E 'test(/_slow_/)' --jobs $(NEXTEST_JOBS); \
    else \
       echo "cargo-nextest not installed; falling back to cargo test (ignored-only)"; \
       CARGO_BUILD_JOBS=$(JOBS) CARGO_INCREMENTAL=1 cargo test --workspace -- --ignored --color always; \
    fi

test-e2e:
	@if command -v cargo-nextest >/dev/null 2>&1; then \
       echo "Running e2e tests with nextest..."; \
       if [ ! -f tests/data/fastq/ERR2112797/ERR2112797_1.fastq.gz ] || [ ! -f tests/data/fastq/ERR2112797/ERR2112797_2.fastq.gz ]; then \
			echo "missing e2e FASTQ fixtures; skipping"; \
			exit 0; \
	   fi; \
       CARGO_BUILD_JOBS=$(JOBS) CARGO_INCREMENTAL=1 BIJUX_E2E=1 cargo nextest run --workspace --all-features --run-ignored ignored-only -E 'test(/_e2e_/)' --jobs $(NEXTEST_JOBS); \
    else \
       echo "cargo-nextest not installed; falling back to cargo test (ignored-only)"; \
       CARGO_BUILD_JOBS=$(JOBS) CARGO_INCREMENTAL=1 BIJUX_E2E=1 cargo test --workspace -- --ignored --color always; \
    fi

guardrails:
	@if command -v cargo-nextest >/dev/null 2>&1; then \
       echo "Running guardrails..."; \
       CARGO_BUILD_JOBS=$(JOBS) CARGO_INCREMENTAL=1 cargo nextest run --workspace --all-features -E 'test(/(no_deep_modules_in_src|file_loc_budget|no_giant_file|no_garbage_module_names|owner_guardrail|public_api_is_small|no_cross_layer_calls|no_new_top_level_modules_without_owner)/)' --jobs $(NEXTEST_JOBS); \
    else \
       echo "cargo-nextest not installed; falling back to cargo test"; \
       CARGO_BUILD_JOBS=$(JOBS) CARGO_INCREMENTAL=1 cargo test --workspace -- --color always no_deep_modules_in_src file_loc_budget no_giant_file; \
    fi

coverage:
	@if command -v cargo-llvm-cov >/dev/null 2>&1; then \
       CARGO_BUILD_JOBS=$(JOBS) CARGO_INCREMENTAL=1 cargo llvm-cov --workspace --all-features --show-missing-lines; \
       echo "Coverage report generated in target/llvm-cov/html/index.html"; \
    else \
       echo "cargo-llvm-cov not installed; skipping coverage"; \
    fi

lint:
	@echo "Checking formatting..."
	cargo fmt --all -- --check
	@echo "Running Clippy (strict)..."
	CARGO_BUILD_JOBS=$(JOBS) CARGO_INCREMENTAL=1 cargo clippy --workspace --all-targets --all-features -- -D warnings
	@if command -v cargo-audit >/dev/null 2>&1; then \
		echo "Checking advisories (cargo-audit)..."; \
		if [ -f audit-allowlist.toml ]; then \
			cargo audit --file audit-allowlist.toml; \
		else \
			cargo audit; \
		fi; \
	else \
		echo "cargo-audit not installed; skipping advisory check"; \
	fi
	@if command -v cargo-deny >/dev/null 2>&1; then \
		echo "Checking licenses/duplicates (cargo-deny)..."; \
		cargo deny check; \
	else \
		echo "cargo-deny not installed; skipping deny check"; \
	fi
	@if command -v cargo-machete >/dev/null 2>&1; then \
		echo "Checking unused dependencies (cargo-machete)..."; \
		cargo machete; \
	else \
		echo "cargo-machete not installed; skipping machete check"; \
	fi

security:
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

lint-fast:
	@echo "Checking formatting..."
	cargo fmt --all -- --check
	@echo "Running Clippy (fast)..."
	CARGO_BUILD_JOBS=$(JOBS) CARGO_INCREMENTAL=1 cargo clippy --workspace --all-targets -- -D warnings

mac-ci-fast:
	@set -e; \
	if command -v sccache >/dev/null 2>&1; then export RUSTC_WRAPPER=sccache; fi; \
	$(MAKE) lint-fast JOBS=$(JOBS) NEXTEST_JOBS=$(NEXTEST_JOBS); \
	$(MAKE) test JOBS=$(JOBS) NEXTEST_JOBS=$(NEXTEST_JOBS);

mac-ci-full:
	@set -e; \
	if command -v sccache >/dev/null 2>&1; then export RUSTC_WRAPPER=sccache; fi; \
	$(MAKE) lint JOBS=$(JOBS) NEXTEST_JOBS=$(NEXTEST_JOBS); \
	$(MAKE) security JOBS=$(JOBS) NEXTEST_JOBS=$(NEXTEST_JOBS); \
	$(MAKE) test-full JOBS=$(JOBS) NEXTEST_JOBS=$(NEXTEST_JOBS); \
	$(MAKE) coverage JOBS=$(JOBS) NEXTEST_JOBS=$(NEXTEST_JOBS);

mac-ci: mac-ci-fast
include makefiles/containers.mk
include makefiles/benchmarks.mk
