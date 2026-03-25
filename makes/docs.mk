##@ Docs

MKDOCS ?= mkdocs
DOCS_ROOT ?= artifacts/docs
DOCS_SITE ?= $(DOCS_ROOT)/site
DOCS_VENV ?= $(DOCS_ROOT)/.venv
DOCS_PY ?= python3
DOCS_REQ ?= configs/docs/requirements.txt
DOCS_CFG ?= configs/docs/mkdocs.toml

$(DOCS_VENV)/bin/activate: $(DOCS_REQ)
	@DOCS_PY="$(DOCS_PY)" DOCS_VENV="$(DOCS_VENV)" DOCS_REQ="$(DOCS_REQ)" cargo run -q -p bijux-dna-dev -- tooling run setup-docs-venv

_docs: $(DOCS_VENV)/bin/activate ## Build docs locally (non-strict)
	@DOCS_VENV="$(DOCS_VENV)" DOCS_CFG="$(DOCS_CFG)" cargo run -q -p bijux-dna-dev -- tooling run docs-build build

_docs-lint: $(DOCS_VENV)/bin/activate ## Build docs in strict mode
	@DOCS_VENV="$(DOCS_VENV)" DOCS_CFG="$(DOCS_CFG)" cargo run -q -p bijux-dna-dev -- tooling run docs-build lint

_docs-serve: $(DOCS_VENV)/bin/activate ## Serve docs locally
	@DOCS_VENV="$(DOCS_VENV)" DOCS_CFG="$(DOCS_CFG)" cargo run -q -p bijux-dna-dev -- tooling run docs-build serve

_docs-clean: ## Remove built docs
	@cargo run -q -p bijux-dna-dev -- tooling run clean-docs "$(DOCS_ROOT)"

_docs-contract: ## Build docs in strict mode under the shared artifacts contract
	@$(ensure_artifact_env)
	@cargo run -q -p bijux-dna-dev -- docs run check-domain-doc-references
	@cargo run -q -p bijux-dna-dev -- docs run check-doc-links
	@cargo run -q -p bijux-dna-dev -- docs run check-docs-graph
	@cargo run -q -p bijux-dna-dev -- docs run check-doc-root-layout
	@cargo run -q -p bijux-dna-dev -- docs run check-doc-depth
	@cargo run -q -p bijux-dna-dev -- docs run check-no-placeholder-language
	@cargo run -q -p bijux-dna-dev -- docs run check-generated-docs
	@cargo run -q -p bijux-dna-dev -- docs run check-doc-assets
	@DOCS_ROOT="$(ARTIFACT_ROOT)/docs" $(MAKE) _docs-lint
	@cargo run -q -p bijux-dna-dev -- docs run check-root-pollution

.PHONY: _docs _docs-lint _docs-serve _docs-clean _docs-contract
