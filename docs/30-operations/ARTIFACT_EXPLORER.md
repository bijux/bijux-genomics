# Artifact Explorer

## What
Guide to run artifacts and how to diff runs.

## Why
Artifacts are the source of truth for reproducibility.

## Non-goals
- Tool-specific debugging.

## Contracts
Run artifacts are defined in [RUN_ARTIFACTS.md](RUN_ARTIFACTS.md), aligned to the exact
runtime handoff in [../10-architecture/DATAFLOW.md](../10-architecture/DATAFLOW.md), and the
report bundle contract in [REPORT_CONTRACT.md](REPORT_CONTRACT.md).

## Examples
Run layout:
- `run_manifest.json`
- `stage_<n>/tool_invocation.json`
- `stage_<n>/execution_record.json`
- `report.json`, `report.html`, `summary.tsv`

Diffing runs:
- compare `run_manifest.json` hashes
- compare `tool_invocation.json` and `effective_config.json`
- compare `report.json` fields

## Failure modes
Missing artifacts indicate contract violations.
