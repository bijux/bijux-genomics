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
