BIJUX_STD_CHECK_SCRIPT ?= .bijux/shared/bijux-checks/check-bijux-std.sh
BIJUX_STD_UPDATE_SCRIPT ?= .bijux/shared/bijux-checks/update-bijux-std.sh
BIJUX_STD_PIN ?= .github/standards/bijux-std.sha
BIJUX_STD_REF ?= $(strip $(shell cat "$(BIJUX_STD_PIN)" 2>/dev/null))
export BIJUX_STD_REF
BIJUX_STD_GIT_URL ?= https://github.com/bijux/bijux-std.git
BIJUX_STD_CAPABILITIES ?= docs python rust
BIJUX_STD_UPDATE_CHANNEL ?= branch
BIJUX_STD_TAG_PATTERN ?= v*
BIJUX_STD_PYCACHE_DIR ?= artifacts/bijux-std/pycache

.PHONY: bijux-std-source-require bijux-std-checks bijux-std-update bijux-std

bijux-std-source-require: ## Require an immutable full standard source reference
	@ref="$${BIJUX_STD_REF}"; \
		case "$$ref" in ""|*[!0-9a-f]*) echo "ERROR: Bijux standard source requires a full lowercase commit SHA" >&2; exit 1 ;; esac; \
		test "$${#ref}" -eq 40 || { echo "ERROR: Bijux standard source requires a full lowercase commit SHA" >&2; exit 1; }

bijux-std-checks: bijux-std-source-require ## Verify shared directories match the pinned bijux-std source
	@mkdir -p "$(BIJUX_STD_PYCACHE_DIR)"
	@PYTHONPYCACHEPREFIX="$(abspath $(BIJUX_STD_PYCACHE_DIR))" \
		BIJUX_STD_CAPABILITIES="$(BIJUX_STD_CAPABILITIES)" \
		BIJUX_STD_REF="$(BIJUX_STD_REF)" \
		BIJUX_STD_GIT_URL="$(BIJUX_STD_GIT_URL)" \
		BIJUX_STD_STRICT_REMOTE=1 \
		BIJUX_STD_REQUIRE_REMOTE_MATCH=1 \
		bash "$(BIJUX_STD_CHECK_SCRIPT)"

bijux-std-update: bijux-std-source-require ## Refresh shared directories from the pinned bijux-std source
	@mkdir -p "$(BIJUX_STD_PYCACHE_DIR)"
	@PYTHONPYCACHEPREFIX="$(abspath $(BIJUX_STD_PYCACHE_DIR))" \
		BIJUX_STD_CAPABILITIES="$(BIJUX_STD_CAPABILITIES)" \
		BIJUX_STD_REF="$(BIJUX_STD_REF)" \
		BIJUX_STD_GIT_URL="$(BIJUX_STD_GIT_URL)" \
		BIJUX_STD_UPDATE_CHANNEL="$(BIJUX_STD_UPDATE_CHANNEL)" \
		BIJUX_STD_TAG_PATTERN="$(BIJUX_STD_TAG_PATTERN)" \
		bash "$(BIJUX_STD_UPDATE_SCRIPT)"

bijux-std: bijux-std-checks ## Backward-compatible alias
