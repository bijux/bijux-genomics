##@ Performance Benchmarks

benchmark-all: ## Run all individual benchmarks sequentially
	@set -e; \
	$(MAKE) benchmark-validate FASTQ_ROOT_OVERRIDE="$(FASTQ_ROOT_OVERRIDE)"; \
	$(MAKE) benchmark-trim FASTQ_ROOT_OVERRIDE="$(FASTQ_ROOT_OVERRIDE)"; \
	$(MAKE) benchmark-merge FASTQ_ROOT_OVERRIDE="$(FASTQ_ROOT_OVERRIDE)"; \
	$(MAKE) benchmark-correct FASTQ_ROOT_OVERRIDE="$(FASTQ_ROOT_OVERRIDE)"; \
	$(MAKE) benchmark-filter FASTQ_ROOT_OVERRIDE="$(FASTQ_ROOT_OVERRIDE)"; \
	$(MAKE) benchmark-stats FASTQ_ROOT_OVERRIDE="$(FASTQ_ROOT_OVERRIDE)"; \
	$(MAKE) benchmark-qc-post FASTQ_ROOT_OVERRIDE="$(FASTQ_ROOT_OVERRIDE)"; \
	$(MAKE) benchmark-umi FASTQ_ROOT_OVERRIDE="$(FASTQ_ROOT_OVERRIDE)"; \
	$(MAKE) benchmark-screen FASTQ_ROOT_OVERRIDE="$(FASTQ_ROOT_OVERRIDE)"; \
	$(MAKE) benchmark-preprocess FASTQ_ROOT_OVERRIDE="$(FASTQ_ROOT_OVERRIDE)"

benchmark-trim: ## Benchmark adapter/quality trimming tools
	@set -e; \
	TOOLS="$(TOOLS)"; \
	if [ -z "$$TOOLS" ]; then TOOLS="$(TOOLS_TRIM)"; fi; \
	OUT_DIR="."; \
	if [ -n "$(FASTQ_ROOT_OVERRIDE)" ]; then FASTQ_ROOT="$(FASTQ_ROOT_OVERRIDE)"; else FASTQ_ROOT="scripts/lab/corpus/fastq"; fi; \
	if [ -z "$(FASTQ_ROOT_OVERRIDE)" ] && [ -d scripts/lab/corpus/fastq/canonical ]; then FASTQ_ROOT="scripts/lab/corpus/fastq/canonical"; fi; \
	ROOTS="$$FASTQ_ROOT"; \
	ROOTS=$$(echo $$ROOTS | tr "," " "); \
	FILES=""; \
	for root in $$ROOTS; do FILES="$$FILES $$(find $$root -type f -name '*_R1.fastq.gz' -o -name '*.fastq.gz')"; done; \
	FILES=$$(echo $$FILES | tr " " "\n" | sort | uniq); \
	if [ -z "$$FILES" ]; then \
		echo "no FASTQ files found in scripts/lab/corpus/fastq"; \
		exit 1; \
	fi; \
	for file in $$FILES; do \
		sample_id=$$(basename "$$file" .fastq.gz); \
		echo "→ benchmark trim $$sample_id"; \
		cargo run --bin bijux -- fastq trim --env docker --tools $$TOOLS --sample-id "$$sample_id" --r1 "$$file" --out "$$OUT_DIR"; \
	done

benchmark-validate: ## Benchmark read validation tools
	@set -e; \
	TOOLS="$(TOOLS)"; \
	if [ -z "$$TOOLS" ]; then TOOLS="$(TOOLS_VALIDATE)"; fi; \
	OUT_DIR="."; \
	if [ -n "$(FASTQ_ROOT_OVERRIDE)" ]; then FASTQ_ROOT="$(FASTQ_ROOT_OVERRIDE)"; else FASTQ_ROOT="scripts/lab/corpus/fastq"; fi; \
	if [ -z "$(FASTQ_ROOT_OVERRIDE)" ] && [ -d scripts/lab/corpus/fastq/canonical ]; then FASTQ_ROOT="scripts/lab/corpus/fastq/canonical"; fi; \
	ROOTS="$$FASTQ_ROOT"; \
	ROOTS=$$(echo $$ROOTS | tr "," " "); \
	FILES=""; \
	for root in $$ROOTS; do FILES="$$FILES $$(find $$root -type f -name '*_R1.fastq.gz' -o -name '*.fastq.gz')"; done; \
	FILES=$$(echo $$FILES | tr " " "\n" | sort | uniq); \
	if [ -z "$$FILES" ]; then \
		echo "no FASTQ files found in scripts/lab/corpus/fastq"; \
		exit 1; \
	fi; \
	for file in $$FILES; do \
		sample_id=$$(basename "$$file" .fastq.gz); \
		echo "→ benchmark validate $$sample_id"; \
		cargo run --bin bijux -- fastq validate --env docker --tools $$TOOLS --sample-id "$$sample_id" --r1 "$$file" --out "$$OUT_DIR"; \
	done

benchmark-filter: ## Benchmark contaminant filtering tools
	@set -e; \
	TOOLS="$(TOOLS)"; \
	if [ -z "$$TOOLS" ]; then TOOLS="$(TOOLS_FILTER)"; fi; \
	OUT_DIR="."; \
	if [ -n "$(FASTQ_ROOT_OVERRIDE)" ]; then FASTQ_ROOT="$(FASTQ_ROOT_OVERRIDE)"; else FASTQ_ROOT="scripts/lab/corpus/fastq"; fi; \
	if [ -z "$(FASTQ_ROOT_OVERRIDE)" ] && [ -d scripts/lab/corpus/fastq/canonical ]; then FASTQ_ROOT="scripts/lab/corpus/fastq/canonical"; fi; \
	ROOTS="$$FASTQ_ROOT"; \
	ROOTS=$$(echo $$ROOTS | tr "," " "); \
	FILES=""; \
	for root in $$ROOTS; do FILES="$$FILES $$(find $$root -type f -name '*_R1.fastq.gz' -o -name '*.fastq.gz')"; done; \
	FILES=$$(echo $$FILES | tr " " "\n" | sort | uniq); \
	if [ -z "$$FILES" ]; then \
		echo "no FASTQ files found in scripts/lab/corpus/fastq"; \
		exit 1; \
	fi; \
	for file in $$FILES; do \
		sample_id=$$(basename "$$file" .fastq.gz); \
		echo "→ benchmark filter $$sample_id"; \
		cargo run --bin bijux -- bench fastq filter --sample-id "$$sample_id" --r1 "$$file" --out "$$OUT_DIR" --tools $$TOOLS; \
	done

benchmark-merge: ## Benchmark read merging tools (paired-end)
	@set -e; \
	TOOLS="$(TOOLS)"; \
	if [ -z "$$TOOLS" ]; then TOOLS="$(TOOLS_MERGE)"; fi; \
	OUT_DIR="."; \
	if [ -n "$(FASTQ_ROOT_OVERRIDE)" ]; then FASTQ_ROOT="$(FASTQ_ROOT_OVERRIDE)"; else FASTQ_ROOT="scripts/lab/corpus/fastq"; fi; \
	if [ -z "$(FASTQ_ROOT_OVERRIDE)" ] && [ -d scripts/lab/corpus/fastq/canonical ]; then FASTQ_ROOT="scripts/lab/corpus/fastq/canonical"; fi; \
	ROOTS="$$FASTQ_ROOT"; \
	ROOTS=$$(echo $$ROOTS | tr "," " "); \
	FILES=""; \
	for root in $$ROOTS; do FILES="$$FILES $$(find $$root -type f -name '*_1.fastq.gz' -o -name '*_R1.fastq.gz')"; done; \
	FILES=$$(echo $$FILES | tr " " "\n" | sort | uniq); \
	if [ -z "$$FILES" ]; then \
		echo "no paired FASTQ files found in scripts/lab/corpus/fastq"; \
		exit 1; \
	fi; \
	for r1 in $$FILES; do \
		r2=$$(echo "$$r1" | sed 's/_1.fastq.gz/_2.fastq.gz/; s/_R1.fastq.gz/_R2.fastq.gz/'); \
		sample_id=$$(basename "$$r1" _1.fastq.gz | sed 's/_R1$$//'); \
		echo "→ benchmark merge $$sample_id"; \
		cargo run --bin bijux -- bench fastq merge --sample-id "$$sample_id" --r1 "$$r1" --r2 "$$r2" --out "$$OUT_DIR" --tools $$TOOLS; \
	done

benchmark-correct: ## Benchmark error correction tools (paired-end)
	@set -e; \
	TOOLS="$(TOOLS)"; \
	if [ -z "$$TOOLS" ]; then TOOLS="$(TOOLS_CORRECT)"; fi; \
	OUT_DIR="."; \
	if [ -n "$(FASTQ_ROOT_OVERRIDE)" ]; then FASTQ_ROOT="$(FASTQ_ROOT_OVERRIDE)"; else FASTQ_ROOT="scripts/lab/corpus/fastq"; fi; \
	if [ -z "$(FASTQ_ROOT_OVERRIDE)" ] && [ -d scripts/lab/corpus/fastq/canonical ]; then FASTQ_ROOT="scripts/lab/corpus/fastq/canonical"; fi; \
	ROOTS="$$FASTQ_ROOT"; \
	ROOTS=$$(echo $$ROOTS | tr "," " "); \
	FILES=""; \
	for root in $$ROOTS; do FILES="$$FILES $$(find $$root -type f -name '*_1.fastq.gz' -o -name '*_R1.fastq.gz')"; done; \
	FILES=$$(echo $$FILES | tr " " "\n" | sort | uniq); \
	if [ -z "$$FILES" ]; then \
		echo "no paired FASTQ files found in scripts/lab/corpus/fastq"; \
		exit 1; \
	fi; \
	for r1 in $$FILES; do \
		r2=$$(echo "$$r1" | sed 's/_1.fastq.gz/_2.fastq.gz/; s/_R1.fastq.gz/_R2.fastq.gz/'); \
		sample_id=$$(basename "$$r1" _1.fastq.gz | sed 's/_R1$$//'); \
		echo "→ benchmark correct $$sample_id"; \
		cargo run --bin bijux -- bench fastq correct --sample-id "$$sample_id" --r1 "$$r1" --r2 "$$r2" --out "$$OUT_DIR" --tools $$TOOLS; \
	done

benchmark-qc-post: ## Benchmark post-processing QC tools
	@set -e; \
	TOOLS="$(TOOLS)"; \
	if [ -z "$$TOOLS" ]; then TOOLS="$(TOOLS_QC_POST)"; fi; \
	OUT_DIR="."; \
	if [ -n "$(FASTQ_ROOT_OVERRIDE)" ]; then FASTQ_ROOT="$(FASTQ_ROOT_OVERRIDE)"; else FASTQ_ROOT="scripts/lab/corpus/fastq"; fi; \
	if [ -z "$(FASTQ_ROOT_OVERRIDE)" ] && [ -d scripts/lab/corpus/fastq/canonical ]; then FASTQ_ROOT="scripts/lab/corpus/fastq/canonical"; fi; \
	ROOTS="$$FASTQ_ROOT"; \
	ROOTS=$$(echo $$ROOTS | tr "," " "); \
	FILES=""; \
	for root in $$ROOTS; do FILES="$$FILES $$(find $$root -type f -name '*_R1.fastq.gz' -o -name '*.fastq.gz')"; done; \
	FILES=$$(echo $$FILES | tr " " "\n" | sort | uniq); \
	if [ -z "$$FILES" ]; then \
		echo "no FASTQ files found in scripts/lab/corpus/fastq"; \
		exit 1; \
	fi; \
	for file in $$FILES; do \
		sample_id=$$(basename "$$file" .fastq.gz); \
		echo "→ benchmark qc_post $$sample_id"; \
		cargo run --bin bijux -- bench fastq qc-post --sample-id "$$sample_id" --r1 "$$file" --out "$$OUT_DIR" --tools $$TOOLS; \
	done

benchmark-umi: ## Benchmark UMI processing tools (paired-end)
	@set -e; \
	TOOLS="$(TOOLS)"; \
	if [ -z "$$TOOLS" ]; then TOOLS="$(TOOLS_UMI)"; fi; \
	OUT_DIR="."; \
	if [ -n "$(FASTQ_ROOT_OVERRIDE)" ]; then FASTQ_ROOT="$(FASTQ_ROOT_OVERRIDE)"; else FASTQ_ROOT="scripts/lab/corpus/fastq"; fi; \
	if [ -z "$(FASTQ_ROOT_OVERRIDE)" ] && [ -d scripts/lab/corpus/fastq/canonical ]; then FASTQ_ROOT="scripts/lab/corpus/fastq/canonical"; fi; \
	ROOTS="$$FASTQ_ROOT"; \
	ROOTS=$$(echo $$ROOTS | tr "," " "); \
	FILES=""; \
	for root in $$ROOTS; do FILES="$$FILES $$(find $$root -type f -name '*_1.fastq.gz' -o -name '*_R1.fastq.gz')"; done; \
	FILES=$$(echo $$FILES | tr " " "\n" | sort | uniq); \
	if [ -z "$$FILES" ]; then \
		echo "no paired FASTQ files found in scripts/lab/corpus/fastq"; \
		exit 1; \
	fi; \
	for r1 in $$FILES; do \
		r2=$$(echo "$$r1" | sed 's/_1.fastq.gz/_2.fastq.gz/; s/_R1.fastq.gz/_R2.fastq.gz/'); \
		sample_id=$$(basename "$$r1" _1.fastq.gz | sed 's/_R1$$//'); \
		echo "→ benchmark umi $$sample_id"; \
		cargo run --bin bijux -- bench fastq umi --sample-id "$$sample_id" --r1 "$$r1" --r2 "$$r2" --out "$$OUT_DIR" --tools $$TOOLS; \
	done

benchmark-stats: ## Benchmark statistics computation tools
	@set -e; \
	TOOLS="$(TOOLS)"; \
	if [ -z "$$TOOLS" ]; then TOOLS="$(TOOLS_STATS)"; fi; \
	OUT_DIR="."; \
	if [ -n "$(FASTQ_ROOT_OVERRIDE)" ]; then FASTQ_ROOT="$(FASTQ_ROOT_OVERRIDE)"; else FASTQ_ROOT="scripts/lab/corpus/fastq"; fi; \
	if [ -z "$(FASTQ_ROOT_OVERRIDE)" ] && [ -d scripts/lab/corpus/fastq/canonical ]; then FASTQ_ROOT="scripts/lab/corpus/fastq/canonical"; fi; \
	ROOTS="$$FASTQ_ROOT"; \
	ROOTS=$$(echo $$ROOTS | tr "," " "); \
	FILES=""; \
	for root in $$ROOTS; do FILES="$$FILES $$(find $$root -type f -name '*_R1.fastq.gz' -o -name '*.fastq.gz')"; done; \
	FILES=$$(echo $$FILES | tr " " "\n" | sort | uniq); \
	if [ -z "$$FILES" ]; then \
		echo "no FASTQ files found in scripts/lab/corpus/fastq"; \
		exit 1; \
	fi; \
	for file in $$FILES; do \
		sample_id=$$(basename "$$file" .fastq.gz); \
		echo "→ benchmark stats $$sample_id"; \
		cargo run --bin bijux -- bench fastq stats --sample-id "$$sample_id" --r1 "$$file" --out "$$OUT_DIR" --tools $$TOOLS; \
	done

benchmark-screen: ## Benchmark screening tools
	@set -e; \
	TOOLS="$(TOOLS)"; \
	if [ -z "$$TOOLS" ]; then TOOLS="$(TOOLS_SCREEN)"; fi; \
	OUT_DIR="."; \
	if [ -n "$(FASTQ_ROOT_OVERRIDE)" ]; then FASTQ_ROOT="$(FASTQ_ROOT_OVERRIDE)"; else FASTQ_ROOT="scripts/lab/corpus/fastq"; fi; \
	if [ -z "$(FASTQ_ROOT_OVERRIDE)" ] && [ -d scripts/lab/corpus/fastq/canonical ]; then FASTQ_ROOT="scripts/lab/corpus/fastq/canonical"; fi; \
	ROOTS="$$FASTQ_ROOT"; \
	ROOTS=$$(echo $$ROOTS | tr "," " "); \
	FILES=""; \
	for root in $$ROOTS; do FILES="$$FILES $$(find $$root -type f -name '*_R1.fastq.gz' -o -name '*.fastq.gz')"; done; \
	FILES=$$(echo $$FILES | tr " " "\n" | sort | uniq); \
	if [ -z "$$FILES" ]; then \
		echo "no FASTQ files found in scripts/lab/corpus/fastq"; \
		exit 1; \
	fi; \
	for file in $$FILES; do \
		sample_id=$$(basename "$$file" .fastq.gz); \
		echo "→ benchmark screen $$sample_id"; \
		cargo run --bin bijux -- bench fastq screen --sample-id "$$sample_id" --r1 "$$file" --out "$$OUT_DIR" --tools $$TOOLS; \
	done

benchmark-preprocess: ## Benchmark full preprocessing pipeline
	@set -e; \
	OUT_DIR="."; \
	if [ -n "$(FASTQ_ROOT_OVERRIDE)" ]; then FASTQ_ROOT="$(FASTQ_ROOT_OVERRIDE)"; else FASTQ_ROOT="scripts/lab/corpus/fastq"; fi; \
	if [ -z "$(FASTQ_ROOT_OVERRIDE)" ] && [ -d scripts/lab/corpus/fastq/canonical ]; then FASTQ_ROOT="scripts/lab/corpus/fastq/canonical"; fi; \
	ROOTS="$$FASTQ_ROOT"; \
	ROOTS=$$(echo $$ROOTS | tr "," " "); \
	FILES=""; \
	for root in $$ROOTS; do FILES="$$FILES $$(find $$root -type f -name '*_R1.fastq.gz' -o -name '*.fastq.gz')"; done; \
	FILES=$$(echo $$FILES | tr " " "\n" | sort | uniq); \
	if [ -z "$$FILES" ]; then \
		echo "no FASTQ files found in scripts/lab/corpus/fastq"; \
		exit 1; \
	fi; \
	for file in $$FILES; do \
		sample_id=$$(basename "$$file" .fastq.gz); \
		echo "→ benchmark preprocess $$sample_id"; \
		cargo run --bin bijux -- fastq preprocess --sample-id "$$sample_id" --r1 "$$file" --out "$$OUT_DIR"; \
	done

.PHONY: benchmark-all benchmark-trim benchmark-validate benchmark-filter \
        benchmark-merge benchmark-correct benchmark-qc-post benchmark-umi \
        benchmark-stats benchmark-screen benchmark-preprocess
