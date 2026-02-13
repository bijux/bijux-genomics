##@ Lab / HPC

lab-fastq: ## Run FASTQ pipelines with lab harness (requires CORPUS_ROOT)
	@CORPUS_ROOT="$(CORPUS_ROOT)" ./scripts/lab/run_pipelines.sh

lab-bam: ## Run BAM benchmarks with lab harness (requires CORPUS_ROOT)
	@CORPUS_ROOT="$(CORPUS_ROOT)" ./scripts/lab/run_bench.sh

.PHONY: lab-fastq lab-bam
