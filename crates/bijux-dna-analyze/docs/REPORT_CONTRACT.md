# Report Contract

## Authority
This document is the source of truth for report artifact shape, schema expectations, privacy
requirements, and performance budgets owned by `bijux-dna-analyze`.

## Final Artifact Bundle
- `report.json`: canonical machine-readable report
- `report_bundle/index.html`: bundled human report with embedded canonical report data
- `report_bundle/report.json`: bundle-local canonical report copy
- `report.html`: optional legacy single-file rendering
- `report.md`: optional markdown rendering

## Required Top-Level Contract Fields
- `schema_version`
- `run_id`
- `contract`
- `completeness`
- `stages`
- `provenance`
- `run_provenance`
- `decision_config`
- `analysis_selection_contract`
- `metric_semantics`
- `telemetry`
- `sections`

## Required Sections
- `qc`
- `final_qc_summary`
- `retention_definition`
- `retention_context`
- `filter_interpretation`
- `reproducibility`
- `method_assumptions`
- `metric_semantics`
- `data_contract_validation`
- `pipeline_overview`
- `pipeline_verdict`
- `stage_completeness`
- `key_findings`
- `claims_registry`

## Data Model Rules
- Report rows are built from typed `FactsRowV1` values, not raw fixture strings.
- Raw JSON is allowed only at artifact boundaries and inside `JsonBlob` wrappers.
- Section names are stable snake_case keys.
- Optional additions must be backward compatible and absent-safe.
- Breaking field removals, type changes, or semantic changes require a new schema version.

## Determinism Rules
- JSON output uses canonical ordering where the renderer owns canonicalization.
- HTML table order is stable by stage and tool identity.
- Numeric output must not depend on hash-map iteration order.
- Timestamp and runtime fields may vary only when they are input data rather than renderer-created
  incidental values.

## Privacy Rules
- Do not emit secrets, access tokens, private environment values, or unredacted PII.
- Paths in fixtures should be normalized when they would otherwise leak local machines.
- Tool command lines may be shown for reproducibility but must not include secrets.

## Performance Budgets
- `report.json` must stay below 5 MB for fixture inputs.
- Report rendering must stay inside the local unit-test budget enforced by
  `tests/contracts/report/performance_budget.rs`.
- SQLite-backed load queries must use indexed lookups and avoid N+1 access patterns.

## Coverage
- `tests/contracts/report/report_contract.rs`
- `tests/contracts/report/report_determinism.rs`
- `tests/contracts/report/report_privacy.rs`
- `tests/contracts/report/report_size_budget.rs`
- `tests/contracts/report/performance_budget.rs`
