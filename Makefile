SHELL := /bin/sh

# Automatic parallel job detection
JOBS ?= $(shell nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 8)
NEXTEST_JOBS ?= $(JOBS)

include makefiles/cargo.mk
include makefiles/containers.mk
include makefiles/benchmarks-fastq.mk
include makefiles/benchmarks-bam.mk
include makefiles/lab.mk
include makefiles/lunarc.mk
include makefiles/docs.mk

.DEFAULT_GOAL := help

##@ General

help: ## Show this help message
	@printf "Public make targets:\n\n"
	@printf "  %-22s %s\n" "fmt" "Format Rust workspace"
	@printf "  %-22s %s\n" "lint" "Run repository policy/lint checks"
	@printf "  %-22s %s\n" "audit" "Run dependency/security audit"
	@printf "  %-22s %s\n" "test" "Run test suite"
	@printf "  %-22s %s\n" "coverage" "Generate and validate coverage report"
	@printf "  %-22s %s\n" "ci" "Run fmt/lint/audit/test/coverage in one isolate"
	@printf "  %-22s %s\n" "refresh-assets-toy" "Refresh toy assets"
	@printf "  %-22s %s\n" "refresh-assets-golden" "Refresh golden assets"
	@if [ "$${SHOW_INTERNAL:-0}" = "1" ]; then \
		printf "\nInternal make targets:\n\n"; \
		printf "  %-22s %s\n" "domain-validate" "Run strict domain validation checks"; \
		printf "  %-22s %s\n" "examples-validate" "Run example structure/corpus/golden checks"; \
		printf "  %-22s %s\n" "_policy-fast" "Fast policy checks for local iteration"; \
		printf "  %-22s %s\n" "_ci-fast" "Fast internal CI profile"; \
		printf "  %-22s %s\n" "_ci-slow" "Slow internal CI profile"; \
	fi
	@printf "\nSee makefiles/README.md for the public surface contract.\n"

_prep-apptainer-batch: ## Build all Apptainer defs in VM-local output dir
	@$(MAKE) _containers-apptainer-build

_gc-mac: ## Remove macOS metadata cruft locally (outside CI)
	@find . -name '.DS_Store' -type f -delete
	@find . -name '._*' -type f -delete
	@echo "macOS cruft removed"
