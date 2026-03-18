# Shared make macros and artifact environment defaults.

ARTIFACT_ROOT ?= artifacts
ARTIFACT_TARGET_DIR ?= $(abspath $(ARTIFACT_ROOT)/target)
ARTIFACT_CARGO_HOME ?= $(abspath $(ARTIFACT_ROOT)/cargo/home)
ARTIFACT_TMP_DIR ?= $(abspath $(ARTIFACT_ROOT)/tmp)
ARTIFACT_RUN_ROOT ?= $(abspath $(ARTIFACT_ROOT)/run/$(or $(RUN_ID),local))

export ISO_ROOT ?= $(ARTIFACT_RUN_ROOT)
export CARGO_TARGET_DIR ?= $(ARTIFACT_TARGET_DIR)
export CARGO_HOME ?= $(ARTIFACT_CARGO_HOME)
export TMPDIR ?= $(ARTIFACT_TMP_DIR)
export TMP ?= $(ARTIFACT_TMP_DIR)
export TEMP ?= $(ARTIFACT_TMP_DIR)

require_tool = command -v $(1) >/dev/null 2>&1 || { echo "$(1) is required" >&2; exit 1; }
require_file = test -f "$(1)" || { echo "required file missing: $(1)" >&2; exit 1; }
require_var = test -n "$${$(1):-}" || { echo "required variable missing: $(1)" >&2; exit 1; }
print_section = printf '\n== %s ==\n' "$(1)"
safe_rm = case "$(abspath $(1))" in "$(abspath $(ARTIFACT_ROOT))"/*) rm -rf "$(1)" ;; *) echo "refusing to delete outside $(ARTIFACT_ROOT): $(1)" >&2; exit 1 ;; esac
ensure_artifact_env = mkdir -p "$(ARTIFACT_ROOT)" "$(ISO_ROOT)" "$(CARGO_TARGET_DIR)" "$(CARGO_HOME)" "$(TMPDIR)" "$(TMP)" "$(TEMP)"
