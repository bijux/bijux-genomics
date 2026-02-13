##@ BAM Benchmarks

BIJUX_BIN ?= ./bin/isolate cargo run --bin bijux -- dna
BAM ?=
OUT_DIR ?= .
TOOLS ?=
BAM_PROFILE ?= bam-to-bam__default__v1
BAM_STAGE ?= validate
BAM_SAMPLE_ID ?= sample

BENCH_TOOLS_ARGS = $(if $(TOOLS),--tools $(TOOLS),)

benchmark-bam-stage: ## Benchmark one BAM stage (set BAM=<path> BAM_STAGE=<stage>)
	@BIJUX_BIN="$(BIJUX_BIN)" BAM="$(BAM)" BAM_STAGE="$(BAM_STAGE)" BAM_SAMPLE_ID="$(BAM_SAMPLE_ID)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ./scripts/run.sh tooling benchmarks bam-stage

benchmark-bam-pipeline: ## Benchmark BAM pipeline (set BAM=<path>, optional BAM_PROFILE)
	@BIJUX_BIN="$(BIJUX_BIN)" BAM="$(BAM)" BAM_PROFILE="$(BAM_PROFILE)" BAM_SAMPLE_ID="$(BAM_SAMPLE_ID)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ./scripts/run.sh tooling benchmarks bam-pipeline

benchmark-bam-all: ## Run BAM stage + pipeline benchmarks
	@BIJUX_BIN="$(BIJUX_BIN)" BAM="$(BAM)" BAM_STAGE="$(BAM_STAGE)" BAM_PROFILE="$(BAM_PROFILE)" BAM_SAMPLE_ID="$(BAM_SAMPLE_ID)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ./scripts/run.sh tooling benchmarks bam-all

.PHONY: benchmark-bam-stage benchmark-bam-pipeline benchmark-bam-all
