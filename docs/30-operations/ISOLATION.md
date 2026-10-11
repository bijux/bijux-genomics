# Artifact Environment

## Purpose
Define the shared artifact environment contract for local development, CI, and operational automation.

## Scope
This document covers artifact, cargo, cache, and temporary output roots used by repository automation.

## Non-goals
- Defining scientific result schemas.
- Replacing per-command output contracts.

## Contract
- `ARTIFACT_ROOT` defaults to `artifacts/`.
- `ISO_ROOT` remains a compatibility alias for `ARTIFACT_ROOT`.
- Cargo build output lives under `artifacts/target/`.
- Cargo home lives under `artifacts/cargo/home/`.
- Temporary files live under `artifacts/tmp/`.
- Make exports `PYTHONPYCACHEPREFIX` and `RUFF_CACHE_DIR` beneath `artifacts/python/`.
- pytest discovers its cache directory from the root `pyproject.toml` and creates
  numbered temporary directories beneath `PYTEST_DEBUG_TEMPROOT` for Make runs.
- Deterministic defaults remain `TZ=UTC` and `LC_ALL=C`.

## Usage Rules
- Make targets must prepare the environment through `makes/_macro.mk`.
- Make targets and helper entrypoints must export the shared artifact environment before invoking `bijux-dna-dev`.
- Automation must write only under `artifacts/`.
- Automation must not hardcode retired `artifacts/isolates/` paths.

## Contracts
- Root build outputs must stay out of the repository top level.
- Automation entrypoints must route generated outputs through the governed artifact environment.

## Python contract checks

Use `make _python-test PYTHON=artifacts/python/venv/bin/python` with an interpreter
containing the `checks` dependency group from `pyproject.toml`. The target runs
the benchmark, corpus materialization, and container adapter Python contracts.
It sets Python temporary output to `artifacts/tmp/`, independently of a host's
ambient `TMPDIR`. An existing interpreter can be selected with `PYTHON`.

When invoking Python directly, set `PYTHONPYCACHEPREFIX` to an absolute directory
under `artifacts/python/` (or use `python3 -B`) and place `TMPDIR` under
`artifacts/tmp/`. The pytest and Ruff cache settings are discovered automatically;
they do not alter Python's interpreter-level bytecode or temporary-file defaults.
