##@ Python contracts

PYTHON ?= python3

.PHONY: _python-test ci-test-python

ci-test-python: ## Run every declared Python CI lane with exact case receipts
	@$(ensure_artifact_env)
	@"$(PYTHON)" -B makes/bin/run_ci_python.py

_python-test: ## Run Python adapter and benchmark contracts with isolated artifacts
	@$(ensure_artifact_env)
	@mkdir -p "$(ARTIFACT_TMP_DIR)" "$(PYTEST_DEBUG_TEMPROOT)"
	@TMPDIR="$(ARTIFACT_TMP_DIR)" TMP="$(ARTIFACT_TMP_DIR)" TEMP="$(ARTIFACT_TMP_DIR)" "$(PYTHON)" -m pytest
