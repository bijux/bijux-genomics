# Run Artifacts

A run produces:
- `run_manifest.json` (truth source)
- `graph.json` (execution graph)
- `tool_invocation.json` per step
- `execution_record.json` per step
- `effective_config.json` per step
- per‑step metrics envelopes
- aggregated reports (`report.json`, `summary.json`, `report.html`)

## Layout
- `run_artifacts/` contains run‑level artifacts and telemetry.
- Each step has its own `run_artifacts/` folder under its tool output directory.

## Consumers
- Analyze loads manifests + metrics + reports to build summaries.
- Benchmark consumes analyze output for comparisons.
