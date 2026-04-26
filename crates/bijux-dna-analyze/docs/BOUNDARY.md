# Boundary

## Purpose
`bijux-dna-analyze` turns completed run and benchmark artifacts into deterministic reports,
summaries, rankings, and contract diagnostics.

## Allowed Inputs
- Runtime facts: `facts.jsonl` or, behind the `parquet` feature, `facts.parquet`
- Run index and run summary records produced by runtime-owned code
- Stage reports, tool invocation records, benchmark reports, manifests, and fixtures
- Pipeline defaults ledgers used for report provenance

## Allowed Outputs
- `analysis.json`, `compare.json`, `ranking.json`, and `decision_trace.json`
- `report.json`, `report_bundle/index.html`, optional `report.html`, and optional `report.md`
- Dashboard facts, stage summaries, and run summaries written by `src/exports/`

## Forbidden Ownership
- Workflow planning and tool selection
- Runtime layout policy
- Tool execution, process spawning, container execution, or network access
- Mutation of generated runtime inputs
- CLI argument parsing or user-interface command routing

## Dependency Shape
- May depend on core contracts, runtime artifact schemas, domain stage identifiers, pipeline
  defaults, and infra file writers needed to read produced artifacts and write declared reports.
- Must not depend on runner or engine internals.
- Must not import benchmark execution code from `src/`; benchmark crates are allowed only in tests
  that prove report compatibility with produced benchmark artifacts.

## Validation
Use `docs/COMMANDS.md` as the single source of truth for crate commands. The minimum boundary
check is:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-analyze --test boundaries --no-default-features
```
