# Tests

## What
Maps tests in this crate to their purpose and failure meaning.

## Why
Tests should explain the contract they enforce.

## Suite entrypoints
- `tests/boundaries.rs` loads architecture and guardrail coverage.
- `tests/contracts.rs` loads public contract and integration coverage.
- `tests/schemas.rs` loads schema and public-surface stability checks.
- `tests/guardrails.rs` and `tests/workspace_paths.rs` keep crate-level support checks visible at the root.

## Boundaries suite (`tests/boundaries/*`)
- `tests/boundaries/architecture.rs` — source tree matches the documented crate architecture.
- `tests/boundaries/guardrails.rs` — shared guardrail harness for the crate.
- `tests/boundaries/guardrails/args_module.rs` — generic `args.rs` files stay forbidden.
- `tests/boundaries/guardrails/policies.rs` — shared policy enforcement.
- `tests/boundaries/v1_cross_guardrails.rs` — v1 layout guardrails.

## Contracts suite (`tests/contracts/*`)
- `tests/contracts/fastq_amplicon_governance_contract.rs` — governed fastq runtime behavior remains stable.
- `tests/contracts/v1_cross_contract_spine.rs` — plan, manifest, and report contract spine stays coherent.
- `tests/contracts/v1_cross_explain_roundtrip.rs` — explainability payloads round-trip.
- `tests/contracts/v1_cross_public_contract.rs` — public API contract stays curated.
- `tests/contracts/v1_dry_run_manifest.rs` — dry-run manifest output remains stable.
- `tests/contracts/v1_fastq_small_integration.rs` — fastq integration contract remains intact.

## Schemas suite (`tests/schemas/*`)
- `tests/schemas/v1_cross_api_stability.rs` — API response schemas are stable.
- `tests/schemas/v1_cross_contract_handshake.rs` — fixtures stay aligned across public contracts.
- `tests/schemas/v1_cross_docs_schema_snapshots.rs` — docs and schema snapshots stay aligned.
- `tests/schemas/v1_cross_public_surface.rs` — the public surface remains curated.
- `tests/schemas/v1_operator_failure_contract.rs` — operator-facing failure envelopes stay stable.

## Testkit patterns
See `crates/bijux-dna-testkit/docs/USAGE.md` for shared fixture and snapshot helpers.
