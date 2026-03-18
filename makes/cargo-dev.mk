SHELL := /bin/sh

# Developer-focused makefile: reuses a single isolate/target dir across commands.
# Usage examples:
#   make -f makes/cargo-dev.mk dev-fmt
#   make -f makes/cargo-dev.mk dev-lint
#   make -f makes/cargo-dev.mk dev-lint-scripts
#   make -f makes/cargo-dev.mk dev-lint-clippy
#   make -f makes/cargo-dev.mk dev-lint-clippy-executors
#   make -f makes/cargo-dev.mk dev-test
#   make -f makes/cargo-dev.mk dev-test-full
#   make -f makes/cargo-dev.mk dev-vcf-certification
#   make -f makes/cargo-dev.mk dev-coverage
#   make -f makes/cargo-dev.mk dev-all
#   DEV_ISO_TAG=my-fast-loop make -f makes/cargo-dev.mk dev-test

DEV_ISO_TAG ?= dev-ci-local
ROOT_MAKE ?= Makefile

ISO_DEV = ./bin/isolate --tag "$(DEV_ISO_TAG)" --reuse

ifeq ($(firstword $(MAKEFILE_LIST)),$(lastword $(MAKEFILE_LIST)))
.DEFAULT_GOAL := help

help:
	@printf '%s\n' \
	  'cargo-dev.mk targets:' \
	  '  dev-fmt dev-lint dev-lint-rustfmt dev-lint-clippy dev-lint-scripts dev-lint-docs dev-lint-configs dev-lint-fast dev-lint-clippy-executors dev-realness-gate dev-audit dev-test dev-test-full dev-vcf-certification dev-coverage dev-all dev-clean' \
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

dev-lint-rustfmt:
	@$(ISO_DEV) $(MAKE) -f $(ROOT_MAKE) _lint-rustfmt

dev-lint-scripts:
	@$(ISO_DEV) $(MAKE) -f $(ROOT_MAKE) _lint-scripts

dev-lint-clippy:
	@$(ISO_DEV) $(MAKE) -f $(ROOT_MAKE) _clippy

dev-lint-docs:
	@$(ISO_DEV) $(MAKE) -f $(ROOT_MAKE) _lint-docs

dev-lint-configs:
	@$(ISO_DEV) $(MAKE) -f $(ROOT_MAKE) _lint-configs

dev-lint-fast:
	@$(ISO_DEV) $(MAKE) -f $(ROOT_MAKE) lint-fast

dev-lint-clippy-executors:
	@$(ISO_DEV) $(MAKE) -f $(ROOT_MAKE) _clippy-executors

dev-realness-gate:
	@$(ISO_DEV) $(MAKE) -f $(ROOT_MAKE) realness-gate

dev-audit:
	@$(ISO_DEV) $(MAKE) -f $(ROOT_MAKE) _audit

dev-test:
	@$(ISO_DEV) $(MAKE) -f $(ROOT_MAKE) _test-fast

dev-test-full:
	@$(ISO_DEV) $(MAKE) -f $(ROOT_MAKE) _test

dev-vcf-certification:
	@$(ISO_DEV) $(MAKE) -f $(ROOT_MAKE) vcf-certification

dev-coverage:
	@$(ISO_DEV) $(MAKE) -f $(ROOT_MAKE) _coverage

dev-all: dev-fmt dev-lint dev-audit dev-test dev-coverage

dev-clean:
	@rm -rf "artifacts/isolates/$(DEV_ISO_TAG)"
	@echo "removed artifacts/isolates/$(DEV_ISO_TAG)"

ifeq ($(firstword $(MAKEFILE_LIST)),$(lastword $(MAKEFILE_LIST)))
.PHONY: help dev-fmt dev-lint dev-lint-rustfmt dev-lint-scripts dev-lint-clippy dev-lint-docs dev-lint-configs dev-lint-fast dev-lint-clippy-executors dev-realness-gate dev-audit dev-test dev-test-full dev-vcf-certification dev-coverage dev-all dev-clean
else
.PHONY: dev-fmt dev-lint dev-lint-rustfmt dev-lint-scripts dev-lint-clippy dev-lint-docs dev-lint-configs dev-lint-fast dev-lint-clippy-executors dev-realness-gate dev-audit dev-test dev-test-full dev-vcf-certification dev-coverage dev-all dev-clean
endif
