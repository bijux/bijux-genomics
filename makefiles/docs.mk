##@ Docs

MKDOCS ?= mkdocs
DOCS_ROOT ?= artifacts/docs
DOCS_SITE ?= $(DOCS_ROOT)/site
DOCS_VENV ?= $(DOCS_ROOT)/.venv
DOCS_PY ?= python3
DOCS_REQ ?= requirements-docs.txt

$(DOCS_VENV)/bin/activate: $(DOCS_REQ)
	$(DOCS_PY) -m venv $(DOCS_VENV)
	$(DOCS_VENV)/bin/pip install --upgrade pip
	$(DOCS_VENV)/bin/pip install -r $(DOCS_REQ)

docs: $(DOCS_VENV)/bin/activate ## Build docs locally (non-strict)
	$(DOCS_VENV)/bin/mkdocs build --site-dir $(DOCS_SITE)

docs-lint: $(DOCS_VENV)/bin/activate ## Build docs in strict mode
	$(DOCS_VENV)/bin/mkdocs build --strict --site-dir $(DOCS_SITE)

docs-serve: $(DOCS_VENV)/bin/activate ## Serve docs locally
	$(DOCS_VENV)/bin/mkdocs serve

docs-clean: ## Remove built docs
	rm -rf $(DOCS_ROOT)

docs-isolate: ## Build docs in strict mode under an isolate dir
	@ISO=$$(date -u +%Y%m%d%H%M%S)-$$$$-$(shell git rev-parse --short HEAD 2>/dev/null || echo nogit); \
	ROOT=artifacts/isolates/$$ISO/docs; \
	DOCS_ROOT=$$ROOT $(MAKE) docs-lint

.PHONY: docs docs-lint docs-serve docs-clean docs-isolate
