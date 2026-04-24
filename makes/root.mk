SHELL := /bin/sh

ROOT_MK_DIR := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))

# Automatic parallel job detection
JOBS ?= $(shell nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 8)
NEXTEST_JOBS ?= $(JOBS)

include $(ROOT_MK_DIR)/_macro.mk
include $(ROOT_MK_DIR)/cargo.mk
include $(ROOT_MK_DIR)/cargo-dev.mk
include $(ROOT_MK_DIR)/containers.mk
include $(ROOT_MK_DIR)/benchmarks-fastq.mk
include $(ROOT_MK_DIR)/benchmarks-bam.mk
include $(ROOT_MK_DIR)/lab.mk
include $(ROOT_MK_DIR)/lunarc.mk
include $(ROOT_MK_DIR)/docs.mk
include $(ROOT_MK_DIR)/policies.mk

.DEFAULT_GOAL := help

##@ General

help: ## Show this help message
	@mkdir -p artifacts/tmp artifacts/target artifacts/cargo/home
	@if [ "$${SHOW_INTERNAL:-0}" = "1" ]; then \
		cargo run -q -p bijux-dna-dev -- tooling run make-help --internal; \
	else \
		cargo run -q -p bijux-dna-dev -- tooling run make-help; \
	fi

_prep-apptainer-batch: ## Build all Apptainer defs in VM-local output dir
	@$(MAKE) _containers-apptainer-build

_gc-mac: ## Remove macOS metadata cruft locally (outside CI)
	@find . -name '.DS_Store' -type f -delete
	@find . -name '._*' -type f -delete
	@echo "macOS cruft removed"
