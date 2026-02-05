SHELL := /bin/sh

.PHONY: help
help:
	@echo "Use tools/* scripts (e.g., tools/lint, tools/test) or cargo make at the workspace root."
	@exit 1
