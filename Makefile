SHELL 			:= /bin/sh
PLATFORM 		?= docker-mac-arm64
TOOLS_TRIM 		?= fastp,cutadapt,bbduk,adapterremoval,trimmomatic,trim_galore
TOOLS_VALIDATE 	?= seqtk,fastqc,fastqvalidator,fqtools
TOOLS_FILTER 	?= prinseq,fastp,seqkit
TOOLS_MERGE 	?= pear,vsearch,bbmerge,flash2

.PHONY: build-images test-images image-qa benchmark-trim benchmark-validate benchmark-filter benchmark-merge \
	test-images-trim test-images-validate test-images-filter test-images-merge lint quality security test

build-images:
	cargo run --bin build_docker_images -- --platform $(PLATFORM)

test-images:
	cargo run --bin test_docker_images -- --platform $(PLATFORM)

image-qa:
	cargo run --bin image_qa -- --platform $(PLATFORM)

test-images-trim:
	cargo run --bin test_docker_images -- --platform $(PLATFORM) --tools fastp,cutadapt,bbduk,adapterremoval,trimmomatic,trim_galore

test-images-validate:
	cargo run --bin test_docker_images -- --platform $(PLATFORM) --tools seqtk,fastqc,fastqvalidator,fqtools

test-images-filter:
	cargo run --bin test_docker_images -- --platform $(PLATFORM) --tools bbduk

test-images-merge:
	cargo run --bin test_docker_images -- --platform $(PLATFORM) --tools pear,flash2

test:
	cargo test --workspace

lint:
	cargo fmt --all -- --check
	cargo clippy --workspace --all-targets --all-features -- -D warnings

security:
	cargo audit

benchmark-trim:
	@set -e; \
	TOOLS="$(TOOLS)"; \
	if [ -z "$$TOOLS" ]; then TOOLS="$(TOOLS_TRIM)"; fi; \
	OUT_DIR="."; \
	FILES=$$(find tests/data/fastq -type f -name '*.fastq.gz' | sort); \
	if [ -z "$$FILES" ]; then \
		echo "no FASTQ files found in tests/data/fastq"; \
		exit 1; \
	fi; \
	for file in $$FILES; do \
		sample_id=$$(basename "$$file" .fastq.gz); \
		echo "→ benchmark trim $$sample_id"; \
		cargo run --bin bijux -- fastq trim --env docker --tools $$TOOLS --sample-id "$$sample_id" --r1 "$$file" --out "$$OUT_DIR"; \
	done

benchmark-validate:
	@set -e; \
	TOOLS="$(TOOLS)"; \
	if [ -z "$$TOOLS" ]; then TOOLS="$(TOOLS_VALIDATE)"; fi; \
	OUT_DIR="."; \
	FILES=$$(find tests/data/fastq -type f -name '*.fastq.gz' | sort); \
	if [ -z "$$FILES" ]; then \
		echo "no FASTQ files found in tests/data/fastq"; \
		exit 1; \
	fi; \
	for file in $$FILES; do \
		sample_id=$$(basename "$$file" .fastq.gz); \
		echo "→ benchmark validate $$sample_id"; \
		cargo run --bin bijux -- fastq validate --env docker --tools $$TOOLS --sample-id "$$sample_id" --r1 "$$file" --out "$$OUT_DIR"; \
	done

benchmark-filter:
	@set -e; \
	TOOLS="$(TOOLS)"; \
	if [ -z "$$TOOLS" ]; then TOOLS="$(TOOLS_FILTER)"; fi; \
	OUT_DIR="."; \
	FILES=$$(find tests/data/fastq -type f -name '*.fastq.gz' | sort); \
	if [ -z "$$FILES" ]; then \
		echo "no FASTQ files found in tests/data/fastq"; \
		exit 1; \
	fi; \
	for file in $$FILES; do \
		sample_id=$$(basename "$$file" .fastq.gz); \
		echo "→ benchmark filter $$sample_id"; \
		cargo run --bin bijux -- bench fastq filter --sample-id "$$sample_id" --r1 "$$file" --out "$$OUT_DIR" --tools $$TOOLS; \
	done

benchmark-merge:
	@set -e; \
	TOOLS="$(TOOLS)"; \
	if [ -z "$$TOOLS" ]; then TOOLS="$(TOOLS_MERGE)"; fi; \
	OUT_DIR="."; \
	FILES=$$(find tests/data/fastq -type f -name '*_1.fastq.gz' | sort); \
	if [ -z "$$FILES" ]; then \
		echo "no paired FASTQ files found in tests/data/fastq"; \
		exit 1; \
	fi; \
	for r1 in $$FILES; do \
		r2=$$(echo "$$r1" | sed 's/_1.fastq.gz/_2.fastq.gz/'); \
		if [ ! -f "$$r2" ]; then \
			echo "missing pair for $$r1"; \
			exit 1; \
		fi; \
		sample_id=$$(basename "$$r1" _1.fastq.gz); \
		echo "→ benchmark merge $$sample_id"; \
		cargo run --bin bijux -- bench fastq merge --sample-id "$$sample_id" --r1 "$$r1" --r2 "$$r2" --out "$$OUT_DIR" --tools $$TOOLS; \
	done
