##@ Docs

MKDOCS ?= mkdocs
DOCS_ROOT ?= artifacts/docs
DOCS_SITE ?= $(DOCS_ROOT)/site
DOCS_VENV ?= $(DOCS_ROOT)/.venv
DOCS_PY ?= python3
DOCS_REQ ?= configs/docs/requirements.txt

$(DOCS_VENV)/bin/activate: $(DOCS_REQ)
	@DOCS_PY="$(DOCS_PY)" DOCS_VENV="$(DOCS_VENV)" DOCS_REQ="$(DOCS_REQ)" ./scripts/run.sh tooling setup-docs-venv

docs: $(DOCS_VENV)/bin/activate ## Build docs locally (non-strict)
	$(DOCS_VENV)/bin/mkdocs build --site-dir $(DOCS_SITE)

docs-lint: $(DOCS_VENV)/bin/activate ## Build docs in strict mode
	$(DOCS_VENV)/bin/mkdocs build --strict --site-dir $(DOCS_SITE)

docs-serve: $(DOCS_VENV)/bin/activate ## Serve docs locally
	$(DOCS_VENV)/bin/mkdocs serve

docs-clean: ## Remove built docs
	@./scripts/run.sh tooling clean-docs "$(DOCS_ROOT)"

docs-isolate: ## Build docs in strict mode under an isolate dir
	@./bin/isolate sh -ceu './scripts/run.sh docs check-domain-doc-references; ./scripts/run.sh docs check-doc-links; ./scripts/run.sh docs check-docs-graph; ./scripts/run.sh docs check-doc-root-layout; ./scripts/run.sh docs check-doc-depth; ./scripts/run.sh docs check-generated-docs; ./scripts/run.sh docs check-doc-assets; DOCS_ROOT="$$ISO_ROOT/docs" $(MAKE) docs-lint; ./scripts/run.sh docs check-root-pollution'

.PHONY: docs docs-lint docs-serve docs-clean docs-isolate
