# Dataflow Narrative

## What
Step-by-step dataflow from pipeline definition to benchmarking.

## Why
Provides an audit-friendly chain of custody for artifacts.

## Non-goals
- Detailed execution tuning.

## Contracts
Enforced by tests:
- `crates/bijux-dna-engine/tests/recording_completeness.rs`
- `crates/bijux-dna-runtime/tests/manifest_integrity.rs`
- `crates/bijux-dna-analyze/tests/report_contract.rs`

## Examples
1. Pipeline → Planner: plan JSON + graph hash.
2. Planner → Engine: `ExecutionGraph`.
3. Engine → Runner/Runtime: step directories with `tool_invocation.json`, `execution_record.json`.
4. Runtime → Analyze: `run_manifest.json` + `report.json`.
5. Analyze → Benchmark: report + summaries.
6. Benchmark suite specs are sourced from `crates/bijux-dna-bench/bench/suites/`.

Exact file outputs:
- `run_manifest.json`
- `stage_<n>/tool_invocation.json`
- `stage_<n>/execution_record.json`
- `report.json`, `report.html`, `summary.tsv`

## Failure modes
Missing files or hash mismatches fail contract enforcement.
