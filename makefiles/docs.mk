##@ Docs

MKDOCS ?= mkdocs
DOCS_ROOT ?= artifacts/docs
DOCS_SITE ?= $(DOCS_ROOT)/site
DOCS_VENV ?= $(DOCS_ROOT)/.venv
DOCS_PY ?= python3
DOCS_REQ ?= configs/docs/requirements.txt

$(DOCS_VENV)/bin/activate: $(DOCS_REQ)
	@DOCS_PY="$(DOCS_PY)" DOCS_VENV="$(DOCS_VENV)" DOCS_REQ="$(DOCS_REQ)" ./scripts/run.sh tooling setup-docs-venv

_docs: $(DOCS_VENV)/bin/activate ## Build docs locally (non-strict)
	$(DOCS_VENV)/bin/mkdocs build --site-dir $(DOCS_SITE)

_docs-lint: $(DOCS_VENV)/bin/activate ## Build docs in strict mode
	$(DOCS_VENV)/bin/mkdocs build --strict --site-dir $(DOCS_SITE)

_docs-serve: $(DOCS_VENV)/bin/activate ## Serve docs locally
	$(DOCS_VENV)/bin/mkdocs serve

_docs-clean: ## Remove built docs
	@./scripts/run.sh tooling clean-docs "$(DOCS_ROOT)"

_docs-isolate: ## Build docs in strict mode under an isolate dir
	@./bin/isolate sh -ceu './scripts/run.sh docs check-domain-doc-references; ./scripts/run.sh docs check-doc-links; ./scripts/run.sh docs check-docs-graph; ./scripts/run.sh docs check-doc-root-layout; ./scripts/run.sh docs check-doc-depth; ./scripts/run.sh docs check-no-placeholder-language; ./scripts/run.sh docs check-generated-docs; ./scripts/run.sh docs check-doc-assets; DOCS_ROOT="$$ISO_ROOT/docs" $(MAKE) _docs-lint; ./scripts/run.sh docs check-root-pollution'

.PHONY: _docs _docs-lint _docs-serve _docs-clean _docs-isolate
