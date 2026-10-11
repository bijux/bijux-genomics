# Publication uses an isolated admitted renderer and source-bound verification.
DOCS_PUBLICATION_SHARED := .bijux/shared/bijux-docs
DOCS_PUBLICATION_PYTHON := $(abspath artifacts/docs/publication/python/bin/python)
DOCS_PUBLICATION_SITE_URL := https://bijux.io/bijux-genomics/
DOCS_PUBLICATION_SITE_DIR := artifacts/docs/site
DOCS_PUBLICATION_BUILDER := $(DOCS_PUBLICATION_SHARED)/tooling/security/publication_builder.py

.PHONY: gh-docs-install gh-docs-build gh-docs-verify

gh-docs-install: ## Install the frozen documentation publication renderer
	@python3 "$(DOCS_PUBLICATION_SHARED)/tooling/security/install_publication_renderer.py" --recipe history-media --python python3

gh-docs-build: ## Build and qualify the exact production documentation artifact
	@bash -euo pipefail -c '\
	  test "$${DOCS_BUILD_SITE_URL:-$(DOCS_PUBLICATION_SITE_URL)}" = "$(DOCS_PUBLICATION_SITE_URL)" || { echo "Exact production documentation URL required" >&2; exit 1; }; \
	  test "$${DOCS_SITE_DIR:-$(DOCS_PUBLICATION_SITE_DIR)}" = "$(DOCS_PUBLICATION_SITE_DIR)" || { echo "Exact production documentation artifact required" >&2; exit 1; }; \
	  export SITE_URL="$(DOCS_PUBLICATION_SITE_URL)" PYTHONDONTWRITEBYTECODE=1; \
	  test -n "$${BIJUX_STD_ROOT:-}" || { echo "Accepted standards checkout required" >&2; exit 1; }; \
	  source "$(DOCS_PUBLICATION_SHARED)/tooling/scripts/docs_source_authority.sh"; \
	  verify_docs_authority "$(CURDIR)" "$(CURDIR)/$(DOCS_PUBLICATION_SHARED)"; \
	  "$(DOCS_PUBLICATION_PYTHON)" "$(DOCS_PUBLICATION_SHARED)/tooling/scripts/project_bijux_docs.py" "$(CURDIR)" "$(CURDIR)/$(DOCS_PUBLICATION_SHARED)" --check; \
	  SITE_URL="$(DOCS_PUBLICATION_SITE_URL)" "$(DOCS_PUBLICATION_PYTHON)" "$(DOCS_PUBLICATION_BUILDER)" capture --standard-root "$$BIJUX_STD_ROOT" --site-url "$(DOCS_PUBLICATION_SITE_URL)" --site-dir "$(DOCS_PUBLICATION_SITE_DIR)" --config mkdocs.yml --build-command "make gh-docs-build" --verify-command "make gh-docs-verify"; \
	  SITE_URL="$(DOCS_PUBLICATION_SITE_URL)" "$(DOCS_PUBLICATION_PYTHON)" "$(DOCS_PUBLICATION_BUILDER)" render --config mkdocs.yml --site-url "$(DOCS_PUBLICATION_SITE_URL)" --site-dir "$(DOCS_PUBLICATION_SITE_DIR)"; \
	  SITE_URL="$(DOCS_PUBLICATION_SITE_URL)" "$(DOCS_PUBLICATION_PYTHON)" "$(DOCS_PUBLICATION_BUILDER)" admit --site-dir "$(DOCS_PUBLICATION_SITE_DIR)" --site-url "$(DOCS_PUBLICATION_SITE_URL)";'

gh-docs-verify: ## Verify accepted source and artifact receipts without rebuilding
	@SITE_URL="$(DOCS_PUBLICATION_SITE_URL)" "$(DOCS_PUBLICATION_PYTHON)" "$(DOCS_PUBLICATION_BUILDER)" verify
