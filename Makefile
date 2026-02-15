SHELL := /bin/sh

# Automatic parallel job detection
JOBS ?= $(shell nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 8)
NEXTEST_JOBS ?= $(JOBS)

include makefiles/cargo.mk
include makefiles/cargo-dev.mk
include makefiles/containers.mk
include makefiles/benchmarks-fastq.mk
include makefiles/benchmarks-bam.mk
include makefiles/lab.mk
include makefiles/lunarc.mk
include makefiles/docs.mk
include makefiles/policies.mk

.DEFAULT_GOAL := help

##@ General

help: ## Show this help message
	@if [ "$${SHOW_INTERNAL:-0}" = "1" ]; then \
		./scripts/run.sh tooling make-help --internal; \
	else \
		./scripts/run.sh tooling make-help; \
	fi

_prep-apptainer-batch: ## Build all Apptainer defs in VM-local output dir
	@$(MAKE) _containers-apptainer-build

_gc-mac: ## Remove macOS metadata cruft locally (outside CI)
	@find . -name '.DS_Store' -type f -delete
	@find . -name '._*' -type f -delete
	@echo "macOS cruft removed"
