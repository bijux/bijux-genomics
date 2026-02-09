##@ Docs

MKDOCS ?= mkdocs
DOCS_SITE ?= target-docs/site
DOCS_VENV ?= target-docs/.venv
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
	rm -rf $(DOCS_SITE)
