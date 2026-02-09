# Pre-HPC Scope Closure Checklist

## What
Defines objective acceptance signals to declare Phase 3 (truth/observability/operator hardening) closed.

## Why
Prevents subjective scope debates and forces auditable closure criteria.

## Non-goals
- Defining post-HPC performance SLAs.
- Replacing per-crate contract docs.

## Contracts
- Telemetry taxonomy is enum-constrained and rejects unknown event names.
- Telemetry attrs are typed (`BTreeMap<String, AttrValue>`), not unbounded JSON blobs.
- Telemetry timestamps are RFC3339 UTC via typed time fields.
- Dry-run and execute always emit a canonical `run_summary.json` artifact.
- CLI contract tests cover `plan`, `dry-run`, `explain`, `env`, and `pipelines list` surfaces.
- API v1 handler/schema tests are deterministic and snapshot-checked.
- Environment-QA covers image catalog + runner edge-case behavior.
- Defaults ledger is machine-checkable and includes provenance/comparability fields.
- FASTQ→BAM cross-domain handoff contracts are type-checked and documented.
- Docs policy enforces crate-root `README.md` + `crate/docs/*` placement.
- Fixture metadata is machine-readable (`CASE.toml`/`CASE.json`) where churn matters.

## Examples
- `cargo test -p bijux-dna-runtime --test contracts mod_contracts_telemetry_contract_rs`
- `cargo test -p bijux-dna-api --test contracts v1_dry_run_manifest`
- `cargo test -p bijux-dna-cli --test contracts cli_behavior`
- `cargo test -p bijux-dna-environment-qa --test contracts`

## Failure modes
- Unknown telemetry events deserialize successfully.
- Dry-run/execute outputs missing `run_summary.json`.
- API/CLI response drift not caught by snapshots.
- Unstructured fixture metadata causes high-churn review loops.
