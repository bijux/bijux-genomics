##@ Container Management

# Runtime selector: docker-arm64 | docker-amd64 | apptainer
# System selector: local | hpc
SYSTEM_TYPE ?= local
ifeq ($(SYSTEM_TYPE),hpc)
CONTAINER_TYPE ?= apptainer
else
CONTAINER_TYPE ?= docker-arm64
endif

PLATFORM ?= docker-arm64
BIJUX_WORKERS ?= 1
JOBS ?= $(BIJUX_WORKERS)
TOOLS ?=
STAGE ?=
APPTAINER_VM_OUT ?= $(HOME)/apptainer-build
APPTAINER_COPY_BACK ?= $(if $(ISOLATE_ROOT),$(ISOLATE_ROOT)/containers/apptainer,artifacts/containers/apptainer)
CONTAINER_ARTIFACT_DIR ?= $(if $(ISOLATE_ROOT),$(ISOLATE_ROOT)/containers,artifacts/containers)
BIJUX_BIN ?= cargo run -q -p bijux-dna-dev -- tooling run bijux --
BIJUX_HPC_ROOT ?= $(HOME)/bijux

CT_KEY := $(subst -,_,$(CONTAINER_TYPE))
SMOKE_SCRIPT_docker_arm64 := smoke-docker-arm64
SMOKE_SCRIPT_docker_amd64 := smoke-docker-amd64
SMOKE_SCRIPT_apptainer := smoke-apptainer
SMOKE_SCRIPT := $(SMOKE_SCRIPT_$(CT_KEY))

_container-runtime-check: ## Validate selected container runtime
	@SYSTEM_TYPE="$(SYSTEM_TYPE)" CONTAINER_TYPE="$(CONTAINER_TYPE)" cargo run -q -p bijux-dna-dev -- containers run container-runtime-check

_env-prep: ## Prepare environment images via CLI (TOOL=<id> or STAGE=<domain.stage|stage>)
	@CONTAINER_TYPE="$(CONTAINER_TYPE)" TOOLS="$(TOOLS)" STAGE="$(STAGE)" BIJUX_BIN="$(BIJUX_BIN)" cargo run -q -p bijux-dna-dev -- containers run env-prep

_env-smoke: ## Smoke environment via CLI (TOOL=<id> or STAGE=<domain.stage|stage>)
	@CONTAINER_TYPE="$(CONTAINER_TYPE)" TOOLS="$(TOOLS)" STAGE="$(STAGE)" BIJUX_BIN="$(BIJUX_BIN)" cargo run -q -p bijux-dna-dev -- containers run env-smoke

_container-smoke: _container-runtime-check ## Prepare+smoke selected tool/stage via CLI
	@CONTAINER_TYPE="$(CONTAINER_TYPE)" TOOLS="$(TOOLS)" STAGE="$(STAGE)" BIJUX_BIN="$(BIJUX_BIN)" cargo run -q -p bijux-dna-dev -- containers run container-smoke

_containers-smoke: _container-runtime-check ## Prepare+smoke every registered stage via CLI
	@CONTAINER_TYPE="$(CONTAINER_TYPE)" BIJUX_BIN="$(BIJUX_BIN)" cargo run -q -p bijux-dna-dev -- containers run containers-smoke

_smoke-containers-docker-arm64: ## Build+smoke Docker arm64 containers
	@CONTAINER_TYPE="$(CONTAINER_TYPE)" TOOLS="$(TOOLS)" BIJUX_WORKERS="$(BIJUX_WORKERS)" CONTAINER_ARTIFACT_DIR="$(CONTAINER_ARTIFACT_DIR)" cargo run -q -p bijux-dna-dev -- containers run smoke-containers-docker-arm64

_smoke-containers-docker-amd64: ## Build+smoke Docker amd64 containers
	@CONTAINER_TYPE="$(CONTAINER_TYPE)" TOOLS="$(TOOLS)" BIJUX_WORKERS="$(BIJUX_WORKERS)" CONTAINER_ARTIFACT_DIR="$(CONTAINER_ARTIFACT_DIR)" cargo run -q -p bijux-dna-dev -- containers run smoke-containers-docker-amd64

_smoke-containers-apptainer: ## Build+smoke Apptainer containers
	@CONTAINER_TYPE="$(CONTAINER_TYPE)" TOOLS="$(TOOLS)" BIJUX_WORKERS="$(BIJUX_WORKERS)" CONTAINER_ARTIFACT_DIR="$(CONTAINER_ARTIFACT_DIR)" cargo run -q -p bijux-dna-dev -- containers run smoke-containers-apptainer

_smoke-containers-apptainer-bijux-run: ## Apptainer smoke in bijux-run mode (registry commands via exec).
	@TOOLS="$(TOOLS)" BIJUX_WORKERS="$(BIJUX_WORKERS)" CONTAINER_ARTIFACT_DIR="$(CONTAINER_ARTIFACT_DIR)" cargo run -q -p bijux-dna-dev -- containers run smoke-containers-apptainer-bijux-run

_smoke-containers-apptainer-apptainer-run: ## Apptainer smoke in runscript mode (apptainer run).
	@TOOLS="$(TOOLS)" BIJUX_WORKERS="$(BIJUX_WORKERS)" CONTAINER_ARTIFACT_DIR="$(CONTAINER_ARTIFACT_DIR)" cargo run -q -p bijux-dna-dev -- containers run smoke-containers-apptainer-apptainer-run

_smoke-containers-apptainer-verify: _smoke-containers-apptainer-bijux-run _smoke-containers-apptainer-apptainer-run ## Compare bijux-run vs apptainer-run smoke statuses.
	@CONTAINER_ARTIFACT_DIR="$(CONTAINER_ARTIFACT_DIR)" cargo run -q -p bijux-dna-dev -- containers run smoke-containers-apptainer-verify

_build-images: ## Build Docker images (docker-arm64 only)
	@CONTAINER_TYPE="$(CONTAINER_TYPE)" TOOLS="$(TOOLS)" BIJUX_WORKERS="$(BIJUX_WORKERS)" BIJUX_BIN="$(BIJUX_BIN)" CONTAINER_ARTIFACT_DIR="$(CONTAINER_ARTIFACT_DIR)" cargo run -q -p bijux-dna-dev -- containers run build-images

_test-images: ## Smoke selected runtime through the native control-plane registry
	@CONTAINER_TYPE="$(CONTAINER_TYPE)" TOOLS="$(TOOLS)" STAGE="$(STAGE)" BIJUX_WORKERS="$(BIJUX_WORKERS)" BIJUX_BIN="$(BIJUX_BIN)" CONTAINER_ARTIFACT_DIR="$(CONTAINER_ARTIFACT_DIR)" cargo run -q -p bijux-dna-dev -- containers run test-images

# Generic stage smoke: make _test-images-stage STAGE=fastq.trim (or STAGE=trim)
_test-images-stage: ## Smoke all tools for one stage via CLI registry
	@CONTAINER_TYPE="$(CONTAINER_TYPE)" STAGE="$(STAGE)" BIJUX_BIN="$(BIJUX_BIN)" cargo run -q -p bijux-dna-dev -- containers run test-images-stage

# Single tool smoke: make _test-images-tool TOOLS=<tool_id>
_test-images-tool: ## Smoke one tool via CLI registry
	@CONTAINER_TYPE="$(CONTAINER_TYPE)" TOOLS="$(TOOLS)" BIJUX_BIN="$(BIJUX_BIN)" cargo run -q -p bijux-dna-dev -- containers run test-images-tool

_image-smoke-vcf: ## Smoke only VCF tools and write manifests under isolate/container artifacts.
	@CONTAINER_TYPE="$(CONTAINER_TYPE)" BIJUX_WORKERS="$(BIJUX_WORKERS)" BIJUX_BIN="$(BIJUX_BIN)" CONTAINER_ARTIFACT_DIR="$(CONTAINER_ARTIFACT_DIR)" cargo run -q -p bijux-dna-dev -- containers run image-smoke-vcf

_image-qa: ## Run image QA (docker-arm64 only)
	@CONTAINER_TYPE="$(CONTAINER_TYPE)" PLATFORM="$(PLATFORM)" cargo run -q -p bijux-dna-dev -- containers run image-qa

_containers-apptainer-build: ## Batch-build Apptainer defs to VM-local output and copy back artifacts
	@BIJUX_WORKERS="$(BIJUX_WORKERS)" JOBS="$(BIJUX_WORKERS)" cargo run -q -p bijux-dna-dev -- containers run build-apptainer-all \
		--defs-dir containers/apptainer \
		--vm-out "$(APPTAINER_VM_OUT)" \
		--copy-back "$(APPTAINER_COPY_BACK)"

apptainer-build-all: ## Build+smoke every Apptainer runtime tool on frontend and refresh lock.
	@ARTIFACT_DIR="$(CONTAINER_ARTIFACT_DIR)/hpc/frontend-smoke" cargo run -q -p bijux-dna-dev -- containers run apptainer-build-all

docker-build-all: ## Build+smoke every docker-arm64 runtime tool and refresh lock.
	@ARTIFACT_DIR="$(CONTAINER_ARTIFACT_DIR)/docker-arm64" cargo run -q -p bijux-dna-dev -- containers run docker-build-all

_apptainer-ensure: ## Ensure apptainer images from SSOT stage list. Use DOMAIN=<domain> STAGES=<s1,s2>
	@DOMAIN="$(DOMAIN)" STAGES="$(STAGES)" BIJUX_HPC_ROOT="$(BIJUX_HPC_ROOT)" BIJUX_BIN="$(BIJUX_BIN)" cargo run -q -p bijux-dna-dev -- containers run apptainer-ensure

_apptainer-ensure-stage: ## Ensure apptainer image(s) for one stage via DOMAIN=<domain> STAGES=<stage>
	@DOMAIN="$(DOMAIN)" STAGES="$(STAGES)" BIJUX_HPC_ROOT="$(BIJUX_HPC_ROOT)" BIJUX_BIN="$(BIJUX_BIN)" cargo run -q -p bijux-dna-dev -- containers run apptainer-ensure-stage

_containers-lint: ## Lint container naming, headers, labels, and forbidden patterns
	@cargo run -q -p bijux-dna-dev -- containers run lint

_containers-ensure-images: ## Ensure container images are up to date with images.toml + registry lock
	@cargo run -q -p bijux-dna-dev -- containers run ensure-images

_containers-doctor: ## Run container doctor status report (missing images, lock drift, parity).
	@cargo run -q -p bijux-dna-dev -- containers run container-doctor

_containers-release-gate: ## Run mandatory container release gate checks.
	@cargo run -q -p bijux-dna-dev -- containers run release-gate

_ghcr-apptainer-publish-matrix: ## Generate GHCR Apptainer publish matrix.
	@cargo run -q -p bijux-dna-dev -- containers run generate-ghcr-apptainer-publish-matrix -- $(GHCR_PUBLISH_ARGS)

_ghcr-docker-publish-matrix: ## Generate GHCR Docker publish matrix.
	@cargo run -q -p bijux-dna-dev -- containers run generate-ghcr-publish-matrix -- $(GHCR_PUBLISH_ARGS)

_ghcr-apptainer-build-one: ## Build one Apptainer image for GHCR publication.
	@cargo run -q -p bijux-dna-dev -- containers run build-apptainer-all -- --build-one "$(TOOL_ID)"

_fastq-container-readiness: ## Generate FASTQ container readiness evidence reports.
	@PYTHONDONTWRITEBYTECODE=1 python3 makes/bin/generate_fastq_container_readiness.py
	@git diff --exit-code -- science/docs/upstream/fastq/container

_containers: ## Print tools/runtime/result/log summary from target-containers manifests
	@MANIFEST_DIR="$(CONTAINER_ARTIFACT_DIR)" cargo run -q -p bijux-dna-dev -- containers run summary

.PHONY: _container-runtime-check _env-prep _env-smoke _container-smoke _containers-smoke \
	_smoke-containers-docker-arm64 _smoke-containers-docker-amd64 _smoke-containers-apptainer \
	_smoke-containers-apptainer-bijux-run _smoke-containers-apptainer-apptainer-run _smoke-containers-apptainer-verify \
	_build-images _test-images _test-images-stage _test-images-tool _image-smoke-vcf _image-qa \
	_containers-apptainer-build apptainer-build-all _containers-lint _containers-ensure-images _containers-doctor _containers-release-gate _fastq-container-readiness _containers \
	_ghcr-apptainer-publish-matrix _ghcr-docker-publish-matrix _ghcr-apptainer-build-one \
	docker-build-all \
	_apptainer-ensure _apptainer-ensure-stage
