SHELL := /bin/sh

# Automatic parallel job detection
JOBS ?= $(shell nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 8)
NEXTEST_JOBS ?= $(JOBS)

include makefiles/cargo.mk
include makefiles/containers.mk
include makefiles/benchmarks.mk
include makefiles/lab.mk
include makefiles/policies.mk
include makefiles/docs.mk

.DEFAULT_GOAL := help

##@ General

help: ## Show this help message
	@printf "\033[1mBijux DNA Project – Available Make Targets\033[0m\n\n"
	@printf "\033[1mUsage:\033[0m make \033[36m<TARGET>\033[0m\n\n"
	@awk 'BEGIN {FS = ":.*?## "} \
	      /^##@/{printf "\n\033[1;34m%s\033[0m\n", substr($$0, 5)} \
	      /^[a-zA-Z0-9_-]+:.*?## /{printf "  \033[36m%-30s\033[0m %s\n", $$1, $$2}' \
	      $(MAKEFILE_LIST)

prep-apptainer-batch: ## Build all Apptainer defs in VM-local output dir
	@$(MAKE) containers-apptainer-build
