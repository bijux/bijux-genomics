##@ Toy Runs

TOY_OUT ?= $(if $(ISOLATE_ROOT),$(ISOLATE_ROOT)/toy_runs,artifacts/toy_runs)


toy-golden-check: ## Compare produced toy outputs to goldens (timestamp-tolerant hashes).
	@./bin/isolate ./scripts/test/toy_runs.py check --profile all --out "$(TOY_OUT)"

refresh-toy: ## Regenerate deterministic toy datasets in assets/toy.
	@./scripts/assets/refresh-toy.sh

refresh-golden: ## Regenerate deterministic toy-run goldens in assets/golden.
	@./scripts/assets/refresh-golden.sh

golden-refresh: refresh-golden ## Backward-compatible alias.

.PHONY: toy-golden-check refresh-toy refresh-golden golden-refresh
