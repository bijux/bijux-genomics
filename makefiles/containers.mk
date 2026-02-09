##@ Docker Container Management

build-images: ## Build all Docker images for the specified platform
	cargo run --bin build_docker_images -- --platform $(PLATFORM)

test-images: ## Test all Docker images for the specified platform
	cargo run --bin test_docker_images -- --platform $(PLATFORM)

image-qa: ## Run quality assurance checks on Docker images
	cargo run --bin image_qa -- --platform $(PLATFORM)

test-images-trim: ## Test trimming tool images (fastp, cutadapt, bbduk, adapterremoval, trimmomatic, trim_galore)
	cargo run --bin test_docker_images -- --platform $(PLATFORM) --tools fastp,cutadapt,bbduk,adapterremoval,trimmomatic,trim_galore

test-images-validate: ## Test validation tool images (seqtk, fastqc, fastqvalidator, fqtools)
	cargo run --bin test_docker_images -- --platform $(PLATFORM) --tools seqtk,fastqc,fastqvalidator,fqtools

test-images-filter: ## Test filtering tool images (bbduk)
	cargo run --bin test_docker_images -- --platform $(PLATFORM) --tools bbduk

test-images-merge: ## Test merging tool images (pear, flash2)
	cargo run --bin test_docker_images -- --platform $(PLATFORM) --tools pear,flash2

smoke-containers-docker-arm64: ## Build+smoke Docker arm64 containers (artifacts/container/{logs,images})
	sh scripts/smoke-containers-docker-arm64.sh

smoke-containers-apptainer: ## Build+smoke Apptainer containers (artifacts/container/{logs,images})
	sh scripts/smoke-containers-apptainer.sh

test-images-fastq-preprocess: ## FASTQ stage fastq.preprocess
	cargo run --bin test_docker_images -- --platform $(PLATFORM) --tools fastp,fastqvalidator_official

test-images-fastq-prepare-reference: ## FASTQ stage core.prepare_reference
	cargo run --bin test_docker_images -- --platform $(PLATFORM) --tools samtools

test-images-fastq-validate-pre: ## FASTQ stage fastq.validate_pre
	cargo run --bin test_docker_images -- --platform $(PLATFORM) --tools seqtk,fastqc,fastqvalidator,fastqvalidator_official,fqtools

test-images-fastq-detect-adapters: ## FASTQ stage fastq.detect_adapters
	cargo run --bin test_docker_images -- --platform $(PLATFORM) --tools fastqc

test-images-fastq-trim: ## FASTQ stage fastq.trim
	cargo run --bin test_docker_images -- --platform $(PLATFORM) --tools fastp,cutadapt,atropos,bbduk,adapterremoval,trimmomatic,trim_galore,seqpurge,prinseq,seqkit

test-images-fastq-filter: ## FASTQ stage fastq.filter
	cargo run --bin test_docker_images -- --platform $(PLATFORM) --tools prinseq,fastp,seqkit,bbduk

test-images-fastq-stats-neutral: ## FASTQ stage fastq.stats_neutral
	cargo run --bin test_docker_images -- --platform $(PLATFORM) --tools seqkit

test-images-fastq-qc-post: ## FASTQ stage fastq.qc_post
	cargo run --bin test_docker_images -- --platform $(PLATFORM) --tools fastqc,multiqc

test-images-fastq-merge: ## FASTQ stage fastq.merge
	cargo run --bin test_docker_images -- --platform $(PLATFORM) --tools pear,vsearch,bbmerge,flash2

test-images-fastq-correct: ## FASTQ stage fastq.correct
	cargo run --bin test_docker_images -- --platform $(PLATFORM) --tools rcorrector,spades,bayeshammer,lighter,musket

test-images-fastq-umi: ## FASTQ stage fastq.umi
	cargo run --bin test_docker_images -- --platform $(PLATFORM) --tools umi_tools

test-images-fastq-screen: ## FASTQ stage fastq.screen
	cargo run --bin test_docker_images -- --platform $(PLATFORM) --tools kraken2,centrifuge,metaphlan,kaiju,fastq_screen

test-images-bam-align: ## BAM stage bam.align
	TOOLS=bwa,bowtie2 sh scripts/smoke-containers-docker-arm64.sh

test-images-bam-validate: ## BAM stage bam.validate
	TOOLS=samtools sh scripts/smoke-containers-docker-arm64.sh

test-images-bam-qc-pre: ## BAM stage bam.qc_pre
	TOOLS=samtools sh scripts/smoke-containers-docker-arm64.sh

test-images-bam-filter: ## BAM stage bam.filter
	TOOLS=samtools sh scripts/smoke-containers-docker-arm64.sh

test-images-bam-markdup: ## BAM stage bam.markdup
	TOOLS=gatk,samtools sh scripts/smoke-containers-docker-arm64.sh

test-images-bam-complexity: ## BAM stage bam.complexity
	TOOLS=preseq sh scripts/smoke-containers-docker-arm64.sh

test-images-bam-coverage: ## BAM stage bam.coverage
	TOOLS=mosdepth,samtools sh scripts/smoke-containers-docker-arm64.sh

test-images-bam-damage: ## BAM stage bam.damage
	TOOLS=pydamage,mapdamage2 sh scripts/smoke-containers-docker-arm64.sh

test-images-bam-authenticity: ## BAM stage bam.authenticity
	TOOLS=authenticct sh scripts/smoke-containers-docker-arm64.sh

test-images-bam-contamination: ## BAM stage bam.contamination
	TOOLS=authenticct sh scripts/smoke-containers-docker-arm64.sh

test-images-bam-sex: ## BAM stage bam.sex
	TOOLS=rxy sh scripts/smoke-containers-docker-arm64.sh

test-images-bam-bias-mitigation: ## BAM stage bam.bias_mitigation
	TOOLS=angsd sh scripts/smoke-containers-docker-arm64.sh

test-images-bam-recalibration: ## BAM stage bam.recalibration
	TOOLS=gatk sh scripts/smoke-containers-docker-arm64.sh

test-images-bam-haplogroups: ## BAM stage bam.haplogroups
	TOOLS=yleaf sh scripts/smoke-containers-docker-arm64.sh

test-images-bam-genotyping: ## BAM stage bam.genotyping
	TOOLS=angsd sh scripts/smoke-containers-docker-arm64.sh

test-images-bam-kinship: ## BAM stage bam.kinship
	TOOLS=king sh scripts/smoke-containers-docker-arm64.sh

.PHONY: build-images test-images image-qa test-images-trim test-images-validate test-images-filter test-images-merge \
	smoke-containers-docker-arm64 smoke-containers-apptainer \
	test-images-fastq-preprocess test-images-fastq-prepare-reference test-images-fastq-validate-pre \
	test-images-fastq-detect-adapters test-images-fastq-trim test-images-fastq-filter \
	test-images-fastq-stats-neutral test-images-fastq-qc-post test-images-fastq-merge \
	test-images-fastq-correct test-images-fastq-umi test-images-fastq-screen \
	test-images-bam-align test-images-bam-validate test-images-bam-qc-pre test-images-bam-filter \
	test-images-bam-markdup test-images-bam-complexity test-images-bam-coverage test-images-bam-damage \
	test-images-bam-authenticity test-images-bam-contamination test-images-bam-sex \
	test-images-bam-bias-mitigation test-images-bam-recalibration test-images-bam-haplogroups \
	test-images-bam-genotyping test-images-bam-kinship
