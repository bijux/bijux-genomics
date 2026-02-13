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
	@if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then \
		$(MAKE) _push-lunarc; \
	else \
		echo "skip push: current directory is not a git worktree"; \
	fi
	@ssh "$(LUNARC_HOST)" 'set -euo pipefail; \
		cd "$(LUNARC_REPO_DIR)"; \
		mkdir -p "$(LUNARC_APPTAINER_DIR)/base" "$(LUNARC_APPTAINER_DIR)/logs"; \
		apptainer build --force "$(LUNARC_APPTAINER_DIR)/base/ubuntu-jammy.sif" docker://ubuntu:22.04; \
		apptainer build --force "$(LUNARC_APPTAINER_DIR)/base/python-3.11-slim.sif" docker://python:3.11-slim; \
		./bin/isolate --tag "$(LUNARC_APPTAINER_BUILD_TAG)" env \
			BIJUX_WORKERS=1 JOBS="$(LUNARC_APPTAINER_JOBS)" \
			FRONTEND_PROOF_MODE=1 \
			SMOKE_LEVEL=build \
			VM_OUT_DIR="$(LUNARC_APPTAINER_DIR)" \
			ARTIFACT_DIR="$(LUNARC_APPTAINER_DIR)" \
			APPTAINER_UBUNTU_BASE_SIF="$(LUNARC_APPTAINER_DIR)/base/ubuntu-jammy.sif" \
			APPTAINER_PYTHON_BASE_SIF="$(LUNARC_APPTAINER_DIR)/base/python-3.11-slim.sif" \
			scripts/containers/smoke-apptainer.sh | tee "$(LUNARC_APPTAINER_DIR)/logs/build-all-j$(LUNARC_APPTAINER_JOBS).log"'

apptainer-lunarc-test: ## Run contract smoke test for all apptainer tools on Lunarc frontend
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
	@mkdir -p "$(LUNARC_LOCAL_APPTAINER_DIR)"
	@rsync -az --delete \
		"$(LUNARC_HOST):$(LUNARC_APPTAINER_DIR)/" \
		"$(LUNARC_LOCAL_APPTAINER_DIR)/"
	@echo "pulled_to=$(LUNARC_LOCAL_APPTAINER_DIR)"

.PHONY: _push-lunarc push-lunarc push-lunarc-confirm _pull-lunarc pull-lunarc _pull-lunarc-results pull-lunarc-results apptainer-lunarc-build apptainer-lunarc-test apptainer-lunarc-pull
