##@ Lab / HPC

_lab-fastq: ## Run FASTQ pipelines with lab harness (requires configs/lab/config.toml or CONFIG_PATH)
	@cargo run -q -p bijux-dna-dev -- lab run run_pipelines

_lab-bam: ## Run BAM benchmarks with lab harness (requires configs/lab/config.toml or CONFIG_PATH)
	@cargo run -q -p bijux-dna-dev -- lab run run_bench

.PHONY: _lab-fastq _lab-bam
