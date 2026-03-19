##@ Toy Runs

ARTIFACT_ROOT ?= artifacts
TOY_OUT ?= $(ARTIFACT_ROOT)/toy_runs


toy-golden-check: ## Compare produced toy outputs to goldens (timestamp-tolerant hashes).
	@ARTIFACT_ROOT="$(ARTIFACT_ROOT)" cargo run -q -p bijux-dev-dna -- test run toy_runs check --profile all --out "$(TOY_OUT)"

refresh-toy: ## Regenerate deterministic toy datasets in assets/toy.
	@cargo run -q -p bijux-dev-dna -- assets run refresh-toy

refresh-golden: ## Regenerate deterministic toy-run goldens in assets/golden.
	@cargo run -q -p bijux-dev-dna -- assets run refresh-golden

refresh-assets-toy: ## Regenerate deterministic toy datasets in assets/toy.
	@cargo run -q -p bijux-dev-dna -- assets run refresh-toy

refresh-assets-golden: ## Regenerate deterministic toy-run goldens in assets/golden.
	@cargo run -q -p bijux-dev-dna -- assets run refresh-golden

golden-refresh: refresh-golden ## Backward-compatible alias.

.PHONY: toy-golden-check refresh-toy refresh-golden refresh-assets-toy refresh-assets-golden golden-refresh
