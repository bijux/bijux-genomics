##@ Lunarc Sync

BIJUX_BENCH_BIN ?= cargo run -q -p bijux-dna --
BENCHMARK_CONFIG ?= configs/bench/benchmark.toml
BENCHMARK_WORKSPACE_VALUE = BIJUX_BENCHMARK_CONFIG="$(BENCHMARK_CONFIG)" $(BIJUX_BENCH_BIN) bench workspace-value --config "$(BENCHMARK_CONFIG)"

BENCHMARK_REMOTE_HOST ?= $(shell $(BENCHMARK_WORKSPACE_VALUE) remote.ssh_host)
BENCHMARK_REMOTE_REPO_ROOT ?= $(shell $(BENCHMARK_WORKSPACE_VALUE) remote.repo_root)
BENCHMARK_REMOTE_RESULTS_ROOT ?= $(shell $(BENCHMARK_WORKSPACE_VALUE) remote.results_root)
BENCHMARK_REMOTE_CORPUS_ROOT ?= $(shell $(BENCHMARK_WORKSPACE_VALUE) remote.corpus_root)
BENCHMARK_LOCAL_RESULTS_ROOT ?= $(shell $(BENCHMARK_WORKSPACE_VALUE) local.results_root)
BENCHMARK_PULL_BASE_DEFAULT ?= $(shell $(BENCHMARK_WORKSPACE_VALUE) sync.defaults.pull_base)
BENCHMARK_PULL_MODE_DEFAULT ?= $(shell $(BENCHMARK_WORKSPACE_VALUE) sync.defaults.pull_mode)
BENCHMARK_INCLUDE_PROFILE_DEFAULT ?= $(shell $(BENCHMARK_WORKSPACE_VALUE) sync.defaults.include_profile)
BENCHMARK_EXCLUDE_PROFILE_DEFAULT ?= $(shell $(BENCHMARK_WORKSPACE_VALUE) sync.defaults.exclude_profile)
CLEAN_CONTEXT ?= $(shell $(BENCHMARK_WORKSPACE_VALUE) sync.defaults.clean_context)
ALLOW_DIRTY ?= $(shell $(BENCHMARK_WORKSPACE_VALUE) sync.defaults.allow_dirty)
INCLUDE_CONTAINERS_MANIFEST ?= $(shell $(BENCHMARK_WORKSPACE_VALUE) sync.defaults.include_containers_manifest)
DATA_MANIFEST_GLOB ?= $(shell $(BENCHMARK_WORKSPACE_VALUE) sync.defaults.data_manifest_glob)
BENCHMARK_REMOTE_CONTAINERS_ROOT ?= $(shell $(BENCHMARK_WORKSPACE_VALUE) remote.containers_root)
LUNARC_APPTAINER_DIR ?= $(BENCHMARK_REMOTE_CONTAINERS_ROOT)/apptainer
LUNARC_APPTAINER_ARTIFACT_DIR ?= $(BENCHMARK_REMOTE_REPO_ROOT)/artifacts/containers/hpc/frontend-smoke
LUNARC_LOCAL_APPTAINER_DIR ?= ../bijux-dna-lunarc/bijux-dna-container/apptainer
LUNARC_APPTAINER_JOBS ?= 10
LUNARC_APPTAINER_BUILD_TAG ?= hpc-all71-j10
LUNARC_FRONTEND_SENTINEL ?= $(BENCHMARK_REMOTE_REPO_ROOT)
LUNARC_APPTAINER_BASE_SEED_DIR ?= $(BENCHMARK_REMOTE_CONTAINERS_ROOT)/base

_benchmark-sync-push: ## Push the governed benchmark repo checkout to the remote frontend
	@BENCHMARK_SYNC_CLEAN_CONTEXT="$(CLEAN_CONTEXT)" \
	BENCHMARK_SYNC_ALLOW_DIRTY="$(ALLOW_DIRTY)" \
	cargo run -q -p bijux-dna-dev -- hpc run lunarc/push

benchmark-sync-push: _benchmark-sync-push ## Push the governed benchmark repo checkout to the remote frontend

push-lunarc: benchmark-sync-push ## Compatibility alias for benchmark-sync-push

benchmark-sync-push-confirm: ## Push the governed benchmark repo checkout to the remote frontend with --confirm
	@BENCHMARK_SYNC_CLEAN_CONTEXT="$(CLEAN_CONTEXT)" \
	BENCHMARK_SYNC_ALLOW_DIRTY="$(ALLOW_DIRTY)" \
	cargo run -q -p bijux-dna-dev -- hpc run lunarc/push --confirm

push-lunarc-confirm: benchmark-sync-push-confirm ## Compatibility alias for benchmark-sync-push-confirm

_benchmark-sync-pull: ## Pull the governed benchmark mirror into the default local destination
	@BENCHMARK_SYNC_PULL_BASE="$(BENCHMARK_PULL_BASE_DEFAULT)" \
	BENCHMARK_SYNC_INCLUDE_CONTAINERS_MANIFEST="$(INCLUDE_CONTAINERS_MANIFEST)" \
	BENCHMARK_SYNC_DATA_MANIFEST_GLOB="$(DATA_MANIFEST_GLOB)" \
	BENCHMARK_SYNC_MODE="$(BENCHMARK_PULL_MODE_DEFAULT)" \
	cargo run -q -p bijux-dna-dev -- hpc run lunarc/pull \
		--include-profile "$(BENCHMARK_INCLUDE_PROFILE_DEFAULT)" \
		--exclude-profile "$(BENCHMARK_EXCLUDE_PROFILE_DEFAULT)"

benchmark-sync-pull: _benchmark-sync-pull ## Pull the governed benchmark mirror into the default local destination

pull-lunarc: benchmark-sync-pull ## Compatibility alias for benchmark-sync-pull

_benchmark-sync-pull-results: ## Pull governed benchmark results and optional manifests into the local archive root
	@BENCHMARK_SYNC_PULL_DEST="$(BENCHMARK_LOCAL_RESULTS_ROOT)" \
	BENCHMARK_SYNC_PULL_BASE="$(BENCHMARK_PULL_BASE_DEFAULT)" \
	BENCHMARK_SYNC_INCLUDE_CONTAINERS_MANIFEST="$(INCLUDE_CONTAINERS_MANIFEST)" \
	BENCHMARK_SYNC_DATA_MANIFEST_GLOB="$(DATA_MANIFEST_GLOB)" \
	BENCHMARK_SYNC_MODE="results" \
	cargo run -q -p bijux-dna-dev -- hpc run lunarc/pull \
		--include-profile "$(BENCHMARK_INCLUDE_PROFILE_DEFAULT)" \
		--exclude-profile "$(BENCHMARK_EXCLUDE_PROFILE_DEFAULT)"

benchmark-sync-pull-results: _benchmark-sync-pull-results ## Pull governed benchmark results into the local archive root

pull-lunarc-results: benchmark-sync-pull-results ## Compatibility alias for benchmark-sync-pull-results

benchmark-sync-pull-results-prune: _benchmark-sync-pull-results ## Pull governed results locally, then clear the remote results payload
	@ssh "$(BENCHMARK_REMOTE_HOST)" 'set -euo pipefail; \
		mkdir -p "$(BENCHMARK_REMOTE_RESULTS_ROOT)"; \
		find "$(BENCHMARK_REMOTE_RESULTS_ROOT)" -mindepth 1 -maxdepth 1 ! -name site_lock.json -exec rm -rf {} +'

pull-lunarc-results-prune: benchmark-sync-pull-results-prune ## Compatibility alias for benchmark-sync-pull-results-prune

benchmark-publication-refresh: ## Pull governed publication inputs, render dossiers, and refresh audits
	@$(MAKE) benchmark-sync-pull-results \
		BENCHMARK_INCLUDE_PROFILE_DEFAULT="pull-benchmark-publication" \
		INCLUDE_CONTAINERS_MANIFEST=1 \
		DATA_MANIFEST_GLOB="benchmark/fastq.screen_taxonomy/read_screening/read_screening/taxonomy_db/lineage.tsv"
	@$(MAKE) _benchmark-normalize-local-results-layout
	@$(MAKE) _benchmark-corpus-01-published-dossiers

benchmark-lunarc-publication-refresh: benchmark-publication-refresh ## Compatibility alias for benchmark-publication-refresh

lunarc-footprint: ## Report Lunarc frontend footprint and fail above 20 GB
	@ssh "$(BENCHMARK_REMOTE_HOST)" 'set -euo pipefail; \
		total_kb=0; \
		for dir in "$(BENCHMARK_REMOTE_REPO_ROOT)" "$(BENCHMARK_REMOTE_CONTAINERS_ROOT)" "$(BENCHMARK_REMOTE_CORPUS_ROOT)" "$(BENCHMARK_REMOTE_RESULTS_ROOT)"; do \
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
	@ssh "$(BENCHMARK_REMOTE_HOST)" 'set -euo pipefail; \
		rm -rf "$(BENCHMARK_REMOTE_REPO_ROOT)/artifacts" "$(BENCHMARK_REMOTE_REPO_ROOT)/target"; \
		mkdir -p "$(BENCHMARK_REMOTE_REPO_ROOT)/artifacts"'

apptainer-lunarc-build: ## Push repo then build all apptainer SIFs on Lunarc frontend
	@if [ "$$(hostname -f 2>/dev/null || hostname)" != "$(BENCHMARK_REMOTE_HOST)" ] && [ "$$(hostname -s 2>/dev/null || hostname)" != "$(BENCHMARK_REMOTE_HOST)" ]; then :; else \
		echo "refusing local-ssh target on frontend host; use: make apptainer-hpc-build"; \
		exit 2; \
	fi
	@if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then \
		$(MAKE) _benchmark-sync-push; \
	else \
		echo "skip push: current directory is not a git worktree"; \
	fi
	@ssh "$(BENCHMARK_REMOTE_HOST)" 'set -euo pipefail; \
		cd "$(BENCHMARK_REMOTE_REPO_ROOT)"; \
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
		env ARTIFACT_ROOT="$(BENCHMARK_REMOTE_REPO_ROOT)/artifacts" \
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
	@if [ "$$(hostname -f 2>/dev/null || hostname)" != "$(BENCHMARK_REMOTE_HOST)" ] && [ "$$(hostname -s 2>/dev/null || hostname)" != "$(BENCHMARK_REMOTE_HOST)" ]; then :; else \
		echo "refusing local-ssh target on frontend host; use: make apptainer-hpc-test"; \
		exit 2; \
	fi
	@ssh "$(BENCHMARK_REMOTE_HOST)" 'set -euo pipefail; \
		cd "$(BENCHMARK_REMOTE_REPO_ROOT)"; \
		mkdir -p "$(LUNARC_APPTAINER_DIR)/logs" "$(LUNARC_APPTAINER_ARTIFACT_DIR)/logs"; \
		env ARTIFACT_ROOT="$(BENCHMARK_REMOTE_REPO_ROOT)/artifacts" \
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
	@if [ "$$(hostname -f 2>/dev/null || hostname)" != "$(BENCHMARK_REMOTE_HOST)" ] && [ "$$(hostname -s 2>/dev/null || hostname)" != "$(BENCHMARK_REMOTE_HOST)" ]; then :; else \
		echo "refusing pull-to-local target on frontend host; run this from your local machine"; \
		exit 2; \
	fi
	@mkdir -p "$(LUNARC_LOCAL_APPTAINER_DIR)"
	@rsync -az --delete \
		"$(BENCHMARK_REMOTE_HOST):$(LUNARC_APPTAINER_DIR)/" \
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
		env ARTIFACT_ROOT="$(BENCHMARK_REMOTE_REPO_ROOT)/artifacts" \
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
		env ARTIFACT_ROOT="$(BENCHMARK_REMOTE_REPO_ROOT)/artifacts" \
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

.PHONY: _benchmark-sync-push benchmark-sync-push push-lunarc benchmark-sync-push-confirm push-lunarc-confirm _benchmark-sync-pull benchmark-sync-pull pull-lunarc _benchmark-sync-pull-results benchmark-sync-pull-results pull-lunarc-results benchmark-sync-pull-results-prune pull-lunarc-results-prune benchmark-publication-refresh benchmark-lunarc-publication-refresh lunarc-footprint lunarc-prune-code apptainer-lunarc-build apptainer-lunarc-test apptainer-lunarc-pull apptainer-hpc-build apptainer-hpc-test apptainer-hpc-clean
