# Run Artifacts

## What
Defines the run artifact set produced by every execution.

## Why
Run artifacts are the single source of truth for reproducibility.

## Non-goals
- Tool‑specific raw logs beyond recorded artifacts.

## Contracts
- run_manifest.json, graph.json, tool_invocation.json, execution_record.json.

## Examples
- A dry‑run produces graph.json and run_manifest.json only.

## Failure modes
- Missing declared artifacts triggers ContractError.
