##@ Toy Runs

TOY_OUT ?= $(if $(ISOLATE_ROOT),$(ISOLATE_ROOT)/toy_runs,artifacts/toy_runs)


toy-golden-check: ## Compare produced toy outputs to goldens (timestamp-tolerant hashes).
	@./bin/isolate ./scripts/toy_runs.py check --profile all --out "$(TOY_OUT)"

golden-refresh: ## Refresh toy goldens (requires ACCEPT=1).
	@if [ "$(ACCEPT)" != "1" ]; then \
		echo "ERROR: golden refresh requires ACCEPT=1"; \
		exit 2; \
	fi
	@./bin/isolate ./scripts/toy_runs.py refresh --accept --profile all --out "$(TOY_OUT)"

.PHONY: toy-golden-check golden-refresh
