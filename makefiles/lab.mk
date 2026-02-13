##@ Lab / HPC

lab-fastq: ## Run FASTQ pipelines with lab harness (requires CORPUS_ROOT)
	@CORPUS_ROOT="$(CORPUS_ROOT)" ./scripts/run.sh lab run_pipelines

lab-bam: ## Run BAM benchmarks with lab harness (requires CORPUS_ROOT)
	@CORPUS_ROOT="$(CORPUS_ROOT)" ./scripts/run.sh lab run_bench

.PHONY: lab-fastq lab-bam
