##@ Lab / HPC

_lab-fastq: ## Run FASTQ pipelines with lab harness (requires CORPUS_ROOT)
	@CORPUS_ROOT="$(CORPUS_ROOT)" ./scripts/run.sh lab run_pipelines

_lab-bam: ## Run BAM benchmarks with lab harness (requires CORPUS_ROOT)
	@CORPUS_ROOT="$(CORPUS_ROOT)" ./scripts/run.sh lab run_bench

.PHONY: _lab-fastq _lab-bam
