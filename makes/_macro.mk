# Shared make macros and artifact environment defaults.

ARTIFACT_ROOT ?= artifacts
ARTIFACT_TARGET_DIR ?= $(abspath $(ARTIFACT_ROOT)/target)
ARTIFACT_CARGO_HOME ?= $(abspath $(ARTIFACT_ROOT)/cargo/home)
ARTIFACT_TMP_DIR ?= $(abspath $(ARTIFACT_ROOT)/tmp)

ISO_ROOT ?= $(abspath $(ARTIFACT_ROOT))
CARGO_TARGET_DIR ?= $(ARTIFACT_TARGET_DIR)
CARGO_HOME ?= $(ARTIFACT_CARGO_HOME)
# An inherited host temporary directory is not a repository output selection.
# Explicit Make values and the selected artifact scratch authority remain intact.
_artifact_env_quote = '$(subst ','"'"',$(1))'
define _artifact_inherited_temp
$(shell \
 declared=$(call _artifact_env_quote,$(1)); candidate="$$declared"; \
 root=$(call _artifact_env_quote,$(ARTIFACT_ROOT)); scratch=$(call _artifact_env_quote,$(ARTIFACT_TMP_DIR)); \
 cwd=$(call _artifact_env_quote,$(CURDIR)); \
 case "$$candidate" in (/*) ;; (*) candidate="$$cwd/$$candidate" ;; esac; \
 case "$$root" in (/*) ;; (*) root="$$cwd/$$root" ;; esac; \
 case "$$scratch" in (/*) ;; (*) scratch="$$cwd/$$scratch" ;; esac; \
 root=$${root%/}; scratch=$${scratch%/}; \
 case "$$candidate" in \
 (*/../*|*/..) printf '%s' $(call _artifact_env_quote,$(2)) ;; \
 ("$$root"|"$$root"/*|"$$scratch"|"$$scratch"/*) printf '%s' "$$declared" ;; \
 (*) printf '%s' $(call _artifact_env_quote,$(2)) ;; \
 esac)
endef
# Keep ordinary normalization file-owned; -e requires an explicit override.
ifeq ($(origin TMPDIR),environment)
TMPDIR := $(call _artifact_inherited_temp,$(TMPDIR),$(ARTIFACT_TMP_DIR))
endif
ifeq ($(origin TMPDIR),environment override)
override TMPDIR := $(call _artifact_inherited_temp,$(TMPDIR),$(ARTIFACT_TMP_DIR))
endif
TMPDIR ?= $(ARTIFACT_TMP_DIR)
ifeq ($(origin TMP),environment)
TMP := $(call _artifact_inherited_temp,$(TMP),$(TMPDIR))
endif
ifeq ($(origin TMP),environment override)
override TMP := $(call _artifact_inherited_temp,$(TMP),$(TMPDIR))
endif
TMP ?= $(TMPDIR)
ifeq ($(origin TEMP),environment)
TEMP := $(call _artifact_inherited_temp,$(TEMP),$(TMPDIR))
endif
ifeq ($(origin TEMP),environment override)
override TEMP := $(call _artifact_inherited_temp,$(TEMP),$(TMPDIR))
endif
TEMP ?= $(TMPDIR)
PYTHONPYCACHEPREFIX ?= $(abspath $(ARTIFACT_ROOT)/python/bytecode)
RUFF_CACHE_DIR ?= $(abspath $(ARTIFACT_ROOT)/python/ruff)
PYTEST_DEBUG_TEMPROOT ?= $(ARTIFACT_TMP_DIR)

export ISO_ROOT
export CARGO_TARGET_DIR
export CARGO_HOME
export TMPDIR
export TMP
export TEMP
export PYTHONPYCACHEPREFIX
export RUFF_CACHE_DIR
export PYTEST_DEBUG_TEMPROOT

require_tool = command -v $(1) >/dev/null 2>&1 || { echo "$(1) is required" >&2; exit 1; }
require_file = test -f "$(1)" || { echo "required file missing: $(1)" >&2; exit 1; }
require_var = test -n "$${$(1):-}" || { echo "required variable missing: $(1)" >&2; exit 1; }
print_section = printf '\n== %s ==\n' "$(1)"
safe_rm = case "$(abspath $(1))" in "$(abspath $(ARTIFACT_ROOT))"/*) rm -rf "$(1)" ;; *) echo "refusing to delete outside $(ARTIFACT_ROOT): $(1)" >&2; exit 1 ;; esac
ensure_artifact_env = mkdir -p "$(ARTIFACT_ROOT)" "$(ISO_ROOT)" "$(CARGO_TARGET_DIR)" "$(CARGO_HOME)" "$(TMPDIR)" "$(TMP)" "$(TEMP)"
