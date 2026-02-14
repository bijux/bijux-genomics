SHELL := /bin/sh

# Developer-focused makefile: reuses a single isolate/target dir across commands.
# Usage examples:
#   make -f makefiles/cargo-dev.mk dev-fmt
#   make -f makefiles/cargo-dev.mk dev-lint
#   make -f makefiles/cargo-dev.mk dev-test
#   make -f makefiles/cargo-dev.mk dev-test-fast
#   make -f makefiles/cargo-dev.mk dev-coverage
#   make -f makefiles/cargo-dev.mk dev-all
#   DEV_ISO_TAG=my-fast-loop make -f makefiles/cargo-dev.mk dev-test

DEV_ISO_TAG ?= dev-ci-local
ROOT_MAKE ?= Makefile

ISO_DEV = ./bin/isolate --tag "$(DEV_ISO_TAG)" --reuse

ifeq ($(firstword $(MAKEFILE_LIST)),$(lastword $(MAKEFILE_LIST)))
.DEFAULT_GOAL := help

help:
	@printf '%s\n' \
	  'cargo-dev.mk targets:' \
	  '  dev-fmt dev-lint dev-audit dev-test dev-test-fast dev-coverage dev-all dev-clean' \
	  '' \
	  'Behavior:' \
	  '  - Runs through one reusable isolate tag for fast local iteration.' \
	  '  - Shared CARGO_TARGET_DIR stays under artifacts/isolates/<DEV_ISO_TAG>/target.' \
	  '' \
	  'Knobs:' \
	  '  DEV_ISO_TAG=<tag>   change shared isolate tag (default: dev-ci-local)'
endif

dev-fmt:
	@$(ISO_DEV) $(MAKE) -f $(ROOT_MAKE) _fmt

dev-lint:
	@$(ISO_DEV) $(MAKE) -f $(ROOT_MAKE) _lint

dev-audit:
	@$(ISO_DEV) $(MAKE) -f $(ROOT_MAKE) _audit

dev-test:
	@$(ISO_DEV) $(MAKE) -f $(ROOT_MAKE) _test

dev-test-fast:
	@$(ISO_DEV) $(MAKE) -f $(ROOT_MAKE) _test-fast

dev-coverage:
	@$(ISO_DEV) $(MAKE) -f $(ROOT_MAKE) _coverage

dev-all: dev-fmt dev-lint dev-audit dev-test dev-coverage

dev-clean:
	@rm -rf "artifacts/isolates/$(DEV_ISO_TAG)"
	@echo "removed artifacts/isolates/$(DEV_ISO_TAG)"

ifeq ($(firstword $(MAKEFILE_LIST)),$(lastword $(MAKEFILE_LIST)))
.PHONY: help dev-fmt dev-lint dev-audit dev-test dev-test-fast dev-coverage dev-all dev-clean
else
.PHONY: dev-fmt dev-lint dev-audit dev-test dev-test-fast dev-coverage dev-all dev-clean
endif
