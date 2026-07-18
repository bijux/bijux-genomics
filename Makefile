include makes/root.mk

.PHONY: ssot-policy-fast ci-fast

ssot-policy-fast:
	@mkdir -p artifacts/tmp
	$(MAKE) _ssot-policy-fast

ci-fast:
	@mkdir -p artifacts/tmp
	$(MAKE) _ci-fast
