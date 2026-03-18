SHELL := /bin/sh
ROOT_MK_DIR := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
include $(ROOT_MK_DIR)/_macro.mk

# Developer-focused makefile: reuses the shared artifacts/target workspace build cache.
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
#   DEV_ARTIFACT_ROOT=artifacts make -f makes/cargo-dev.mk dev-test

DEV_ARTIFACT_ROOT ?= artifacts
ROOT_MAKE ?= Makefile

ifeq ($(notdir $(firstword $(MAKEFILE_LIST))),cargo-dev.mk)
.DEFAULT_GOAL := help

help:
	@printf '%s\n' \
	  'cargo-dev.mk targets:' \
	  '  dev-fmt dev-lint dev-lint-rustfmt dev-lint-clippy dev-lint-scripts dev-lint-docs dev-lint-configs dev-lint-fast dev-lint-clippy-executors dev-realness-gate dev-audit dev-test dev-test-full dev-vcf-certification dev-coverage dev-all dev-clean' \
	  '' \
	  'Behavior:' \
	  '  - Reuses the shared artifacts contract for fast local iteration.' \
	  '  - Shared CARGO_TARGET_DIR stays under $(DEV_ARTIFACT_ROOT)/target.' \
	  '' \
	  'Knobs:' \
	  '  DEV_ARTIFACT_ROOT=<path>   change shared artifact root (default: artifacts)'
endif

dev-fmt:
	@ARTIFACT_ROOT="$(DEV_ARTIFACT_ROOT)" $(MAKE) -f $(ROOT_MAKE) _fmt

dev-lint:
	@ARTIFACT_ROOT="$(DEV_ARTIFACT_ROOT)" $(MAKE) -f $(ROOT_MAKE) _lint

dev-lint-rustfmt:
	@ARTIFACT_ROOT="$(DEV_ARTIFACT_ROOT)" $(MAKE) -f $(ROOT_MAKE) _lint-rustfmt

dev-lint-scripts:
	@ARTIFACT_ROOT="$(DEV_ARTIFACT_ROOT)" $(MAKE) -f $(ROOT_MAKE) _lint-scripts

dev-lint-clippy:
	@ARTIFACT_ROOT="$(DEV_ARTIFACT_ROOT)" $(MAKE) -f $(ROOT_MAKE) _clippy

dev-lint-docs:
	@ARTIFACT_ROOT="$(DEV_ARTIFACT_ROOT)" $(MAKE) -f $(ROOT_MAKE) _lint-docs

dev-lint-configs:
	@ARTIFACT_ROOT="$(DEV_ARTIFACT_ROOT)" $(MAKE) -f $(ROOT_MAKE) _lint-configs

dev-lint-fast:
	@ARTIFACT_ROOT="$(DEV_ARTIFACT_ROOT)" $(MAKE) -f $(ROOT_MAKE) lint-fast

dev-lint-clippy-executors:
	@ARTIFACT_ROOT="$(DEV_ARTIFACT_ROOT)" $(MAKE) -f $(ROOT_MAKE) _clippy-executors

dev-realness-gate:
	@ARTIFACT_ROOT="$(DEV_ARTIFACT_ROOT)" $(MAKE) -f $(ROOT_MAKE) realness-gate

dev-audit:
	@ARTIFACT_ROOT="$(DEV_ARTIFACT_ROOT)" $(MAKE) -f $(ROOT_MAKE) _audit

dev-test:
	@ARTIFACT_ROOT="$(DEV_ARTIFACT_ROOT)" $(MAKE) -f $(ROOT_MAKE) _test-fast

dev-test-full:
	@ARTIFACT_ROOT="$(DEV_ARTIFACT_ROOT)" $(MAKE) -f $(ROOT_MAKE) _test

dev-vcf-certification:
	@ARTIFACT_ROOT="$(DEV_ARTIFACT_ROOT)" $(MAKE) -f $(ROOT_MAKE) vcf-certification

dev-coverage:
	@ARTIFACT_ROOT="$(DEV_ARTIFACT_ROOT)" $(MAKE) -f $(ROOT_MAKE) _coverage

dev-all: dev-fmt dev-lint dev-audit dev-test dev-coverage

dev-clean:
	@$(call safe_rm,$(DEV_ARTIFACT_ROOT)/tmp)
	@mkdir -p "$(DEV_ARTIFACT_ROOT)/tmp"
	@echo "reset scratch directories under $(DEV_ARTIFACT_ROOT)"

ifeq ($(notdir $(firstword $(MAKEFILE_LIST))),cargo-dev.mk)
.PHONY: help dev-fmt dev-lint dev-lint-rustfmt dev-lint-scripts dev-lint-clippy dev-lint-docs dev-lint-configs dev-lint-fast dev-lint-clippy-executors dev-realness-gate dev-audit dev-test dev-test-full dev-vcf-certification dev-coverage dev-all dev-clean
else
.PHONY: dev-fmt dev-lint dev-lint-rustfmt dev-lint-scripts dev-lint-clippy dev-lint-docs dev-lint-configs dev-lint-fast dev-lint-clippy-executors dev-realness-gate dev-audit dev-test dev-test-full dev-vcf-certification dev-coverage dev-all dev-clean
endif
