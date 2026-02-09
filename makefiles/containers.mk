##@ Container Management

# Container runtime selector:
#   docker-arm64 | docker-amd64 | apptainer
# System selector:
#   local | hpc
SYSTEM_TYPE ?= local
ifeq ($(SYSTEM_TYPE),hpc)
CONTAINER_TYPE ?= apptainer
else
CONTAINER_TYPE ?= docker-arm64
endif

# Optional: platform for docker QA binaries
PLATFORM ?=

# Optional pass-through knobs for smoke scripts
JOBS ?= 1
TOOLS ?=
APPTAINER_VM_OUT ?= $(HOME)/apptainer-build
APPTAINER_COPY_BACK ?= target-containers/apptainer

CT_KEY := $(subst -,_,$(CONTAINER_TYPE))
SMOKE_SCRIPT_docker_arm64 := scripts/smoke-containers-docker-arm64.sh
SMOKE_SCRIPT_docker_amd64 := scripts/smoke-containers-docker-amd64.sh
SMOKE_SCRIPT_apptainer := scripts/smoke-containers-apptainer.sh
SMOKE_SCRIPT := $(SMOKE_SCRIPT_$(CT_KEY))

# ---- Tool groups by stage/domain (derived from configs/tools.toml SSOT) ----
FASTQ_TOOLS_preprocess := $(shell ./scripts/registry-tools.sh stage-tools fastq.preprocess all)
FASTQ_TOOLS_prepare_reference := $(shell ./scripts/registry-tools.sh stage-tools fastq.prepare_reference all)
FASTQ_TOOLS_validate_pre := $(shell ./scripts/registry-tools.sh stage-tools fastq.validate_pre all)
FASTQ_TOOLS_detect_adapters := $(shell ./scripts/registry-tools.sh stage-tools fastq.detect_adapters all)
FASTQ_TOOLS_trim := $(shell ./scripts/registry-tools.sh stage-tools fastq.trim all)
FASTQ_TOOLS_filter := $(shell ./scripts/registry-tools.sh stage-tools fastq.filter all)
FASTQ_TOOLS_stats_neutral := $(shell ./scripts/registry-tools.sh stage-tools fastq.stats_neutral all)
FASTQ_TOOLS_qc_post := $(shell ./scripts/registry-tools.sh stage-tools fastq.qc_post all)
FASTQ_TOOLS_merge := $(shell ./scripts/registry-tools.sh stage-tools fastq.merge all)
FASTQ_TOOLS_correct := $(shell ./scripts/registry-tools.sh stage-tools fastq.correct all)
FASTQ_TOOLS_umi := $(shell ./scripts/registry-tools.sh stage-tools fastq.umi all)
FASTQ_TOOLS_screen := $(shell ./scripts/registry-tools.sh stage-tools fastq.screen all)

BAM_TOOLS_align := $(shell ./scripts/registry-tools.sh stage-tools bam.align all)
BAM_TOOLS_validate := $(shell ./scripts/registry-tools.sh stage-tools bam.validate all)
BAM_TOOLS_qc_pre := $(shell ./scripts/registry-tools.sh stage-tools bam.qc_pre all)
BAM_TOOLS_filter := $(shell ./scripts/registry-tools.sh stage-tools bam.filter all)
BAM_TOOLS_markdup := $(shell ./scripts/registry-tools.sh stage-tools bam.markdup all)
BAM_TOOLS_complexity := $(shell ./scripts/registry-tools.sh stage-tools bam.complexity all)
BAM_TOOLS_coverage := $(shell ./scripts/registry-tools.sh stage-tools bam.coverage all)
BAM_TOOLS_damage := $(shell ./scripts/registry-tools.sh stage-tools bam.damage all)
BAM_TOOLS_authenticity := $(shell ./scripts/registry-tools.sh stage-tools bam.authenticity all)
BAM_TOOLS_contamination := $(shell ./scripts/registry-tools.sh stage-tools bam.contamination all)
BAM_TOOLS_sex := $(shell ./scripts/registry-tools.sh stage-tools bam.sex all)
BAM_TOOLS_bias_mitigation := $(shell ./scripts/registry-tools.sh stage-tools bam.bias_mitigation all)
BAM_TOOLS_recalibration := $(shell ./scripts/registry-tools.sh stage-tools bam.recalibration all)
BAM_TOOLS_haplogroups := $(shell ./scripts/registry-tools.sh stage-tools bam.haplogroups all)
BAM_TOOLS_genotyping := $(shell ./scripts/registry-tools.sh stage-tools bam.genotyping all)
BAM_TOOLS_kinship := $(shell ./scripts/registry-tools.sh stage-tools bam.kinship all)

# ---- Core dispatch ----
container-runtime-check: ## Validate selected container runtime and script wiring
	@if [ -z "$(SMOKE_SCRIPT)" ]; then \
		echo "ERROR: unsupported CONTAINER_TYPE=$(CONTAINER_TYPE)"; \
		echo "       supported: docker-arm64 | docker-amd64 | apptainer"; \
		exit 2; \
	fi
	@echo "SYSTEM_TYPE=$(SYSTEM_TYPE) CONTAINER_TYPE=$(CONTAINER_TYPE)"
	@echo "smoke-script=$(SMOKE_SCRIPT)"

container-smoke: container-runtime-check ## Build+smoke selected runtime (optional TOOLS=tool1,tool2)
	@TOOLS="$(TOOLS)" JOBS="$(JOBS)" sh "$(SMOKE_SCRIPT)"

containers-smoke: container-runtime-check ## Contract smoke all tools (--version/--help/binary)
	@SMOKE_LEVEL=contract JOBS="$(JOBS)" sh "$(SMOKE_SCRIPT)"

smoke-containers-docker-arm64: ## Build+smoke Docker arm64 containers (target-containers/{logs,images})
	@TOOLS="$(TOOLS)" JOBS="$(JOBS)" sh scripts/smoke-containers-docker-arm64.sh

smoke-containers-docker-amd64: ## Build+smoke Docker amd64 containers (target-containers/{logs,images})
	@TOOLS="$(TOOLS)" JOBS="$(JOBS)" sh scripts/smoke-containers-docker-amd64.sh

smoke-containers-apptainer: ## Build+smoke Apptainer containers (target-containers/{logs,images})
	@TOOLS="$(TOOLS)" JOBS="$(JOBS)" sh scripts/smoke-containers-apptainer.sh

# ---- Docker-only QA/build paths (kept for local docker workflows) ----
build-images: ## Build Docker images (only when CONTAINER_TYPE=docker-arm64)
	@if [ "$(CONTAINER_TYPE)" != "docker-arm64" ]; then \
		echo "skip: build-images is docker-only (CONTAINER_TYPE=$(CONTAINER_TYPE))"; \
		exit 0; \
	fi
	cargo run --bin build_docker_images -- --platform $(PLATFORM)

test-images: ## Test Docker images (docker uses test_docker_images; apptainer uses smoke script)
	@if [ "$(CONTAINER_TYPE)" = "docker-arm64" ]; then \
		cargo run --bin test_docker_images -- --platform $(PLATFORM); \
	else \
		$(MAKE) container-smoke; \
	fi

image-qa: ## Run image QA (docker-only)
	@if [ "$(CONTAINER_TYPE)" != "docker-arm64" ]; then \
		echo "skip: image-qa is docker-only (CONTAINER_TYPE=$(CONTAINER_TYPE))"; \
		exit 0; \
	fi
	cargo run --bin image_qa -- --platform $(PLATFORM)

# Legacy aliases (docker-centric names retained for compatibility)
test-images-trim: ## Legacy alias: trimming tool images
	@$(MAKE) test-images-fastq-trim

test-images-validate: ## Legacy alias: validation tool images
	@$(MAKE) test-images-fastq-validate-pre

test-images-filter: ## Legacy alias: filtering tool images
	@$(MAKE) test-images-fastq-filter

test-images-merge: ## Legacy alias: merging tool images
	@$(MAKE) test-images-fastq-merge

# ---- Stage-specific test-images-* ----
define FASTQ_STAGE_TARGET
.PHONY: test-images-fastq-$(1)
test-images-fastq-$(1): ## FASTQ stage fastq.$(1)
	@TOOLS="$$(FASTQ_TOOLS_$(1))" $(MAKE) container-smoke
endef

$(eval $(call FASTQ_STAGE_TARGET,preprocess))
$(eval $(call FASTQ_STAGE_TARGET,prepare_reference))
$(eval $(call FASTQ_STAGE_TARGET,validate_pre))
$(eval $(call FASTQ_STAGE_TARGET,detect_adapters))
$(eval $(call FASTQ_STAGE_TARGET,trim))
$(eval $(call FASTQ_STAGE_TARGET,filter))
$(eval $(call FASTQ_STAGE_TARGET,stats_neutral))
$(eval $(call FASTQ_STAGE_TARGET,qc_post))
$(eval $(call FASTQ_STAGE_TARGET,merge))
$(eval $(call FASTQ_STAGE_TARGET,correct))
$(eval $(call FASTQ_STAGE_TARGET,umi))
$(eval $(call FASTQ_STAGE_TARGET,screen))

define BAM_STAGE_TARGET
.PHONY: test-images-bam-$(1)
test-images-bam-$(1): ## BAM stage bam.$(1)
	@TOOLS="$$(BAM_TOOLS_$(1))" $(MAKE) container-smoke
endef

$(eval $(call BAM_STAGE_TARGET,align))
$(eval $(call BAM_STAGE_TARGET,validate))
$(eval $(call BAM_STAGE_TARGET,qc_pre))
$(eval $(call BAM_STAGE_TARGET,filter))
$(eval $(call BAM_STAGE_TARGET,markdup))
$(eval $(call BAM_STAGE_TARGET,complexity))
$(eval $(call BAM_STAGE_TARGET,coverage))
$(eval $(call BAM_STAGE_TARGET,damage))
$(eval $(call BAM_STAGE_TARGET,authenticity))
$(eval $(call BAM_STAGE_TARGET,contamination))
$(eval $(call BAM_STAGE_TARGET,sex))
$(eval $(call BAM_STAGE_TARGET,bias_mitigation))
$(eval $(call BAM_STAGE_TARGET,recalibration))
$(eval $(call BAM_STAGE_TARGET,haplogroups))
$(eval $(call BAM_STAGE_TARGET,genotyping))
$(eval $(call BAM_STAGE_TARGET,kinship))

containers-smoke-fastq-all: ## Smoke all FASTQ stage tool sets via selected runtime
	@TOOLS="$(FASTQ_TOOLS_preprocess),$(FASTQ_TOOLS_prepare_reference),$(FASTQ_TOOLS_validate_pre),$(FASTQ_TOOLS_detect_adapters),$(FASTQ_TOOLS_trim),$(FASTQ_TOOLS_filter),$(FASTQ_TOOLS_stats_neutral),$(FASTQ_TOOLS_qc_post),$(FASTQ_TOOLS_merge),$(FASTQ_TOOLS_correct),$(FASTQ_TOOLS_umi),$(FASTQ_TOOLS_screen)" \
	$(MAKE) container-smoke

containers-smoke-bam-all: ## Smoke all BAM stage tool sets via selected runtime
	@TOOLS="$(BAM_TOOLS_align),$(BAM_TOOLS_validate),$(BAM_TOOLS_qc_pre),$(BAM_TOOLS_filter),$(BAM_TOOLS_markdup),$(BAM_TOOLS_complexity),$(BAM_TOOLS_coverage),$(BAM_TOOLS_damage),$(BAM_TOOLS_authenticity),$(BAM_TOOLS_contamination),$(BAM_TOOLS_sex),$(BAM_TOOLS_bias_mitigation),$(BAM_TOOLS_recalibration),$(BAM_TOOLS_haplogroups),$(BAM_TOOLS_genotyping),$(BAM_TOOLS_kinship)" \
	$(MAKE) container-smoke

containers-smoke-all: ## Smoke all registered tools via selected runtime
	@$(MAKE) containers-smoke

containers-apptainer-build: ## Batch-build Apptainer defs to VM-local output and copy back artifacts
	@JOBS="$(JOBS)" ./scripts/apptainer_build_all.sh \
		--defs-dir containers/apptainer \
		--vm-out "$(APPTAINER_VM_OUT)" \
		--copy-back "$(APPTAINER_COPY_BACK)"

containers-lint: ## Lint container naming, headers, labels, and forbidden patterns
	@./scripts/lint-containers.sh

containers: ## Print tools/runtime/result/log summary from target-containers manifests
	@./scripts/containers-summary.sh

.PHONY: container-runtime-check container-smoke \
	containers-smoke \
	smoke-containers-docker-arm64 smoke-containers-docker-amd64 smoke-containers-apptainer \
	build-images test-images image-qa test-images-trim test-images-validate test-images-filter test-images-merge \
	containers-smoke-fastq-all containers-smoke-bam-all containers-smoke-all \
	containers-apptainer-build containers-lint containers
