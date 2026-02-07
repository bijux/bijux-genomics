# Artifacts Contract

## What
Defines required artifact files per run and per step.

## Why
Ensures reproducibility and consistent reporting.

## Non-goals
- Tool‑specific log formats.

## Contracts
- run_manifest.json, tool_invocation.json, execution_record.json.

## Examples
- Each step writes `run_artifacts/` containing metrics and records.

## Failure modes
- Missing artifacts fail contract enforcement.
