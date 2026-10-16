ROOT_MK_DIR := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
include $(ROOT_MK_DIR)/_macro.mk

# Developer-focused makefile: reuses the shared artifacts/target workspace build cache.
# Usage examples:
#   make -f makes/cargo-dev.mk dev-fmt
#   make -f makes/cargo-dev.mk dev-lint
#   make -f makes/cargo-dev.mk dev-lint-automation
#   make -f makes/cargo-dev.mk dev-lint-clippy
#   make -f makes/cargo-dev.mk dev-lint-clippy-executors
#   make -f makes/cargo-dev.mk dev-test
#   make -f makes/cargo-dev.mk dev-test-full
#   make -f makes/cargo-dev.mk dev-vcf-certification
#   make -f makes/cargo-dev.mk dev-coverage
#   make -f makes/cargo-dev.mk dev-all
#   DEV_ARTIFACT_ROOT=artifacts make -f makes/cargo-dev.mk dev-test

DEV_ARTIFACT_ROOT ?= $(ARTIFACT_ROOT)
ROOT_MAKE ?= Makefile

# A parent Make exports its derived paths before this recursive Make selects a
# different root. Retain valid caller paths and let the child derive the rest.
DEV_ARTIFACT_PATH_VARIABLES := ARTIFACT_TARGET_DIR ARTIFACT_CARGO_HOME ARTIFACT_TMP_DIR \
 CARGO_TARGET_DIR CARGO_HOME TMPDIR TMP TEMP TEST_TMP_DIR ISO_ROOT ARTIFACT_DIR \
 PYTHONPYCACHEPREFIX RUFF_CACHE_DIR PYTEST_DEBUG_TEMPROOT ARTIFACTS_DIR \
 RS_ARTIFACT_ROOT RS_TARGET_DIR RS_CARGO_HOME RS_TMP_DIR RS_NEXTEST_CACHE_DIR \
 RS_NEXTEST_CONFIG_HOME NEXTEST_CACHE_DIR DEV_DNA_BIN
DEV_EXPLICIT_ARTIFACT_VARIABLES = $(foreach name,$(DEV_ARTIFACT_PATH_VARIABLES),$(if $(filter command line override,$(origin $(name))),$(name)))

define dev_artifact_environment
artifact_root="$(abspath $(DEV_ARTIFACT_ROOT))"; \
reset_variables=$$(python3 -c '\
from pathlib import Path; \
import os, sys; \
root = Path(sys.argv[1]).resolve(); \
boundary = Path("artifacts").resolve(); \
(root == boundary or boundary in root.parents) or sys.exit("developer artifacts must stay under workspace artifacts/"); \
explicit = set(sys.argv[2].split()); \
bad = [name for name in sys.argv[3:] if os.environ.get(name) and not (Path(os.environ[name]).resolve() == root or root in Path(os.environ[name]).resolve().parents)]; \
not explicit.intersection(bad) or sys.exit("developer artifact override escapes selected root: " + ", ".join(sorted(explicit.intersection(bad)))); \
print(" ".join(bad))' "$$artifact_root" "$(DEV_EXPLICIT_ARTIFACT_VARIABLES)" $(DEV_ARTIFACT_PATH_VARIABLES)) || exit $$?; \
for name in $$reset_variables; do unset "$$name"; done;
endef

define dev_make
	@$(dev_artifact_environment) \
$(MAKE) -f "$(ROOT_MAKE)" ARTIFACT_ROOT="$$artifact_root" ISO_ROOT="$$artifact_root" $(1)
endef

ifeq ($(notdir $(firstword $(MAKEFILE_LIST))),cargo-dev.mk)
.DEFAULT_GOAL := help

help:
	@printf '%s\n' \
	  'cargo-dev.mk targets:' \
	  '  dev-fmt dev-lint dev-lint-rustfmt dev-lint-clippy dev-lint-automation dev-lint-docs dev-lint-configs dev-lint-fast dev-lint-clippy-executors dev-realness-gate dev-audit dev-test dev-test-full dev-vcf-certification dev-coverage dev-all dev-clean' \
	  '' \
	  'Behavior:' \
	  '  - Reuses the shared artifacts contract for fast local iteration.' \
	  '  - Shared CARGO_TARGET_DIR stays under $(DEV_ARTIFACT_ROOT)/target.' \
	  '' \
	  'Knobs:' \
	  '  DEV_ARTIFACT_ROOT=<path>   change shared artifact root (default: artifacts)'
endif

dev-fmt:
	$(call dev_make,_fmt)

dev-lint:
	$(call dev_make,_lint)

dev-lint-rustfmt:
	$(call dev_make,_lint-rustfmt)

dev-lint-automation:
	$(call dev_make,_lint-automation)

dev-lint-scripts:
	$(call dev_make,dev-lint-automation)

dev-lint-clippy:
	$(call dev_make,_clippy)

dev-lint-docs:
	$(call dev_make,_lint-docs)

dev-lint-configs:
	$(call dev_make,_lint-configs)

dev-lint-fast:
	$(call dev_make,lint-fast)

dev-lint-clippy-executors:
	$(call dev_make,_clippy-executors)

dev-realness-gate:
	$(call dev_make,realness-gate)

dev-audit:
	$(call dev_make,_audit)

dev-test:
	$(call dev_make,_test-fast)

dev-test-full:
	$(call dev_make,_test)

dev-vcf-certification:
	$(call dev_make,vcf-certification)

dev-coverage:
	$(call dev_make,_coverage)

dev-all: dev-fmt dev-lint dev-audit dev-test dev-coverage

dev-clean:
	@$(dev_artifact_environment) \
rm -rf "$$artifact_root/tmp" || exit $$?; \
mkdir -p "$$artifact_root/tmp" || exit $$?; \
printf 'reset scratch directories under %s\n' "$$artifact_root"

ifeq ($(notdir $(firstword $(MAKEFILE_LIST))),cargo-dev.mk)
.PHONY: help dev-fmt dev-lint dev-lint-rustfmt dev-lint-automation dev-lint-scripts dev-lint-clippy dev-lint-docs dev-lint-configs dev-lint-fast dev-lint-clippy-executors dev-realness-gate dev-audit dev-test dev-test-full dev-vcf-certification dev-coverage dev-all dev-clean
else
.PHONY: dev-fmt dev-lint dev-lint-rustfmt dev-lint-automation dev-lint-scripts dev-lint-clippy dev-lint-docs dev-lint-configs dev-lint-fast dev-lint-clippy-executors dev-realness-gate dev-audit dev-test dev-test-full dev-vcf-certification dev-coverage dev-all dev-clean
endif
