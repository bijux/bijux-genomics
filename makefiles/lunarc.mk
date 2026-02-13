##@ Lunarc Sync

LUNARC_HOST ?= lunarc
LUNARC_ROOT ?= $(HOME)/bijux
LUNARC_REPO_DIR ?= $(LUNARC_ROOT)/bijux-dna
LUNARC_PULL_BASE ?= $(HOME)/bijux
CLEAN_CONTEXT ?= 1
ALLOW_DIRTY ?= 0
INCLUDE_CONTAINERS_MANIFEST ?= 0
DATA_MANIFEST_GLOB ?=

push-lunarc: ## Push repo to Lunarc with safety checks and remote git status
	@LUNARC_HOST="$(LUNARC_HOST)" \
	LUNARC_ROOT="$(LUNARC_ROOT)" \
	LUNARC_REPO_DIR="$(LUNARC_REPO_DIR)" \
	CLEAN_CONTEXT="$(CLEAN_CONTEXT)" \
	ALLOW_DIRTY="$(ALLOW_DIRTY)" \
	./scripts/run.sh hpc lunarc/push

pull-lunarc: ## Pull from Lunarc into timestamped local dir (default mode: results)
	@LUNARC_HOST="$(LUNARC_HOST)" \
	LUNARC_ROOT="$(LUNARC_ROOT)" \
	LUNARC_REPO_DIR="$(LUNARC_REPO_DIR)" \
	LUNARC_PULL_BASE="$(LUNARC_PULL_BASE)" \
	INCLUDE_CONTAINERS_MANIFEST="$(INCLUDE_CONTAINERS_MANIFEST)" \
	DATA_MANIFEST_GLOB="$(DATA_MANIFEST_GLOB)" \
	PULL_MODE="results" \
	./scripts/run.sh hpc lunarc/pull

pull-lunarc-results: ## Recommended: pull results + optional manifests only
	@LUNARC_HOST="$(LUNARC_HOST)" \
	LUNARC_ROOT="$(LUNARC_ROOT)" \
	LUNARC_REPO_DIR="$(LUNARC_REPO_DIR)" \
	LUNARC_PULL_BASE="$(LUNARC_PULL_BASE)" \
	INCLUDE_CONTAINERS_MANIFEST="$(INCLUDE_CONTAINERS_MANIFEST)" \
	DATA_MANIFEST_GLOB="$(DATA_MANIFEST_GLOB)" \
	PULL_MODE="results" \
	./scripts/run.sh hpc lunarc/pull

.PHONY: push-lunarc pull-lunarc pull-lunarc-results
