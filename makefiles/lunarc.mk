##@ Lunarc Sync

LUNARC_HOST ?= lunarc
LUNARC_ROOT ?= /home/bijan/bijux
LUNARC_REPO_DIR ?= $(LUNARC_ROOT)/bijux-dna
LUNARC_PULL_BASE ?= $(HOME)/bijux
CLEAN_CONTEXT ?= 1
ALLOW_DIRTY ?= 0
INCLUDE_CONTAINERS_MANIFEST ?= 0
DATA_MANIFEST_GLOB ?=
LUNARC_APPTAINER_DIR ?= $(LUNARC_ROOT)/bijux-dna-apptainer
LUNARC_LOCAL_APPTAINER_DIR ?= ../bijux-dna-lunarc/bijux-dna-apptainer
LUNARC_APPTAINER_JOBS ?= 10
LUNARC_APPTAINER_BUILD_TAG ?= hpc-all71-j10
LUNARC_FRONTEND_SENTINEL ?= /home/bijan/bijux/bijux-dna
LUNARC_APPTAINER_BASE_SEED_DIR ?= /home/bijan/bijux/apptainer-build/base

_push-lunarc: ## Push repo to Lunarc with safety checks and remote git status
	@LUNARC_HOST="$(LUNARC_HOST)" \
	LUNARC_ROOT="$(LUNARC_ROOT)" \
	LUNARC_REPO_DIR="$(LUNARC_REPO_DIR)" \
	CLEAN_CONTEXT="$(CLEAN_CONTEXT)" \
	ALLOW_DIRTY="$(ALLOW_DIRTY)" \
	./scripts/run.sh hpc lunarc/push

push-lunarc: _push-lunarc ## Public alias for pushing repo to Lunarc

push-lunarc-confirm: ## Push repo to Lunarc (executes --confirm)
	@LUNARC_HOST="$(LUNARC_HOST)" \
	LUNARC_ROOT="$(LUNARC_ROOT)" \
	LUNARC_REPO_DIR="$(LUNARC_REPO_DIR)" \
	CLEAN_CONTEXT="$(CLEAN_CONTEXT)" \
	ALLOW_DIRTY="$(ALLOW_DIRTY)" \
	./scripts/run.sh hpc lunarc/push --confirm

_pull-lunarc: ## Pull from Lunarc into timestamped local dir (default mode: results)
	@LUNARC_HOST="$(LUNARC_HOST)" \
	LUNARC_ROOT="$(LUNARC_ROOT)" \
	LUNARC_REPO_DIR="$(LUNARC_REPO_DIR)" \
	LUNARC_PULL_BASE="$(LUNARC_PULL_BASE)" \
	INCLUDE_CONTAINERS_MANIFEST="$(INCLUDE_CONTAINERS_MANIFEST)" \
	DATA_MANIFEST_GLOB="$(DATA_MANIFEST_GLOB)" \
	PULL_MODE="results" \
	./scripts/run.sh hpc lunarc/pull

pull-lunarc: _pull-lunarc ## Public alias for pull from Lunarc

_pull-lunarc-results: ## Recommended: pull results + optional manifests only
	@LUNARC_HOST="$(LUNARC_HOST)" \
	LUNARC_ROOT="$(LUNARC_ROOT)" \
	LUNARC_REPO_DIR="$(LUNARC_REPO_DIR)" \
	LUNARC_PULL_BASE="$(LUNARC_PULL_BASE)" \
	INCLUDE_CONTAINERS_MANIFEST="$(INCLUDE_CONTAINERS_MANIFEST)" \
	DATA_MANIFEST_GLOB="$(DATA_MANIFEST_GLOB)" \
	PULL_MODE="results" \
	./scripts/run.sh hpc lunarc/pull

pull-lunarc-results: _pull-lunarc-results ## Public alias for pull results from Lunarc

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
		mkdir -p "$(LUNARC_APPTAINER_DIR)/base" "$(LUNARC_APPTAINER_DIR)/logs"; \
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
		./bin/isolate --tag "$(LUNARC_APPTAINER_BUILD_TAG)" env \
			BIJUX_WORKERS=1 JOBS="$(LUNARC_APPTAINER_JOBS)" \
			FRONTEND_PROOF_MODE=1 \
			SMOKE_LEVEL=build \
			VM_OUT_DIR="$(LUNARC_APPTAINER_DIR)" \
			ARTIFACT_DIR="$(LUNARC_APPTAINER_DIR)" \
			APPTAINER_UBUNTU_BASE_SIF="$(LUNARC_APPTAINER_DIR)/base/ubuntu-jammy.sif" \
			$$py_arg \
			scripts/containers/smoke-apptainer.sh | tee "$(LUNARC_APPTAINER_DIR)/logs/build-all-j$(LUNARC_APPTAINER_JOBS).log"'

apptainer-lunarc-test: ## Run contract smoke test for all apptainer tools on Lunarc frontend
	@if [ "$$(hostname -f 2>/dev/null || hostname)" != "$(LUNARC_HOST)" ] && [ "$$(hostname -s 2>/dev/null || hostname)" != "$(LUNARC_HOST)" ]; then :; else \
		echo "refusing local-ssh target on frontend host; use: make apptainer-hpc-test"; \
		exit 2; \
	fi
	@ssh "$(LUNARC_HOST)" 'set -euo pipefail; \
		cd "$(LUNARC_REPO_DIR)"; \
		mkdir -p "$(LUNARC_APPTAINER_DIR)/logs"; \
		./bin/isolate --tag "$(LUNARC_APPTAINER_BUILD_TAG)-test" env \
			BIJUX_WORKERS=1 JOBS="$(LUNARC_APPTAINER_JOBS)" \
			FRONTEND_PROOF_MODE=1 \
			SMOKE_LEVEL=contract \
			VM_OUT_DIR="$(LUNARC_APPTAINER_DIR)" \
			ARTIFACT_DIR="$(LUNARC_APPTAINER_DIR)" \
			APPTAINER_UBUNTU_BASE_SIF="$(LUNARC_APPTAINER_DIR)/base/ubuntu-jammy.sif" \
			APPTAINER_PYTHON_BASE_SIF="$(LUNARC_APPTAINER_DIR)/base/python-3.11-slim.sif" \
			scripts/containers/smoke-apptainer.sh | tee "$(LUNARC_APPTAINER_DIR)/logs/smoke-all-j$(LUNARC_APPTAINER_JOBS).log"; \
		tail -n 20 "$(LUNARC_APPTAINER_DIR)/logs/apptainer/summary.txt"'

apptainer-lunarc-pull: ## Pull Lunarc apptainer artifacts into ../bijux-dna-lunarc/bijux-dna-apptainer
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
		./bin/isolate --tag "$(LUNARC_APPTAINER_BUILD_TAG)" env \
			BIJUX_WORKERS=1 JOBS="$(LUNARC_APPTAINER_JOBS)" \
			FRONTEND_PROOF_MODE=1 \
			SMOKE_LEVEL=build \
			VM_OUT_DIR="$(LUNARC_APPTAINER_DIR)" \
			ARTIFACT_DIR="$(LUNARC_APPTAINER_DIR)" \
			APPTAINER_UBUNTU_BASE_SIF="$(LUNARC_APPTAINER_DIR)/base/ubuntu-jammy.sif" \
			$$py_arg \
			scripts/containers/smoke-apptainer.sh | tee "$(LUNARC_APPTAINER_DIR)/logs/build-all-j$(LUNARC_APPTAINER_JOBS).log"

apptainer-hpc-test: ## Run contract smoke test directly on HPC frontend (no ssh)
	@if [ -d "$(LUNARC_FRONTEND_SENTINEL)" ]; then :; else \
		echo "refusing HPC-native target off frontend; use: make apptainer-lunarc-test"; \
		exit 2; \
	fi
	@set -euo pipefail; \
		mkdir -p "$(LUNARC_APPTAINER_DIR)/logs"; \
		py_arg=""; \
		if [ -s "$(LUNARC_APPTAINER_DIR)/base/python-3.11-slim.sif" ]; then py_arg="APPTAINER_PYTHON_BASE_SIF=$(LUNARC_APPTAINER_DIR)/base/python-3.11-slim.sif"; fi; \
		./bin/isolate --tag "$(LUNARC_APPTAINER_BUILD_TAG)-test" env \
			BIJUX_WORKERS=1 JOBS="$(LUNARC_APPTAINER_JOBS)" \
			FRONTEND_PROOF_MODE=1 \
			SMOKE_LEVEL=contract \
			VM_OUT_DIR="$(LUNARC_APPTAINER_DIR)" \
			ARTIFACT_DIR="$(LUNARC_APPTAINER_DIR)" \
			APPTAINER_UBUNTU_BASE_SIF="$(LUNARC_APPTAINER_DIR)/base/ubuntu-jammy.sif" \
			$$py_arg \
			scripts/containers/smoke-apptainer.sh | tee "$(LUNARC_APPTAINER_DIR)/logs/smoke-all-j$(LUNARC_APPTAINER_JOBS).log"; \
		tail -n 20 "$(LUNARC_APPTAINER_DIR)/logs/apptainer/summary.txt"

apptainer-hpc-clean: ## Remove frontend apptainer output dir
	@set -euo pipefail; \
		rm -rf "$(LUNARC_APPTAINER_DIR)"; \
		echo "removed $(LUNARC_APPTAINER_DIR)"

.PHONY: _push-lunarc push-lunarc push-lunarc-confirm _pull-lunarc pull-lunarc _pull-lunarc-results pull-lunarc-results apptainer-lunarc-build apptainer-lunarc-test apptainer-lunarc-pull apptainer-hpc-build apptainer-hpc-test apptainer-hpc-clean
