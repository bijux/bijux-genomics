##@ BAM Benchmarks

BIJUX_BENCH_BIN ?= cargo run -q -p bijux-dna --
BAM ?=
OUT_DIR ?= .
TOOLS ?=
BAM_PROFILE ?= bam-to-bam__default__v1
BAM_STAGE ?= validate
BAM_SAMPLE_ID ?= sample

BENCH_TOOLS_ARGS = $(if $(TOOLS),--tools $(TOOLS),)

_benchmark-bam-stage: ## Benchmark one BAM stage (set BAM=<path> BAM_STAGE=<stage>)
	@$(BIJUX_BENCH_BIN) bench bam stage \
		--sample-id "$(BAM_SAMPLE_ID)" \
		--stage "$(BAM_STAGE)" \
		--bam "$(BAM)" \
		--out "$(OUT_DIR)" \
		$(BENCH_TOOLS_ARGS)

_benchmark-bam-pipeline: ## Benchmark BAM pipeline (set BAM=<path>, optional BAM_PROFILE)
	@$(BIJUX_BENCH_BIN) bench bam pipeline \
		--sample-id "$(BAM_SAMPLE_ID)" \
		--profile "$(BAM_PROFILE)" \
		--bam "$(BAM)" \
		--out "$(OUT_DIR)" \
		$(BENCH_TOOLS_ARGS)

_benchmark-bam-all: ## Run BAM stage + pipeline benchmarks
	@$(MAKE) _benchmark-bam-stage BAM="$(BAM)" BAM_STAGE="$(BAM_STAGE)" BAM_SAMPLE_ID="$(BAM_SAMPLE_ID)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)"
	@$(MAKE) _benchmark-bam-pipeline BAM="$(BAM)" BAM_PROFILE="$(BAM_PROFILE)" BAM_SAMPLE_ID="$(BAM_SAMPLE_ID)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)"

.PHONY: _benchmark-bam-stage _benchmark-bam-pipeline _benchmark-bam-all
