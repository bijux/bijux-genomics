##@ Lab / HPC

lab-fastq: ## Run FASTQ pipelines with lab harness (requires CORPUS_ROOT)
	@if [ -z "$(CORPUS_ROOT)" ]; then \
		echo "CORPUS_ROOT is required (e.g., make lab-fastq CORPUS_ROOT=/data/corpus)"; \
		exit 1; \
	fi
	@CORPUS_ROOT="$(CORPUS_ROOT)" scripts/lab/run_pipelines.sh

lab-bam: ## Run BAM benchmarks with lab harness (requires CORPUS_ROOT)
	@if [ -z "$(CORPUS_ROOT)" ]; then \
		echo "CORPUS_ROOT is required (e.g., make lab-bam CORPUS_ROOT=/data/corpus)"; \
		exit 1; \
	fi
	@CORPUS_ROOT="$(CORPUS_ROOT)" scripts/lab/run_bench.sh

.PHONY: lab-fastq lab-bam
