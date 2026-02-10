##@ Performance Benchmarks

BIJUX_BIN ?= cargo run --bin bijux-dna --
FASTQ_CORPUS ?= canonical
OUT_DIR ?= .
TOOLS ?=

BENCH_TOOLS_ARGS = $(if $(TOOLS),--tools $(TOOLS),)

# Internal generic runner:
#   STAGE_NAME (for logs), STAGE_CMD (CLI subcommand), PAIRED=0|1
benchmark-fastq-stage:
	@if [ -z "$(STAGE_NAME)" ] || [ -z "$(STAGE_CMD)" ]; then \
		echo "ERROR: benchmark-fastq-stage requires STAGE_NAME and STAGE_CMD"; \
		exit 2; \
	fi
	@set -e; \
	if [ "$(PAIRED)" = "1" ]; then \
		FILES="$$( $(BIJUX_BIN) lab corpus list-fastq --corpus $(FASTQ_CORPUS) --paired )"; \
	else \
		FILES="$$( $(BIJUX_BIN) lab corpus list-fastq --corpus $(FASTQ_CORPUS) )"; \
	fi; \
	if [ -z "$$FILES" ]; then \
		echo "no FASTQ files found for corpus $(FASTQ_CORPUS)"; \
		exit 1; \
	fi; \
	for r1 in $$FILES; do \
		sample_id=$$(basename "$$r1" .fastq.gz | sed 's/_R1$$//' | sed 's/_1$$/'); \
		echo "-> benchmark $(STAGE_NAME) $$sample_id"; \
		if [ "$(PAIRED)" = "1" ]; then \
			r2=$$(echo "$$r1" | sed 's/_1.fastq.gz/_2.fastq.gz/; s/_R1.fastq.gz/_R2.fastq.gz/'); \
			$(BIJUX_BIN) bench fastq $(STAGE_CMD) --sample-id "$$sample_id" --r1 "$$r1" --r2 "$$r2" --out "$(OUT_DIR)" $(BENCH_TOOLS_ARGS); \
		else \
			$(BIJUX_BIN) bench fastq $(STAGE_CMD) --sample-id "$$sample_id" --r1 "$$r1" --out "$(OUT_DIR)" $(BENCH_TOOLS_ARGS); \
		fi; \
	done

benchmark-trim: ## Benchmark adapter/quality trimming tools
	@$(MAKE) benchmark-fastq-stage STAGE_NAME=trim STAGE_CMD=trim PAIRED=0 FASTQ_CORPUS="$(FASTQ_CORPUS)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)"

benchmark-validate: ## Benchmark read validation tools
	@$(MAKE) benchmark-fastq-stage STAGE_NAME=validate STAGE_CMD=validate PAIRED=0 FASTQ_CORPUS="$(FASTQ_CORPUS)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)"

benchmark-filter: ## Benchmark contaminant filtering tools
	@$(MAKE) benchmark-fastq-stage STAGE_NAME=filter STAGE_CMD=filter PAIRED=0 FASTQ_CORPUS="$(FASTQ_CORPUS)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)"

benchmark-merge: ## Benchmark read merging tools (paired-end)
	@$(MAKE) benchmark-fastq-stage STAGE_NAME=merge STAGE_CMD=merge PAIRED=1 FASTQ_CORPUS="$(FASTQ_CORPUS)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)"

benchmark-correct: ## Benchmark error correction tools (paired-end)
	@$(MAKE) benchmark-fastq-stage STAGE_NAME=correct STAGE_CMD=correct PAIRED=1 FASTQ_CORPUS="$(FASTQ_CORPUS)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)"

benchmark-qc-post: ## Benchmark post-processing QC tools
	@$(MAKE) benchmark-fastq-stage STAGE_NAME=qc-post STAGE_CMD=qc-post PAIRED=0 FASTQ_CORPUS="$(FASTQ_CORPUS)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)"

benchmark-umi: ## Benchmark UMI processing tools (paired-end)
	@$(MAKE) benchmark-fastq-stage STAGE_NAME=umi STAGE_CMD=umi PAIRED=1 FASTQ_CORPUS="$(FASTQ_CORPUS)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)"

benchmark-stats: ## Benchmark statistics computation tools
	@$(MAKE) benchmark-fastq-stage STAGE_NAME=stats STAGE_CMD=stats PAIRED=0 FASTQ_CORPUS="$(FASTQ_CORPUS)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)"

benchmark-screen: ## Benchmark screening tools
	@$(MAKE) benchmark-fastq-stage STAGE_NAME=screen STAGE_CMD=screen PAIRED=0 FASTQ_CORPUS="$(FASTQ_CORPUS)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)"

benchmark-preprocess: ## Benchmark full preprocessing pipeline
	@$(MAKE) benchmark-fastq-stage STAGE_NAME=preprocess STAGE_CMD=preprocess PAIRED=0 FASTQ_CORPUS="$(FASTQ_CORPUS)" OUT_DIR="$(OUT_DIR)" TOOLS="$(TOOLS)"

benchmark-all: ## Run all individual benchmarks sequentially
	@set -e; \
	$(MAKE) benchmark-validate FASTQ_CORPUS="$(FASTQ_CORPUS)" TOOLS="$(TOOLS)" OUT_DIR="$(OUT_DIR)"; \
	$(MAKE) benchmark-trim FASTQ_CORPUS="$(FASTQ_CORPUS)" TOOLS="$(TOOLS)" OUT_DIR="$(OUT_DIR)"; \
	$(MAKE) benchmark-merge FASTQ_CORPUS="$(FASTQ_CORPUS)" TOOLS="$(TOOLS)" OUT_DIR="$(OUT_DIR)"; \
	$(MAKE) benchmark-correct FASTQ_CORPUS="$(FASTQ_CORPUS)" TOOLS="$(TOOLS)" OUT_DIR="$(OUT_DIR)"; \
	$(MAKE) benchmark-filter FASTQ_CORPUS="$(FASTQ_CORPUS)" TOOLS="$(TOOLS)" OUT_DIR="$(OUT_DIR)"; \
	$(MAKE) benchmark-stats FASTQ_CORPUS="$(FASTQ_CORPUS)" TOOLS="$(TOOLS)" OUT_DIR="$(OUT_DIR)"; \
	$(MAKE) benchmark-qc-post FASTQ_CORPUS="$(FASTQ_CORPUS)" TOOLS="$(TOOLS)" OUT_DIR="$(OUT_DIR)"; \
	$(MAKE) benchmark-umi FASTQ_CORPUS="$(FASTQ_CORPUS)" TOOLS="$(TOOLS)" OUT_DIR="$(OUT_DIR)"; \
	$(MAKE) benchmark-screen FASTQ_CORPUS="$(FASTQ_CORPUS)" TOOLS="$(TOOLS)" OUT_DIR="$(OUT_DIR)"; \
	$(MAKE) benchmark-preprocess FASTQ_CORPUS="$(FASTQ_CORPUS)" TOOLS="$(TOOLS)" OUT_DIR="$(OUT_DIR)"

.PHONY: benchmark-fastq-stage benchmark-all benchmark-trim benchmark-validate benchmark-filter \
	benchmark-merge benchmark-correct benchmark-qc-post benchmark-umi \
	benchmark-stats benchmark-screen benchmark-preprocess
