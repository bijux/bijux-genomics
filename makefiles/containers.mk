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
JOBS ?= 1
TOOLS ?=
STAGE ?=
APPTAINER_VM_OUT ?= $(HOME)/apptainer-build
APPTAINER_COPY_BACK ?= $(if $(ISOLATE_ROOT),$(ISOLATE_ROOT)/container/apptainer,artifacts/container/apptainer)
CONTAINER_ARTIFACT_DIR ?= $(if $(ISOLATE_ROOT),$(ISOLATE_ROOT)/container,artifacts/container)
BIJUX_BIN ?= ./bin/isolate cargo run --bin bijux-dna --

CT_KEY := $(subst -,_,$(CONTAINER_TYPE))
SMOKE_SCRIPT_docker_arm64 := scripts/smoke-containers-docker-arm64.sh
SMOKE_SCRIPT_docker_amd64 := scripts/smoke-containers-docker-amd64.sh
SMOKE_SCRIPT_apptainer := scripts/smoke-containers-apptainer.sh
SMOKE_SCRIPT := $(SMOKE_SCRIPT_$(CT_KEY))

container-runtime-check: ## Validate selected container runtime
	@if [ -z "$(SMOKE_SCRIPT)" ]; then \
		echo "ERROR: unsupported CONTAINER_TYPE=$(CONTAINER_TYPE)"; \
		echo "supported: docker-arm64 | docker-amd64 | apptainer"; \
		exit 2; \
	fi
	@echo "SYSTEM_TYPE=$(SYSTEM_TYPE) CONTAINER_TYPE=$(CONTAINER_TYPE)"

env-prep: ## Prepare environment images via CLI (TOOL=<id> or STAGE=<domain.stage|stage>)
	@if [ -z "$(TOOLS)" ] && [ -z "$(STAGE)" ]; then \
		echo "ERROR: set TOOLS=<tool_id> or STAGE=<stage>"; \
		exit 2; \
	fi
	@if [ -n "$(STAGE)" ]; then \
		$(BIJUX_BIN) environment prep $(CONTAINER_TYPE) --stage $(STAGE); \
	else \
		$(BIJUX_BIN) environment prep $(CONTAINER_TYPE) $(TOOLS); \
	fi

env-smoke: ## Smoke environment via CLI (TOOL=<id> or STAGE=<domain.stage|stage>)
	@if [ -z "$(TOOLS)" ] && [ -z "$(STAGE)" ]; then \
		echo "ERROR: set TOOLS=<tool_id> or STAGE=<stage>"; \
		exit 2; \
	fi
	@if [ -n "$(STAGE)" ]; then \
		$(BIJUX_BIN) environment smoke $(CONTAINER_TYPE) --stage $(STAGE); \
	else \
		$(BIJUX_BIN) environment smoke $(CONTAINER_TYPE) $(TOOLS); \
	fi

container-smoke: container-runtime-check ## Prepare+smoke selected tool/stage via CLI
	@if [ -z "$(TOOLS)" ] && [ -z "$(STAGE)" ]; then \
		echo "ERROR: set TOOLS=<tool_id> or STAGE=<stage>"; \
		exit 2; \
	fi
	@if [ -n "$(STAGE)" ]; then \
		$(BIJUX_BIN) environment prep $(CONTAINER_TYPE) --stage $(STAGE); \
		$(BIJUX_BIN) environment smoke $(CONTAINER_TYPE) --stage $(STAGE); \
	else \
		$(BIJUX_BIN) environment prep $(CONTAINER_TYPE) $(TOOLS); \
		$(BIJUX_BIN) environment smoke $(CONTAINER_TYPE) $(TOOLS); \
	fi

containers-smoke: container-runtime-check ## Prepare+smoke every registered stage via CLI
	@set -e; \
	for stage in $$($(BIJUX_BIN) registry list-stages); do \
		echo "== stage $$stage"; \
		$(BIJUX_BIN) environment prep $(CONTAINER_TYPE) --stage "$$stage"; \
		$(BIJUX_BIN) environment smoke $(CONTAINER_TYPE) --stage "$$stage"; \
	done

smoke-containers-docker-arm64: ## Build+smoke Docker arm64 containers
	@./bin/isolate env TOOLS="$(TOOLS)" JOBS="$(JOBS)" ARTIFACT_DIR="$(CONTAINER_ARTIFACT_DIR)" sh scripts/smoke-containers-docker-arm64.sh

smoke-containers-docker-amd64: ## Build+smoke Docker amd64 containers
	@./bin/isolate env TOOLS="$(TOOLS)" JOBS="$(JOBS)" ARTIFACT_DIR="$(CONTAINER_ARTIFACT_DIR)" sh scripts/smoke-containers-docker-amd64.sh

smoke-containers-apptainer: ## Build+smoke Apptainer containers
	@./bin/isolate env TOOLS="$(TOOLS)" JOBS="$(JOBS)" ARTIFACT_DIR="$(CONTAINER_ARTIFACT_DIR)" sh scripts/smoke-containers-apptainer.sh

build-images: ## Build Docker images (docker-arm64 only)
	@if [ "$(CONTAINER_TYPE)" != "docker-arm64" ]; then \
		echo "skip: build-images is docker-only (CONTAINER_TYPE=$(CONTAINER_TYPE))"; \
		exit 0; \
	fi
	@set -e; \
	TOOLS_VAL="$(TOOLS)"; \
	if [ -z "$$TOOLS_VAL" ]; then \
		TOOLS_VAL="$$( $(BIJUX_BIN) registry list-tools --kind primary | paste -sd, - )"; \
	fi; \
	./bin/isolate env TOOLS="$$TOOLS_VAL" JOBS="$(JOBS)" SMOKE_LEVEL="build" SAVE_TAR="0" ARTIFACT_DIR="$(CONTAINER_ARTIFACT_DIR)" sh scripts/smoke-containers-docker-arm64.sh

test-images: ## Smoke selected runtime (registry-driven via scripts/CLI)
	@if [ "$(CONTAINER_TYPE)" = "docker-arm64" ]; then \
		if [ -n "$(STAGE)" ]; then \
			TOOLS="$$( $(BIJUX_BIN) registry list-tools --stage "$(STAGE)" --kind all | paste -sd, - )"; \
			./bin/isolate env TOOLS="$$TOOLS" JOBS="$(JOBS)" SMOKE_LEVEL="contract" SAVE_TAR="0" ARTIFACT_DIR="$(CONTAINER_ARTIFACT_DIR)" sh scripts/smoke-containers-docker-arm64.sh; \
		else \
			TOOLS_VAL="$(TOOLS)"; \
			if [ -z "$$TOOLS_VAL" ]; then \
				TOOLS_VAL="$$( $(BIJUX_BIN) registry list-tools --kind primary | paste -sd, - )"; \
			fi; \
			./bin/isolate env TOOLS="$$TOOLS_VAL" JOBS="$(JOBS)" SMOKE_LEVEL="contract" SAVE_TAR="0" ARTIFACT_DIR="$(CONTAINER_ARTIFACT_DIR)" sh scripts/smoke-containers-docker-arm64.sh; \
		fi; \
	elif [ -n "$(STAGE)" ]; then \
		$(MAKE) env-smoke STAGE="$(STAGE)" CONTAINER_TYPE="$(CONTAINER_TYPE)"; \
	elif [ -n "$(TOOLS)" ]; then \
		$(MAKE) env-smoke TOOLS="$(TOOLS)" CONTAINER_TYPE="$(CONTAINER_TYPE)"; \
	else \
		$(MAKE) containers-smoke CONTAINER_TYPE="$(CONTAINER_TYPE)"; \
	fi

# Generic stage smoke: make test-images-stage STAGE=fastq.trim (or STAGE=trim)
test-images-stage: ## Smoke all tools for one stage via CLI registry
	@if [ -z "$(STAGE)" ]; then \
		echo "ERROR: set STAGE=<domain.stage|stage> (example: STAGE=fastq.trim)"; \
		exit 2; \
	fi
	@$(MAKE) env-smoke STAGE="$(STAGE)" CONTAINER_TYPE="$(CONTAINER_TYPE)"

# Single tool smoke: make test-images-tool TOOLS=<tool_id>
test-images-tool: ## Smoke one tool via CLI registry
	@if [ -z "$(TOOLS)" ]; then \
		echo "ERROR: set TOOLS=<tool_id>"; \
		exit 2; \
	fi
	@$(MAKE) env-smoke TOOLS="$(TOOLS)" CONTAINER_TYPE="$(CONTAINER_TYPE)"

image-qa: ## Run image QA (docker-arm64 only)
	@if [ "$(CONTAINER_TYPE)" != "docker-arm64" ]; then \
		echo "skip: image-qa is docker-only (CONTAINER_TYPE=$(CONTAINER_TYPE))"; \
		exit 0; \
	fi
	./bin/isolate cargo run --bin image_qa -- --platform $(PLATFORM)

containers-apptainer-build: ## Batch-build Apptainer defs to VM-local output and copy back artifacts
	@JOBS="$(JOBS)" ./scripts/apptainer_build_all.sh \
		--defs-dir containers/apptainer \
		--vm-out "$(APPTAINER_VM_OUT)" \
		--copy-back "$(APPTAINER_COPY_BACK)"

containers-lint: ## Lint container naming, headers, labels, and forbidden patterns
	@./scripts/lint-containers.sh

containers: ## Print tools/runtime/result/log summary from target-containers manifests
	@MANIFEST_DIR="$(CONTAINER_ARTIFACT_DIR)" ./scripts/containers-summary.sh

.PHONY: container-runtime-check env-prep env-smoke container-smoke containers-smoke \
	smoke-containers-docker-arm64 smoke-containers-docker-amd64 smoke-containers-apptainer \
	build-images test-images test-images-stage test-images-tool image-qa \
	containers-apptainer-build containers-lint containers
