SHELL := /bin/sh
PLATFORM ?= docker-mac-arm64

.PHONY: build-images test-images benchmark-trim benchmark-validate lint quality security test

build-images:
	cargo run --bin build_docker_images -- --platform $(PLATFORM)

test-images:
	cargo run --bin test_docker_images -- --platform $(PLATFORM)

test:
	cargo test --workspace

lint:
	cargo fmt --all -- --check
	cargo clippy --workspace --all-targets --all-features -- -D warnings

security:
	cargo audit

benchmark-trim:
	@set -e; \
	TOOLS="fastp,cutadapt,bbduk,adapterremoval,trimmomatic,trim_galore"; \
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
	TOOLS="seqtk,fastqc,fastqvalidator,fqtools"; \
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
