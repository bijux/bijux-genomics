##@ Performance Benchmarks

BIJUX_BIN ?= ./bin/isolate cargo run --bin bijux -- dna
OUT_DIR ?= .
TOOLS ?=
SAMPLE_ID ?=
R1 ?=
R2 ?=
ALLOW_EXPERIMENTAL ?= 0

BENCH_TOOLS_ARGS = $(if $(TOOLS),--tools $(TOOLS),)
BENCH_EXPERIMENTAL_ARGS = $(if $(filter 1 true yes,$(ALLOW_EXPERIMENTAL)),--allow-experimental,)

benchmark-fastq-stage: ## Benchmark FASTQ stage via CLI (requires STAGE=<stage> SAMPLE_ID R1, optional R2)
	@BIJUX_BIN="$(BIJUX_BIN)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" R2="$(R2)" STAGE="$(STAGE)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)" ./scripts/tooling/benchmarks.sh fastq-stage

benchmark-trim: ## Benchmark adapter/quality trimming tools
	@$(MAKE) benchmark-fastq-stage STAGE=trim SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)"

benchmark-validate: ## Benchmark read validation tools
	@$(MAKE) benchmark-fastq-stage STAGE=validate SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)"

benchmark-filter: ## Benchmark contaminant filtering tools
	@$(MAKE) benchmark-fastq-stage STAGE=filter SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)"

benchmark-merge: ## Benchmark read merging tools (paired-end)
	@$(MAKE) benchmark-fastq-stage STAGE=merge SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" R2="$(R2)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)"

benchmark-correct: ## Benchmark error correction tools (paired-end)
	@$(MAKE) benchmark-fastq-stage STAGE=correct SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" R2="$(R2)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)"

benchmark-qc-post: ## Benchmark post-processing QC tools
	@$(MAKE) benchmark-fastq-stage STAGE=qc-post SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)"

benchmark-umi: ## Benchmark UMI processing tools (paired-end)
	@$(MAKE) benchmark-fastq-stage STAGE=umi SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" R2="$(R2)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)"

benchmark-stats: ## Benchmark statistics computation tools
	@$(MAKE) benchmark-fastq-stage STAGE=stats SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)"

benchmark-screen: ## Benchmark screening tools
	@$(MAKE) benchmark-fastq-stage STAGE=screen SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)"

benchmark-preprocess: ## Benchmark full preprocessing pipeline
	@BIJUX_BIN="$(BIJUX_BIN)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)" ./scripts/tooling/benchmarks.sh fastq-preprocess

benchmark-all: ## Run all FASTQ benchmarks sequentially for one explicit sample input
	@BIJUX_BIN="$(BIJUX_BIN)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" R2="$(R2)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)" ./scripts/tooling/benchmarks.sh fastq-all

benchmark-status: ## Show canonical benchmark suite/config directories and detected suites
	@BIJUX_BIN="$(BIJUX_BIN)" ./scripts/tooling/benchmarks.sh fastq-status

.PHONY: benchmark-fastq-stage benchmark-all benchmark-trim benchmark-validate benchmark-filter \
	benchmark-merge benchmark-correct benchmark-qc-post benchmark-umi \
	benchmark-stats benchmark-screen benchmark-preprocess benchmark-status
