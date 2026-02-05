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

.PHONY: build-images test-images image-qa test-images-trim test-images-validate test-images-filter test-images-merge
