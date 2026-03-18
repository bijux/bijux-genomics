##@ Toy Runs

TOY_OUT ?= $(if $(ISOLATE_ROOT),$(ISOLATE_ROOT)/toy_runs,artifacts/toy_runs)


toy-golden-check: ## Compare produced toy outputs to goldens (timestamp-tolerant hashes).
	@./bin/isolate ./scripts/run.sh test toy_runs check --profile all --out "$(TOY_OUT)"

refresh-toy: ## Regenerate deterministic toy datasets in assets/toy.
	@./scripts/run.sh assets refresh-toy

refresh-golden: ## Regenerate deterministic toy-run goldens in assets/golden.
	@./scripts/run.sh assets refresh-golden

refresh-assets-toy: ## Regenerate deterministic toy datasets in assets/toy.
	@./scripts/run.sh assets refresh-toy

refresh-assets-golden: ## Regenerate deterministic toy-run goldens in assets/golden.
	@./scripts/run.sh assets refresh-golden

golden-refresh: refresh-golden ## Backward-compatible alias.

.PHONY: toy-golden-check refresh-toy refresh-golden refresh-assets-toy refresh-assets-golden golden-refresh
