##@ BAM Benchmarks

BIJUX_BIN ?= ./bin/isolate cargo run --bin bijux-dna --
BAM ?=
OUT_DIR ?= .
TOOLS ?=
BAM_PROFILE ?= bam-to-bam__default__v1
BAM_STAGE ?= validate
BAM_SAMPLE_ID ?= sample

BENCH_TOOLS_ARGS = $(if $(TOOLS),--tools $(TOOLS),)

benchmark-bam-stage: ## Benchmark one BAM stage (set BAM=<path> BAM_STAGE=<stage>)
	@if [ -z "$(BAM)" ]; then \
		echo "ERROR: set BAM=<path/to/input.bam>"; \
		exit 2; \
	fi
	@$(BIJUX_BIN) bench bam stage \
		--sample-id "$(BAM_SAMPLE_ID)" \
		--stage "$(BAM_STAGE)" \
		--bam "$(BAM)" \
		--out "$(OUT_DIR)" \
		$(BENCH_TOOLS_ARGS)

benchmark-bam-pipeline: ## Benchmark BAM pipeline (set BAM=<path>, optional BAM_PROFILE)
	@if [ -z "$(BAM)" ]; then \
		echo "ERROR: set BAM=<path/to/input.bam>"; \
		exit 2; \
	fi
	@$(BIJUX_BIN) bench bam pipeline \
		--sample-id "$(BAM_SAMPLE_ID)" \
		--profile "$(BAM_PROFILE)" \
		--bam "$(BAM)" \
		--out "$(OUT_DIR)" \
		$(BENCH_TOOLS_ARGS)

benchmark-bam-all: ## Run BAM stage + pipeline benchmarks
	@set -e; \
	$(MAKE) benchmark-bam-stage BAM="$(BAM)" BAM_STAGE="$(BAM_STAGE)" BAM_SAMPLE_ID="$(BAM_SAMPLE_ID)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)"; \
	$(MAKE) benchmark-bam-pipeline BAM="$(BAM)" BAM_PROFILE="$(BAM_PROFILE)" BAM_SAMPLE_ID="$(BAM_SAMPLE_ID)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)"

.PHONY: benchmark-bam-stage benchmark-bam-pipeline benchmark-bam-all
