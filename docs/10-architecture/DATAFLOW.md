# Dataflow Narrative

## What
Step-by-step dataflow from pipeline definition to benchmarking.

## Why
Provides an audit-friendly chain of custody for artifacts.

## Non-goals
- Detailed execution tuning.

## Contracts
Enforced by tests:
- [../../crates/bijux-dna-engine/tests/contracts/recording/recording_completeness.rs](../../crates/bijux-dna-engine/tests/contracts/recording/recording_completeness.rs)
- [../../crates/bijux-dna-runtime/tests/contracts/manifest_integrity.rs](../../crates/bijux-dna-runtime/tests/contracts/manifest_integrity.rs)
- [../../crates/bijux-dna-analyze/tests/contracts/report/report_contract.rs](../../crates/bijux-dna-analyze/tests/contracts/report/report_contract.rs)
- Runtime artifact layout lives in [../30-operations/RUN_ARTIFACTS.md](../30-operations/RUN_ARTIFACTS.md).
- Report bundle layout lives in [../30-operations/REPORT_CONTRACT.md](../30-operations/REPORT_CONTRACT.md).
- Pipeline family entrypoints live in [../50-reference/PIPELINES.md](../50-reference/PIPELINES.md).

## Examples
1. Pipeline → Planner: plan JSON + graph hash.
2. Planner → Engine: `ExecutionGraph`.
3. Engine → Runner/Runtime: step directories with `tool_invocation.json`, `execution_record.json`.
4. Runtime → Analyze: `run_manifest.json` + `report.json`.
5. Analyze → Benchmark: report + summaries.
6. Benchmark suite specs are sourced from the governed pipeline and benchmark surfaces.

Exact file outputs:
- `run_manifest.json`
- `stage_<n>/tool_invocation.json`
- `stage_<n>/execution_record.json`
- `report.json`, `report.html`, `summary.tsv`

## Failure modes
Missing files or hash mismatches fail contract enforcement.
