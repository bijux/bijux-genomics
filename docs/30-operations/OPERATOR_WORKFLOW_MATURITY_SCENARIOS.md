# Operator Workflow Maturity Scenarios

This operation exercises iteration-20 goals G191-G200 through governed operator-workflow evaluators.

## Command

```bash
cargo run -q -p bijux-dna-dev -- tooling run operator-workflow-maturity
```

Optional filters:

```bash
cargo run -q -p bijux-dna-dev -- tooling run operator-workflow-maturity -- --scenario G191
cargo run -q -p bijux-dna-dev -- tooling run operator-workflow-maturity -- --scenario g200_resource_prediction_from_past_runs
cargo run -q -p bijux-dna-dev -- tooling run operator-workflow-maturity -- --out artifacts/operator_workflow_maturity/custom.json
```

## Output

- Default report path: `artifacts/operator_workflow_maturity/scenario_suite.json`
- Each row records `goal_id`, `scenario_id`, `status`, scenario notes, and structured evidence.

## Covered Goals

- `G191` `g191_workflow_import_export_package`: workflow package export/import with preserved run identity and caveats.
- `G192` `g192_run_comparison_command`: run delta reporting across stages, tools, references, artifacts, metrics, caveats, and trust class.
- `G193` `g193_artifact_retention_simulation`: retention planning for delete/compress/archive/retain decisions from replay semantics.
- `G194` `g194_artifact_deduplication_lineage`: digest-based dedup planning with preserved producer/consumer lineage.
- `G195` `g195_cache_corruption_quarantine`: sha/size mismatch quarantine while retaining valid cache entries.
- `G196` `g196_bundle_portability_check`: copied-bundle portability checks for relative paths and required evidence files.
- `G197` `g197_offline_review_profile`: no-network review profile requiring local evidence verification files.
- `G198` `g198_operator_command_recipes`: canonical run/inspect/replay/diff/export recipes mapped to concrete evidence paths.
- `G199` `g199_scale_aware_progress_reporting`: sample/stage/artifact progress summaries with explicit failure rows.
- `G200` `g200_resource_prediction_from_past_runs`: advisory CPU/memory/scratch suggestions from successful historical runs.

## Purpose
This document describes the governed intent and operator-facing meaning of this surface.

## Scope
The scope is limited to repository-owned behavior, contracts, and evidence paths for this topic.

## Non-goals
This document does not redefine source-of-truth schemas, code ownership boundaries, or release policy outside this surface.

## Contracts
Claims here are valid only when they remain consistent with governed configs, domain authorities, and policy checks.

