##@ Docs

MKDOCS ?= mkdocs
DOCS_ROOT ?= artifacts/docs
DOCS_SITE ?= $(DOCS_ROOT)/site
DOCS_VENV ?= $(DOCS_ROOT)/.venv
DOCS_PY ?= python3
DOCS_REQ ?= scripts/docs/requirements.txt

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
	@./bin/isolate sh -ceu './scripts/docs/check-domain-doc-references.sh; ./scripts/docs/check-doc-links.sh; ./scripts/docs/check-generated-docs.sh; ./scripts/docs/check-doc-assets.sh; DOCS_ROOT="$$ISO_ROOT/docs" $(MAKE) docs-lint; ./scripts/docs/check-root-pollution.sh'

.PHONY: docs docs-lint docs-serve docs-clean docs-isolate
