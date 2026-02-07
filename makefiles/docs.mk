##@ Docs

docs: ## Build docs locally (non-strict)
	mkdocs build

docs-lint: ## Build docs in strict mode
	mkdocs build --strict
