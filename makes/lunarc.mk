##@ Lunarc Sync

BIJUX_BENCH_BIN ?= cargo run -q -p bijux-dna --
BENCHMARK_CONFIG ?= configs/bench/benchmark.toml
BENCHMARK_WORKSPACE_VALUE = BIJUX_BENCHMARK_CONFIG="$(BENCHMARK_CONFIG)" $(BIJUX_BENCH_BIN) bench workspace-value --config "$(BENCHMARK_CONFIG)"

LUNARC_HOST ?= $(shell $(BENCHMARK_WORKSPACE_VALUE) remote.ssh_host)
LUNARC_REPO_DIR ?= $(shell $(BENCHMARK_WORKSPACE_VALUE) remote.repo_root)
LUNARC_RESULTS_DIR ?= $(shell $(BENCHMARK_WORKSPACE_VALUE) remote.results_root)
LUNARC_CORPUS_ROOT ?= $(shell $(BENCHMARK_WORKSPACE_VALUE) remote.corpus_root)
LUNARC_LOCAL_RESULTS_DIR ?= $(shell $(BENCHMARK_WORKSPACE_VALUE) local.results_root)
LUNARC_PULL_BASE ?= $(shell $(BENCHMARK_WORKSPACE_VALUE) sync.defaults.pull_base)
LUNARC_PULL_MODE ?= $(shell $(BENCHMARK_WORKSPACE_VALUE) sync.defaults.pull_mode)
LUNARC_INCLUDE_PROFILE ?= $(shell $(BENCHMARK_WORKSPACE_VALUE) sync.defaults.include_profile)
LUNARC_EXCLUDE_PROFILE ?= $(shell $(BENCHMARK_WORKSPACE_VALUE) sync.defaults.exclude_profile)
CLEAN_CONTEXT ?= $(shell $(BENCHMARK_WORKSPACE_VALUE) sync.defaults.clean_context)
ALLOW_DIRTY ?= $(shell $(BENCHMARK_WORKSPACE_VALUE) sync.defaults.allow_dirty)
INCLUDE_CONTAINERS_MANIFEST ?= $(shell $(BENCHMARK_WORKSPACE_VALUE) sync.defaults.include_containers_manifest)
DATA_MANIFEST_GLOB ?= $(shell $(BENCHMARK_WORKSPACE_VALUE) sync.defaults.data_manifest_glob)
LUNARC_CONTAINERS_ROOT ?= $(shell $(BENCHMARK_WORKSPACE_VALUE) remote.containers_root)
LUNARC_APPTAINER_DIR ?= $(LUNARC_CONTAINERS_ROOT)/apptainer
LUNARC_APPTAINER_ARTIFACT_DIR ?= $(LUNARC_REPO_DIR)/artifacts/containers/hpc/frontend-smoke
LUNARC_LOCAL_APPTAINER_DIR ?= ../bijux-dna-lunarc/bijux-dna-container/apptainer
LUNARC_APPTAINER_JOBS ?= 10
LUNARC_APPTAINER_BUILD_TAG ?= hpc-all71-j10
LUNARC_FRONTEND_SENTINEL ?= $(LUNARC_REPO_DIR)
LUNARC_APPTAINER_BASE_SEED_DIR ?= $(LUNARC_CONTAINERS_ROOT)/base

_push-lunarc: ## Push repo to Lunarc with safety checks and remote git status
	@BENCHMARK_SYNC_CLEAN_CONTEXT="$(CLEAN_CONTEXT)" \
	BENCHMARK_SYNC_ALLOW_DIRTY="$(ALLOW_DIRTY)" \
	cargo run -q -p bijux-dna-dev -- hpc run lunarc/push

push-lunarc: _push-lunarc ## Public alias for pushing repo to Lunarc

push-lunarc-confirm: ## Push repo to Lunarc (executes --confirm)
	@BENCHMARK_SYNC_CLEAN_CONTEXT="$(CLEAN_CONTEXT)" \
	BENCHMARK_SYNC_ALLOW_DIRTY="$(ALLOW_DIRTY)" \
	cargo run -q -p bijux-dna-dev -- hpc run lunarc/push --confirm

_pull-lunarc: ## Pull from Lunarc into timestamped local dir (default mode: results)
	@BENCHMARK_SYNC_PULL_BASE="$(LUNARC_PULL_BASE)" \
	BENCHMARK_SYNC_INCLUDE_CONTAINERS_MANIFEST="$(INCLUDE_CONTAINERS_MANIFEST)" \
	BENCHMARK_SYNC_DATA_MANIFEST_GLOB="$(DATA_MANIFEST_GLOB)" \
	BENCHMARK_SYNC_MODE="$(LUNARC_PULL_MODE)" \
	cargo run -q -p bijux-dna-dev -- hpc run lunarc/pull \
		--include-profile "$(LUNARC_INCLUDE_PROFILE)" \
		--exclude-profile "$(LUNARC_EXCLUDE_PROFILE)"

pull-lunarc: _pull-lunarc ## Public alias for pull from Lunarc

_pull-lunarc-results: ## Recommended: pull results + optional manifests only
	@BENCHMARK_SYNC_PULL_DEST="$(LUNARC_LOCAL_RESULTS_DIR)" \
	BENCHMARK_SYNC_PULL_BASE="$(LUNARC_PULL_BASE)" \
	BENCHMARK_SYNC_INCLUDE_CONTAINERS_MANIFEST="$(INCLUDE_CONTAINERS_MANIFEST)" \
	BENCHMARK_SYNC_DATA_MANIFEST_GLOB="$(DATA_MANIFEST_GLOB)" \
	BENCHMARK_SYNC_MODE="results" \
	cargo run -q -p bijux-dna-dev -- hpc run lunarc/pull \
		--include-profile "$(LUNARC_INCLUDE_PROFILE)" \
		--exclude-profile "$(LUNARC_EXCLUDE_PROFILE)"

pull-lunarc-results: _pull-lunarc-results ## Public alias for pull results from Lunarc

pull-lunarc-results-prune: _pull-lunarc-results ## Pull results locally, then clear remote results payload
	@ssh "$(LUNARC_HOST)" 'set -euo pipefail; \
		mkdir -p "$(LUNARC_RESULTS_DIR)"; \
		find "$(LUNARC_RESULTS_DIR)" -mindepth 1 -maxdepth 1 ! -name site_lock.json -exec rm -rf {} +'

benchmark-lunarc-publication-refresh: ## Pull governed publication inputs, render dossiers, and refresh audits
	@$(MAKE) pull-lunarc-results \
		LUNARC_INCLUDE_PROFILE="pull-benchmark-publication" \
		INCLUDE_CONTAINERS_MANIFEST=1 \
		DATA_MANIFEST_GLOB="benchmark/fastq.screen_taxonomy/read_screening/read_screening/taxonomy_db/lineage.tsv"
	@$(MAKE) _benchmark-normalize-local-results-layout
	@$(MAKE) _benchmark-corpus-01-published-dossiers

lunarc-footprint: ## Report Lunarc frontend footprint and fail above 20 GB
	@ssh "$(LUNARC_HOST)" 'set -euo pipefail; \
		total_kb=0; \
		for dir in "$(LUNARC_REPO_DIR)" "$(LUNARC_CONTAINERS_ROOT)" "$(LUNARC_CORPUS_ROOT)" "$(LUNARC_RESULTS_DIR)"; do \
			size_kb=$$(du -sk "$$dir" 2>/dev/null | awk "{print \$$1}" || true); \
			size_kb=$${size_kb:-0}; \
			total_kb=$$((total_kb + size_kb)); \
			printf "%s\t%s\n" "$$(basename "$$dir")" "$$size_kb"; \
		done; \
		printf "total\t%s\n" "$$total_kb"; \
		limit_kb=$$((20 * 1024 * 1024)); \
		if [ "$$total_kb" -gt "$$limit_kb" ]; then \
			echo "frontend footprint exceeds 20 GB" >&2; \
			exit 2; \
		fi'

lunarc-prune-code: ## Remove transient build residue from the Lunarc repo checkout
	@ssh "$(LUNARC_HOST)" 'set -euo pipefail; \
		rm -rf "$(LUNARC_REPO_DIR)/artifacts" "$(LUNARC_REPO_DIR)/target"; \
		mkdir -p "$(LUNARC_REPO_DIR)/artifacts"'

apptainer-lunarc-build: ## Push repo then build all apptainer SIFs on Lunarc frontend
	@if [ "$$(hostname -f 2>/dev/null || hostname)" != "$(LUNARC_HOST)" ] && [ "$$(hostname -s 2>/dev/null || hostname)" != "$(LUNARC_HOST)" ]; then :; else \
		echo "refusing local-ssh target on frontend host; use: make apptainer-hpc-build"; \
		exit 2; \
	fi
	@if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then \
		$(MAKE) _push-lunarc; \
	else \
		echo "skip push: current directory is not a git worktree"; \
	fi
	@ssh "$(LUNARC_HOST)" 'set -euo pipefail; \
		cd "$(LUNARC_REPO_DIR)"; \
		mkdir -p "$(LUNARC_APPTAINER_DIR)/base" "$(LUNARC_APPTAINER_DIR)/logs" "$(LUNARC_APPTAINER_ARTIFACT_DIR)/logs"; \
		if [ ! -s "$(LUNARC_APPTAINER_DIR)/base/ubuntu-jammy.sif" ]; then \
			apptainer build --force "$(LUNARC_APPTAINER_DIR)/base/ubuntu-jammy.sif" docker://ubuntu:22.04 || echo "warning: ubuntu base pull failed; trying non-docker fallback"; \
		fi; \
		if [ ! -s "$(LUNARC_APPTAINER_DIR)/base/ubuntu-jammy.sif" ]; then \
			curl -fsSL "https://depot.galaxyproject.org/singularity/ubuntu:22.04" -o "$(LUNARC_APPTAINER_DIR)/base/ubuntu-jammy.sif" || true; \
		fi; \
		if [ ! -s "$(LUNARC_APPTAINER_DIR)/base/python-3.11-slim.sif" ]; then \
			apptainer build --force "$(LUNARC_APPTAINER_DIR)/base/python-3.11-slim.sif" docker://python:3.11-slim || echo "warning: python base pull failed; continuing without local python base SIF"; \
		fi; \
		if [ ! -s "$(LUNARC_APPTAINER_DIR)/base/python-3.11-slim.sif" ]; then \
			curl -fsSL "https://depot.galaxyproject.org/singularity/python:3.11" -o "$(LUNARC_APPTAINER_DIR)/base/python-3.11-slim.sif" || true; \
		fi; \
		if [ ! -s "$(LUNARC_APPTAINER_DIR)/base/python-3.11-slim.sif" ] && [ -s "$(LUNARC_APPTAINER_DIR)/base/ubuntu-jammy.sif" ]; then \
			tmp_def="$$(mktemp /tmp/bijux-python-base.XXXXXX.def)"; \
			printf '%s\n' \
				'Bootstrap: localimage' \
				'From: $(LUNARC_APPTAINER_DIR)/base/ubuntu-jammy.sif' \
				'' \
				'%post' \
				'    set -eux' \
				'    export DEBIAN_FRONTEND=noninteractive' \
				'    apt-get update' \
				'    apt-get install -y --no-install-recommends ca-certificates python3 python3-pip' \
				'    rm -rf /var/lib/apt/lists/*' > "$$tmp_def"; \
			apptainer build --force "$(LUNARC_APPTAINER_DIR)/base/python-3.11-slim.sif" "$$tmp_def" || true; \
			rm -f "$$tmp_def"; \
		fi; \
		py_arg=""; \
		if [ -s "$(LUNARC_APPTAINER_DIR)/base/python-3.11-slim.sif" ]; then py_arg="APPTAINER_PYTHON_BASE_SIF=$(LUNARC_APPTAINER_DIR)/base/python-3.11-slim.sif"; fi; \
		env ARTIFACT_ROOT="$(LUNARC_REPO_DIR)/artifacts" \
			BIJUX_WORKERS=1 JOBS="$(LUNARC_APPTAINER_JOBS)" \
			FRONTEND_PROOF_MODE=1 \
			SMOKE_LEVEL=build \
			VM_OUT_DIR="$(LUNARC_APPTAINER_DIR)" \
			ARTIFACT_DIR="$(LUNARC_APPTAINER_ARTIFACT_DIR)" \
			APPTAINER_UBUNTU_BASE_SIF="$(LUNARC_APPTAINER_DIR)/base/ubuntu-jammy.sif" \
			$$py_arg \
			cargo run -q -p bijux-dna-dev -- containers run smoke-apptainer | tee "$(LUNARC_APPTAINER_ARTIFACT_DIR)/logs/build-all-j$(LUNARC_APPTAINER_JOBS).log"; \
		cargo run -q -p bijux-dna-dev -- containers run check-apptainer-frontend-smoke-proof -- "$(LUNARC_APPTAINER_ARTIFACT_DIR)"'

apptainer-lunarc-test: ## Run contract smoke test for all apptainer tools on Lunarc frontend
	@if [ "$$(hostname -f 2>/dev/null || hostname)" != "$(LUNARC_HOST)" ] && [ "$$(hostname -s 2>/dev/null || hostname)" != "$(LUNARC_HOST)" ]; then :; else \
		echo "refusing local-ssh target on frontend host; use: make apptainer-hpc-test"; \
		exit 2; \
	fi
	@ssh "$(LUNARC_HOST)" 'set -euo pipefail; \
		cd "$(LUNARC_REPO_DIR)"; \
		mkdir -p "$(LUNARC_APPTAINER_DIR)/logs" "$(LUNARC_APPTAINER_ARTIFACT_DIR)/logs"; \
		env ARTIFACT_ROOT="$(LUNARC_REPO_DIR)/artifacts" \
			BIJUX_WORKERS=1 JOBS="$(LUNARC_APPTAINER_JOBS)" \
			FRONTEND_PROOF_MODE=1 \
			SMOKE_LEVEL=contract \
			VM_OUT_DIR="$(LUNARC_APPTAINER_DIR)" \
			ARTIFACT_DIR="$(LUNARC_APPTAINER_ARTIFACT_DIR)" \
			APPTAINER_UBUNTU_BASE_SIF="$(LUNARC_APPTAINER_DIR)/base/ubuntu-jammy.sif" \
			APPTAINER_PYTHON_BASE_SIF="$(LUNARC_APPTAINER_DIR)/base/python-3.11-slim.sif" \
			cargo run -q -p bijux-dna-dev -- containers run smoke-apptainer | tee "$(LUNARC_APPTAINER_ARTIFACT_DIR)/logs/smoke-all-j$(LUNARC_APPTAINER_JOBS).log"; \
		cargo run -q -p bijux-dna-dev -- containers run check-apptainer-frontend-smoke-proof -- "$(LUNARC_APPTAINER_ARTIFACT_DIR)"; \
		tail -n 20 "$(LUNARC_APPTAINER_ARTIFACT_DIR)/logs/apptainer/summary.txt"'

apptainer-lunarc-pull: ## Pull Lunarc apptainer artifacts into ../bijux-dna-lunarc/bijux-dna-container/apptainer
	@if [ "$$(hostname -f 2>/dev/null || hostname)" != "$(LUNARC_HOST)" ] && [ "$$(hostname -s 2>/dev/null || hostname)" != "$(LUNARC_HOST)" ]; then :; else \
		echo "refusing pull-to-local target on frontend host; run this from your local machine"; \
		exit 2; \
	fi
	@mkdir -p "$(LUNARC_LOCAL_APPTAINER_DIR)"
	@rsync -az --delete \
		"$(LUNARC_HOST):$(LUNARC_APPTAINER_DIR)/" \
		"$(LUNARC_LOCAL_APPTAINER_DIR)/"
	@echo "pulled_to=$(LUNARC_LOCAL_APPTAINER_DIR)"

apptainer-hpc-build: ## Build all apptainer SIFs directly on HPC frontend (no ssh)
	@if [ -d "$(LUNARC_FRONTEND_SENTINEL)" ]; then :; else \
		echo "refusing HPC-native target off frontend; use: make apptainer-lunarc-build"; \
		exit 2; \
	fi
	@set -euo pipefail; \
		mkdir -p "$(LUNARC_APPTAINER_DIR)/base" "$(LUNARC_APPTAINER_DIR)/logs"; \
		if [ ! -s "$(LUNARC_APPTAINER_DIR)/base/ubuntu-jammy.sif" ] && [ -s "$(LUNARC_APPTAINER_BASE_SEED_DIR)/ubuntu-jammy.sif" ]; then \
			cp -f "$(LUNARC_APPTAINER_BASE_SEED_DIR)/ubuntu-jammy.sif" "$(LUNARC_APPTAINER_DIR)/base/ubuntu-jammy.sif"; \
		fi; \
		if [ ! -s "$(LUNARC_APPTAINER_DIR)/base/python-3.11-slim.sif" ] && [ -s "$(LUNARC_APPTAINER_BASE_SEED_DIR)/python-3.11-slim.sif" ]; then \
			cp -f "$(LUNARC_APPTAINER_BASE_SEED_DIR)/python-3.11-slim.sif" "$(LUNARC_APPTAINER_DIR)/base/python-3.11-slim.sif"; \
		fi; \
		if [ ! -s "$(LUNARC_APPTAINER_DIR)/base/ubuntu-jammy.sif" ]; then \
			apptainer build --force "$(LUNARC_APPTAINER_DIR)/base/ubuntu-jammy.sif" docker://ubuntu:22.04 || echo "warning: ubuntu base pull failed; trying non-docker fallback"; \
		fi; \
		if [ ! -s "$(LUNARC_APPTAINER_DIR)/base/ubuntu-jammy.sif" ]; then \
			curl -fsSL "https://depot.galaxyproject.org/singularity/ubuntu:22.04" -o "$(LUNARC_APPTAINER_DIR)/base/ubuntu-jammy.sif" || true; \
		fi; \
		if [ ! -s "$(LUNARC_APPTAINER_DIR)/base/python-3.11-slim.sif" ]; then \
			apptainer build --force "$(LUNARC_APPTAINER_DIR)/base/python-3.11-slim.sif" docker://python:3.11-slim || echo "warning: python base pull failed; continuing without local python base SIF"; \
		fi; \
		if [ ! -s "$(LUNARC_APPTAINER_DIR)/base/python-3.11-slim.sif" ]; then \
			curl -fsSL "https://depot.galaxyproject.org/singularity/python:3.11" -o "$(LUNARC_APPTAINER_DIR)/base/python-3.11-slim.sif" || true; \
		fi; \
		py_arg=""; \
		if [ -s "$(LUNARC_APPTAINER_DIR)/base/python-3.11-slim.sif" ]; then py_arg="APPTAINER_PYTHON_BASE_SIF=$(LUNARC_APPTAINER_DIR)/base/python-3.11-slim.sif"; fi; \
		env ARTIFACT_ROOT="$(LUNARC_REPO_DIR)/artifacts" \
			BIJUX_WORKERS=1 JOBS="$(LUNARC_APPTAINER_JOBS)" \
			FRONTEND_PROOF_MODE=1 \
			SMOKE_LEVEL=build \
			VM_OUT_DIR="$(LUNARC_APPTAINER_DIR)" \
			ARTIFACT_DIR="$(LUNARC_APPTAINER_DIR)" \
			APPTAINER_UBUNTU_BASE_SIF="$(LUNARC_APPTAINER_DIR)/base/ubuntu-jammy.sif" \
			$$py_arg \
			cargo run -q -p bijux-dna-dev -- containers run smoke-apptainer | tee "$(LUNARC_APPTAINER_DIR)/logs/build-all-j$(LUNARC_APPTAINER_JOBS).log"; \
		cargo run -q -p bijux-dna-dev -- containers run check-apptainer-frontend-smoke-proof -- "$(LUNARC_APPTAINER_DIR)"

apptainer-hpc-test: ## Run contract smoke test directly on HPC frontend (no ssh)
	@if [ -d "$(LUNARC_FRONTEND_SENTINEL)" ]; then :; else \
		echo "refusing HPC-native target off frontend; use: make apptainer-lunarc-test"; \
		exit 2; \
	fi
	@set -euo pipefail; \
		mkdir -p "$(LUNARC_APPTAINER_DIR)/logs"; \
		py_arg=""; \
		if [ -s "$(LUNARC_APPTAINER_DIR)/base/python-3.11-slim.sif" ]; then py_arg="APPTAINER_PYTHON_BASE_SIF=$(LUNARC_APPTAINER_DIR)/base/python-3.11-slim.sif"; fi; \
		env ARTIFACT_ROOT="$(LUNARC_REPO_DIR)/artifacts" \
			BIJUX_WORKERS=1 JOBS="$(LUNARC_APPTAINER_JOBS)" \
			FRONTEND_PROOF_MODE=1 \
			SMOKE_LEVEL=contract \
			VM_OUT_DIR="$(LUNARC_APPTAINER_DIR)" \
			ARTIFACT_DIR="$(LUNARC_APPTAINER_DIR)" \
			APPTAINER_UBUNTU_BASE_SIF="$(LUNARC_APPTAINER_DIR)/base/ubuntu-jammy.sif" \
			$$py_arg \
			cargo run -q -p bijux-dna-dev -- containers run smoke-apptainer | tee "$(LUNARC_APPTAINER_DIR)/logs/smoke-all-j$(LUNARC_APPTAINER_JOBS).log"; \
		cargo run -q -p bijux-dna-dev -- containers run check-apptainer-frontend-smoke-proof -- "$(LUNARC_APPTAINER_DIR)"; \
		tail -n 20 "$(LUNARC_APPTAINER_DIR)/logs/apptainer/summary.txt"

apptainer-hpc-clean: ## Remove frontend apptainer output dir
	@set -euo pipefail; \
		rm -rf "$(LUNARC_APPTAINER_DIR)"; \
		echo "removed $(LUNARC_APPTAINER_DIR)"

.PHONY: _push-lunarc push-lunarc push-lunarc-confirm _pull-lunarc pull-lunarc _pull-lunarc-results pull-lunarc-results pull-lunarc-results-prune benchmark-lunarc-publication-refresh lunarc-footprint lunarc-prune-code apptainer-lunarc-build apptainer-lunarc-test apptainer-lunarc-pull apptainer-hpc-build apptainer-hpc-test apptainer-hpc-clean
