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
	@if [ -z "$(STAGE)" ] || [ -z "$(SAMPLE_ID)" ] || [ -z "$(R1)" ]; then \
		echo "ERROR: set STAGE=<trim|validate|...> SAMPLE_ID=<id> R1=<path>"; \
		exit 2; \
	fi
	@if [ -n "$(R2)" ]; then \
		$(BIJUX_BIN) bench fastq "$(STAGE)" --sample-id "$(SAMPLE_ID)" --r1 "$(R1)" --r2 "$(R2)" --out "$(OUT_DIR)" $(BENCH_TOOLS_ARGS) $(BENCH_EXPERIMENTAL_ARGS); \
	else \
		$(BIJUX_BIN) bench fastq "$(STAGE)" --sample-id "$(SAMPLE_ID)" --r1 "$(R1)" --out "$(OUT_DIR)" $(BENCH_TOOLS_ARGS) $(BENCH_EXPERIMENTAL_ARGS); \
	fi

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
	@$(BIJUX_BIN) bench fastq preprocess --sample-id "$(SAMPLE_ID)" --r1 "$(R1)" --out "$(OUT_DIR)" $(BENCH_TOOLS_ARGS) $(BENCH_EXPERIMENTAL_ARGS)

benchmark-all: ## Run all FASTQ benchmarks sequentially for one explicit sample input
	@set -e; \
	$(MAKE) benchmark-validate SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)"; \
	$(MAKE) benchmark-trim SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)"; \
	$(MAKE) benchmark-filter SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)"; \
	$(MAKE) benchmark-stats SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)"; \
	$(MAKE) benchmark-qc-post SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)"; \
	$(MAKE) benchmark-screen SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)"; \
	$(MAKE) benchmark-preprocess SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)"; \
	if [ -n "$(R2)" ]; then \
		$(MAKE) benchmark-merge SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" R2="$(R2)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)"; \
		$(MAKE) benchmark-correct SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" R2="$(R2)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)"; \
		$(MAKE) benchmark-umi SAMPLE_ID="$(SAMPLE_ID)" R1="$(R1)" R2="$(R2)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)" ALLOW_EXPERIMENTAL="$(ALLOW_EXPERIMENTAL)"; \
	fi

.PHONY: benchmark-fastq-stage benchmark-all benchmark-trim benchmark-validate benchmark-filter \
	benchmark-merge benchmark-correct benchmark-qc-post benchmark-umi \
	benchmark-stats benchmark-screen benchmark-preprocess
