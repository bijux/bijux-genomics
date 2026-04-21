include makes/root.mk

.PHONY: ssot-policy-fast ci-fast bijux-std-checks

ssot-policy-fast: _ssot-policy-fast

ci-fast: _ci-fast

bijux-std-checks:
	@BIJUX_STD_REF="main" BIJUX_STD_REMOTE="https://raw.githubusercontent.com/bijux/bijux-std" bash .bijux/shared/bijux-checks/check-bijux-std.sh
