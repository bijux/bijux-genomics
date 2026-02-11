##@ Toy Runs

TOY_OUT ?= $(if $(ISOLATE_ROOT),$(ISOLATE_ROOT)/toy_runs,artifacts/toy_runs)


toy-run-fastq: ## Run deterministic FASTQ toy reference profile artifacts.
	@./bin/isolate ./scripts/toy_runs.py run --profile fastq --out "$(TOY_OUT)"

toy-run-bam: ## Run deterministic BAM toy reference profile artifacts.
	@./bin/isolate ./scripts/toy_runs.py run --profile bam --out "$(TOY_OUT)"

toy-run-vcf: ## Run deterministic VCF toy reference profile artifacts.
	@./bin/isolate ./scripts/toy_runs.py run --profile vcf --out "$(TOY_OUT)"

toy-golden-check: ## Compare produced toy outputs to goldens (timestamp-tolerant hashes).
	@./bin/isolate ./scripts/toy_runs.py check --profile all --out "$(TOY_OUT)"

golden-refresh: ## Refresh toy goldens (requires ACCEPT=1).
	@if [ "$(ACCEPT)" != "1" ]; then \
		echo "ERROR: golden refresh requires ACCEPT=1"; \
		exit 2; \
	fi
	@./bin/isolate ./scripts/toy_runs.py refresh --accept --profile all --out "$(TOY_OUT)"

demo: ## Run all toy profiles and produce one combined demo report.
	@./bin/isolate ./scripts/toy_runs.py demo --profile all --out "$(TOY_OUT)"

.PHONY: toy-run-fastq toy-run-bam toy-run-vcf toy-golden-check golden-refresh demo
