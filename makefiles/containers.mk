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
APPTAINER_COPY_BACK ?= $(if $(ISOLATE_ROOT),$(ISOLATE_ROOT)/container/apptainer,artifacts/container/apptainer)
CONTAINER_ARTIFACT_DIR ?= $(if $(ISOLATE_ROOT),$(ISOLATE_ROOT)/container,artifacts/container)
BIJUX_BIN ?= ./bin/isolate cargo run --bin bijux -- dna
BIJUX_HPC_ROOT ?= $(HOME)/bijux

CT_KEY := $(subst -,_,$(CONTAINER_TYPE))
SMOKE_SCRIPT_docker_arm64 := scripts/containers/smoke-docker-arm64.sh
SMOKE_SCRIPT_docker_amd64 := scripts/containers/smoke-docker-amd64.sh
SMOKE_SCRIPT_apptainer := scripts/containers/smoke-apptainer.sh
SMOKE_SCRIPT := $(SMOKE_SCRIPT_$(CT_KEY))

container-runtime-check: ## Validate selected container runtime
	@SYSTEM_TYPE="$(SYSTEM_TYPE)" CONTAINER_TYPE="$(CONTAINER_TYPE)" ./scripts/run.sh containers make container-runtime-check

env-prep: ## Prepare environment images via CLI (TOOL=<id> or STAGE=<domain.stage|stage>)
	@CONTAINER_TYPE="$(CONTAINER_TYPE)" TOOLS="$(TOOLS)" STAGE="$(STAGE)" BIJUX_BIN="$(BIJUX_BIN)" ./scripts/run.sh containers make env-prep

env-smoke: ## Smoke environment via CLI (TOOL=<id> or STAGE=<domain.stage|stage>)
	@CONTAINER_TYPE="$(CONTAINER_TYPE)" TOOLS="$(TOOLS)" STAGE="$(STAGE)" BIJUX_BIN="$(BIJUX_BIN)" ./scripts/run.sh containers make env-smoke

container-smoke: container-runtime-check ## Prepare+smoke selected tool/stage via CLI
	@CONTAINER_TYPE="$(CONTAINER_TYPE)" TOOLS="$(TOOLS)" STAGE="$(STAGE)" BIJUX_BIN="$(BIJUX_BIN)" ./scripts/run.sh containers make container-smoke

containers-smoke: container-runtime-check ## Prepare+smoke every registered stage via CLI
	@CONTAINER_TYPE="$(CONTAINER_TYPE)" BIJUX_BIN="$(BIJUX_BIN)" ./scripts/run.sh containers make containers-smoke

smoke-containers-docker-arm64: ## Build+smoke Docker arm64 containers
	@CONTAINER_TYPE="$(CONTAINER_TYPE)" TOOLS="$(TOOLS)" BIJUX_WORKERS="$(BIJUX_WORKERS)" CONTAINER_ARTIFACT_DIR="$(CONTAINER_ARTIFACT_DIR)" ./scripts/run.sh containers make smoke-containers-docker-arm64

smoke-containers-docker-amd64: ## Build+smoke Docker amd64 containers
	@CONTAINER_TYPE="$(CONTAINER_TYPE)" TOOLS="$(TOOLS)" BIJUX_WORKERS="$(BIJUX_WORKERS)" CONTAINER_ARTIFACT_DIR="$(CONTAINER_ARTIFACT_DIR)" ./scripts/run.sh containers make smoke-containers-docker-amd64

smoke-containers-apptainer: ## Build+smoke Apptainer containers
	@CONTAINER_TYPE="$(CONTAINER_TYPE)" TOOLS="$(TOOLS)" BIJUX_WORKERS="$(BIJUX_WORKERS)" CONTAINER_ARTIFACT_DIR="$(CONTAINER_ARTIFACT_DIR)" ./scripts/run.sh containers make smoke-containers-apptainer

smoke-cntainers-apptainer-bijux-run: ## Apptainer smoke in bijux-run mode (registry commands via exec).
	@TOOLS="$(TOOLS)" BIJUX_WORKERS="$(BIJUX_WORKERS)" CONTAINER_ARTIFACT_DIR="$(CONTAINER_ARTIFACT_DIR)" ./scripts/run.sh containers make smoke-cntainers-apptainer-bijux-run

smoke-cntainers-apptainer-apptainer-run: ## Apptainer smoke in runscript mode (apptainer run).
	@TOOLS="$(TOOLS)" BIJUX_WORKERS="$(BIJUX_WORKERS)" CONTAINER_ARTIFACT_DIR="$(CONTAINER_ARTIFACT_DIR)" ./scripts/run.sh containers make smoke-cntainers-apptainer-apptainer-run

smoke-cntainers-apptainer-verify: smoke-cntainers-apptainer-bijux-run smoke-cntainers-apptainer-apptainer-run ## Compare bijux-run vs apptainer-run smoke statuses.
	@CONTAINER_ARTIFACT_DIR="$(CONTAINER_ARTIFACT_DIR)" ./scripts/run.sh containers make smoke-cntainers-apptainer-verify

build-images: ## Build Docker images (docker-arm64 only)
	@CONTAINER_TYPE="$(CONTAINER_TYPE)" TOOLS="$(TOOLS)" BIJUX_WORKERS="$(BIJUX_WORKERS)" BIJUX_BIN="$(BIJUX_BIN)" CONTAINER_ARTIFACT_DIR="$(CONTAINER_ARTIFACT_DIR)" ./scripts/run.sh containers make build-images

test-images: ## Smoke selected runtime (registry-driven via scripts/CLI)
	@CONTAINER_TYPE="$(CONTAINER_TYPE)" TOOLS="$(TOOLS)" STAGE="$(STAGE)" BIJUX_WORKERS="$(BIJUX_WORKERS)" BIJUX_BIN="$(BIJUX_BIN)" CONTAINER_ARTIFACT_DIR="$(CONTAINER_ARTIFACT_DIR)" ./scripts/run.sh containers make test-images

# Generic stage smoke: make test-images-stage STAGE=fastq.trim (or STAGE=trim)
test-images-stage: ## Smoke all tools for one stage via CLI registry
	@CONTAINER_TYPE="$(CONTAINER_TYPE)" STAGE="$(STAGE)" BIJUX_BIN="$(BIJUX_BIN)" ./scripts/run.sh containers make test-images-stage

# Single tool smoke: make test-images-tool TOOLS=<tool_id>
test-images-tool: ## Smoke one tool via CLI registry
	@CONTAINER_TYPE="$(CONTAINER_TYPE)" TOOLS="$(TOOLS)" BIJUX_BIN="$(BIJUX_BIN)" ./scripts/run.sh containers make test-images-tool

image-smoke-vcf: ## Smoke only VCF tools and write manifests under isolate/container artifacts.
	@CONTAINER_TYPE="$(CONTAINER_TYPE)" BIJUX_WORKERS="$(BIJUX_WORKERS)" BIJUX_BIN="$(BIJUX_BIN)" CONTAINER_ARTIFACT_DIR="$(CONTAINER_ARTIFACT_DIR)" ./scripts/run.sh containers make image-smoke-vcf

image-qa: ## Run image QA (docker-arm64 only)
	@CONTAINER_TYPE="$(CONTAINER_TYPE)" PLATFORM="$(PLATFORM)" ./scripts/run.sh containers make image-qa

containers-apptainer-build: ## Batch-build Apptainer defs to VM-local output and copy back artifacts
	@BIJUX_WORKERS="$(BIJUX_WORKERS)" JOBS="$(BIJUX_WORKERS)" ./scripts/run.sh containers build-apptainer-all \
		--defs-dir containers/apptainer \
		--vm-out "$(APPTAINER_VM_OUT)" \
		--copy-back "$(APPTAINER_COPY_BACK)"

apptainer-ensure: ## Ensure apptainer images from SSOT stage list. Use DOMAIN=<domain> STAGES=<s1,s2>
	@DOMAIN="$(DOMAIN)" STAGES="$(STAGES)" BIJUX_HPC_ROOT="$(BIJUX_HPC_ROOT)" BIJUX_BIN="$(BIJUX_BIN)" ./scripts/run.sh containers make apptainer-ensure

apptainer-ensure-stage: ## Ensure apptainer image(s) for one stage via DOMAIN=<domain> STAGES=<stage>
	@DOMAIN="$(DOMAIN)" STAGES="$(STAGES)" BIJUX_HPC_ROOT="$(BIJUX_HPC_ROOT)" BIJUX_BIN="$(BIJUX_BIN)" ./scripts/run.sh containers make apptainer-ensure-stage

containers-lint: ## Lint container naming, headers, labels, and forbidden patterns
	@./scripts/run.sh containers lint

containers-ensure-images: ## Ensure container images are up to date with images.toml + registry lock
	@./scripts/run.sh containers ensure-images

containers: ## Print tools/runtime/result/log summary from target-containers manifests
	@MANIFEST_DIR="$(CONTAINER_ARTIFACT_DIR)" ./scripts/run.sh containers summary

.PHONY: container-runtime-check env-prep env-smoke container-smoke containers-smoke \
	smoke-containers-docker-arm64 smoke-containers-docker-amd64 smoke-containers-apptainer \
	smoke-cntainers-apptainer-bijux-run smoke-cntainers-apptainer-apptainer-run smoke-cntainers-apptainer-verify \
	build-images test-images test-images-stage test-images-tool image-smoke-vcf image-qa \
	containers-apptainer-build containers-lint containers-ensure-images containers \
	apptainer-ensure apptainer-ensure-stage
