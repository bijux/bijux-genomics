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
	@./bin/isolate env TOOLS="$(TOOLS)" BIJUX_WORKERS="$(BIJUX_WORKERS)" JOBS="$(BIJUX_WORKERS)" ARTIFACT_DIR="$(CONTAINER_ARTIFACT_DIR)" sh scripts/containers/smoke-docker-arm64.sh

smoke-containers-docker-amd64: ## Build+smoke Docker amd64 containers
	@./bin/isolate env TOOLS="$(TOOLS)" BIJUX_WORKERS="$(BIJUX_WORKERS)" JOBS="$(BIJUX_WORKERS)" ARTIFACT_DIR="$(CONTAINER_ARTIFACT_DIR)" sh scripts/containers/smoke-docker-amd64.sh

smoke-containers-apptainer: ## Build+smoke Apptainer containers
	@./bin/isolate env TOOLS="$(TOOLS)" BIJUX_WORKERS="$(BIJUX_WORKERS)" JOBS="$(BIJUX_WORKERS)" ARTIFACT_DIR="$(CONTAINER_ARTIFACT_DIR)" sh scripts/containers/smoke-apptainer.sh

smoke-cntainers-apptainer-bijux-run: ## Apptainer smoke in bijux-run mode (registry commands via exec).
	@./bin/isolate env TOOLS="$(TOOLS)" BIJUX_WORKERS="$(BIJUX_WORKERS)" JOBS="$(BIJUX_WORKERS)" SMOKE_RUN_MODE="bijux-run" SMOKE_LEVEL="contract" ARTIFACT_DIR="$(CONTAINER_ARTIFACT_DIR)/apptainer-bijux-run" sh scripts/containers/smoke-apptainer.sh

smoke-cntainers-apptainer-apptainer-run: ## Apptainer smoke in runscript mode (apptainer run).
	@./bin/isolate env TOOLS="$(TOOLS)" BIJUX_WORKERS="$(BIJUX_WORKERS)" JOBS="$(BIJUX_WORKERS)" SMOKE_RUN_MODE="apptainer-run" SMOKE_LEVEL="contract" ARTIFACT_DIR="$(CONTAINER_ARTIFACT_DIR)/apptainer-apptainer-run" sh scripts/containers/smoke-apptainer.sh

smoke-cntainers-apptainer-verify: smoke-cntainers-apptainer-bijux-run smoke-cntainers-apptainer-apptainer-run ## Compare bijux-run vs apptainer-run smoke statuses.
	@python3 - <<'PY'
		import json
		from pathlib import Path

		root = Path("artifacts/container")
		a = root / "apptainer-bijux-run"
		b = root / "apptainer-apptainer-run"
		if not a.exists() or not b.exists():
		    raise SystemExit("missing smoke artifact dirs for compare")

		def load_statuses(base: Path) -> dict[str, str]:
		    statuses = {}
		    for mf in sorted(base.glob("*.json")):
		        if mf.name in {"report.json", "summary.json"}:
		            continue
		        payload = json.loads(mf.read_text())
		        tool = payload.get("tool")
		        status = payload.get("status")
		        if tool:
		            statuses[tool] = status
		    return statuses

		left = load_statuses(a)
		right = load_statuses(b)
		missing_left = sorted(set(right) - set(left))
		missing_right = sorted(set(left) - set(right))
		mismatch = sorted(t for t in set(left) & set(right) if left[t] != right[t])
		if missing_left or missing_right or mismatch:
		    print("smoke mode mismatch detected")
		    if missing_left:
		        print("missing in bijux-run:", ",".join(missing_left))
		    if missing_right:
		        print("missing in apptainer-run:", ",".join(missing_right))
		    if mismatch:
		        print("status mismatch:", ",".join(f"{t}:{left[t]}!={right[t]}" for t in mismatch))
		    raise SystemExit(1)
		print(f"smoke mode compare OK for {len(left)} tools")
		PY

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
	./bin/isolate env TOOLS="$$TOOLS_VAL" BIJUX_WORKERS="$(BIJUX_WORKERS)" JOBS="$(BIJUX_WORKERS)" SMOKE_LEVEL="build" SAVE_TAR="0" ARTIFACT_DIR="$(CONTAINER_ARTIFACT_DIR)" sh scripts/containers/smoke-docker-arm64.sh

test-images: ## Smoke selected runtime (registry-driven via scripts/CLI)
	@if [ "$(CONTAINER_TYPE)" = "docker-arm64" ]; then \
		if [ -n "$(STAGE)" ]; then \
			TOOLS="$$( $(BIJUX_BIN) registry list-tools --stage "$(STAGE)" --kind all | paste -sd, - )"; \
			./bin/isolate env TOOLS="$$TOOLS" BIJUX_WORKERS="$(BIJUX_WORKERS)" JOBS="$(BIJUX_WORKERS)" SMOKE_LEVEL="contract" SAVE_TAR="0" ARTIFACT_DIR="$(CONTAINER_ARTIFACT_DIR)" sh scripts/containers/smoke-docker-arm64.sh; \
		else \
			TOOLS_VAL="$(TOOLS)"; \
			if [ -z "$$TOOLS_VAL" ]; then \
				TOOLS_VAL="$$( $(BIJUX_BIN) registry list-tools --kind primary | paste -sd, - )"; \
			fi; \
			./bin/isolate env TOOLS="$$TOOLS_VAL" BIJUX_WORKERS="$(BIJUX_WORKERS)" JOBS="$(BIJUX_WORKERS)" SMOKE_LEVEL="contract" SAVE_TAR="0" ARTIFACT_DIR="$(CONTAINER_ARTIFACT_DIR)" sh scripts/containers/smoke-docker-arm64.sh; \
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

image-smoke-vcf: ## Smoke only VCF tools and write manifests under isolate/container artifacts.
	@set -eu; \
	TOOLS_VCF="$$( stages="$$( $(BIJUX_BIN) registry list-stages | awk -F. '$$1==\"vcf\"{print $$0}' )"; \
		if [ -z "$$stages" ]; then \
			echo ""; \
		else \
			for stage in $$stages; do \
				$(BIJUX_BIN) registry list-tools --stage "$$stage" --kind all; \
			done | tr ',' '\n' | sed '/^$$/d' | sort -u | paste -sd, -; \
		fi )"; \
	if [ -z "$$TOOLS_VCF" ]; then \
		echo "ERROR: no VCF tools found via registry stage/tool mapping"; \
		exit 2; \
	fi; \
	if [ "$(CONTAINER_TYPE)" = "apptainer" ]; then \
		./bin/isolate env TOOLS="$$TOOLS_VCF" BIJUX_WORKERS="$(BIJUX_WORKERS)" JOBS="$(BIJUX_WORKERS)" ARTIFACT_DIR="$(CONTAINER_ARTIFACT_DIR)" sh scripts/containers/smoke-apptainer.sh; \
	else \
		./bin/isolate env TOOLS="$$TOOLS_VCF" BIJUX_WORKERS="$(BIJUX_WORKERS)" JOBS="$(BIJUX_WORKERS)" SMOKE_LEVEL="contract" SAVE_TAR="0" ARTIFACT_DIR="$(CONTAINER_ARTIFACT_DIR)" sh scripts/containers/smoke-docker-arm64.sh; \
	fi

image-qa: ## Run image QA (docker-arm64 only)
	@if [ "$(CONTAINER_TYPE)" != "docker-arm64" ]; then \
		echo "skip: image-qa is docker-only (CONTAINER_TYPE=$(CONTAINER_TYPE))"; \
		exit 0; \
	fi
	./bin/isolate cargo run --bin image_qa -- --platform $(PLATFORM)

containers-apptainer-build: ## Batch-build Apptainer defs to VM-local output and copy back artifacts
	@BIJUX_WORKERS="$(BIJUX_WORKERS)" JOBS="$(BIJUX_WORKERS)" ./scripts/containers/apptainer_build_all.sh \
		--defs-dir containers/apptainer \
		--vm-out "$(APPTAINER_VM_OUT)" \
		--copy-back "$(APPTAINER_COPY_BACK)"

apptainer-ensure: ## Ensure apptainer images from SSOT stage list. Use DOMAIN=<domain> STAGES=<s1,s2>
	@if [ -z "$(DOMAIN)" ] || [ -z "$(STAGES)" ]; then \
		echo "ERROR: set DOMAIN=<domain> and STAGES=<comma-separated>"; \
		echo "example: make apptainer-ensure DOMAIN=fastq STAGES=validate_pre,trim,filter,stats,qc_post"; \
		exit 2; \
	fi
	@BIJUX_HPC_ROOT="$(BIJUX_HPC_ROOT)" $(BIJUX_BIN) env ensure-images --domain "$(DOMAIN)" --stages "$(STAGES)"

APPTAINER_STAGE_TARGETS := $(shell $(BIJUX_BIN) registry list-stages | awk -F. 'NF==2{printf "make-apptainer-%s-%s ", $$1, $$2}')

define APPTAINER_STAGE_TARGET_template
$(1): ## Ensure apptainer image(s) for stage $(2).$(3)
	@BIJUX_HPC_ROOT="$(BIJUX_HPC_ROOT)" $(BIJUX_BIN) env ensure-images --domain "$(2)" --stages "$(3)"
endef

$(foreach target,$(APPTAINER_STAGE_TARGETS),$(eval $(call APPTAINER_STAGE_TARGET_template,$(target),$(word 3,$(subst -, ,$(target))),$(word 4,$(subst -, ,$(target))))))

containers-lint: ## Lint container naming, headers, labels, and forbidden patterns
	@./scripts/containers/lint.sh

containers: ## Print tools/runtime/result/log summary from target-containers manifests
	@MANIFEST_DIR="$(CONTAINER_ARTIFACT_DIR)" ./scripts/containers/summary.sh

.PHONY: container-runtime-check env-prep env-smoke container-smoke containers-smoke \
	smoke-containers-docker-arm64 smoke-containers-docker-amd64 smoke-containers-apptainer \
	smoke-cntainers-apptainer-bijux-run smoke-cntainers-apptainer-apptainer-run smoke-cntainers-apptainer-verify \
	build-images test-images test-images-stage test-images-tool image-smoke-vcf image-qa \
	containers-apptainer-build containers-lint containers \
	apptainer-ensure $(APPTAINER_STAGE_TARGETS)
