include makes/root.mk

.PHONY: ssot-policy-fast ci-fast bijux-std-checks

ssot-policy-fast:
	@mkdir -p artifacts/tmp
	$(MAKE) _ssot-policy-fast

ci-fast:
	@mkdir -p artifacts/tmp
	$(MAKE) _ci-fast

bijux-std-checks:
	@mkdir -p artifacts/tmp
	@BIJUX_STD_REF="main" BIJUX_STD_REMOTE="https://raw.githubusercontent.com/bijux/bijux-std" bash .bijux/shared/bijux-checks/check-bijux-std.sh
