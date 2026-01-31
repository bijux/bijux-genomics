\# bijux-analyze contract (v1)

## Purpose
Define stable inputs, outputs, and compatibility rules for `bijux-analyze`.

## Inputs
- Facts: `facts.jsonl` with `schema_version = bijux.facts.v1` (JSONL of `FactsRowV1`).
- Run index: `run_index.jsonl` with `schema_version = 1` (JSONL of `RunIndexLine`).
- Run summary: `run_summary.json` with `schema_version = bijux.run_summary.v1`.

## Outputs
- `analysis.json` (summary + warnings + dataset pointers).
- `compare.json` (pairwise comparisons).
- `ranking.json` (ranked tools + explanations).
- `report.json` and optional `report.html`.

## Compatibility
- Backward compatible for minor additions (new fields must be optional).
- Breaking changes require a new schema version (e.g., `bijux.facts.v2`).
- Analyze must reject unknown schema versions with actionable errors.

## Stability guarantees
- Deterministic ordering for JSON outputs.
- Stable ranking tie-breakers.
- Errors include file path + line/field when possible.
