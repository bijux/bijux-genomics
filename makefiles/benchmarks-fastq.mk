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

_benchmark-fastq-stage: ## Benchmark FASTQ stage via CLI (requires STAGE=<stage> SAMPLE_ID R1, optional R2)
	@BIJUX_BIN="$(BIJUX_BIN)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" R2="$(R2)" STAGE="$(STAGE)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)" ./scripts/run.sh tooling benchmarks fastq-stage

_benchmark-trim: ## Benchmark adapter/quality trimming tools
	@$(MAKE) _benchmark-fastq-stage STAGE=trim SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)"

_benchmark-validate: ## Benchmark read validation tools
	@$(MAKE) _benchmark-fastq-stage STAGE=validate SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)"

_benchmark-filter: ## Benchmark contaminant filtering tools
	@$(MAKE) _benchmark-fastq-stage STAGE=filter SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)"

_benchmark-merge: ## Benchmark read merging tools (paired-end)
	@$(MAKE) _benchmark-fastq-stage STAGE=merge SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" R2="$(R2)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)"

_benchmark-correct: ## Benchmark error correction tools (paired-end)
	@$(MAKE) _benchmark-fastq-stage STAGE=correct SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" R2="$(R2)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)"

_benchmark-qc-post: ## Benchmark post-processing QC tools
	@$(MAKE) _benchmark-fastq-stage STAGE=qc-post SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)"

_benchmark-umi: ## Benchmark UMI processing tools (paired-end)
	@$(MAKE) _benchmark-fastq-stage STAGE=umi SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" R2="$(R2)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)"

_benchmark-stats: ## Benchmark statistics computation tools
	@$(MAKE) _benchmark-fastq-stage STAGE=stats SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)"

_benchmark-screen: ## Benchmark screening tools
	@$(MAKE) _benchmark-fastq-stage STAGE=screen SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)"

_benchmark-preprocess: ## Benchmark full preprocessing pipeline
	@BIJUX_BIN="$(BIJUX_BIN)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)" ./scripts/run.sh tooling benchmarks fastq-preprocess

_benchmark-all: ## Run all FASTQ benchmarks sequentially for one explicit sample input
	@BIJUX_BIN="$(BIJUX_BIN)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" R2="$(R2)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)" ./scripts/run.sh tooling benchmarks fastq-all

_benchmark-status: ## Show canonical benchmark suite/config directories and detected suites
	@BIJUX_BIN="$(BIJUX_BIN)" ./scripts/run.sh tooling benchmarks fastq-status

.PHONY: _benchmark-fastq-stage _benchmark-all _benchmark-trim _benchmark-validate _benchmark-filter \
	_benchmark-merge _benchmark-correct _benchmark-qc-post _benchmark-umi \
	_benchmark-stats _benchmark-screen _benchmark-preprocess _benchmark-status
