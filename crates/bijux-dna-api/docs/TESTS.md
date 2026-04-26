# Tests

This file is the single test documentation entrypoint for `bijux-dna-api`.
README files are intentionally not allowed under `tests/`.

## Suite Entrypoints

- `tests/boundaries.rs` loads architecture, docs-layout, and guardrail tests.
- `tests/schemas.rs` loads schema snapshots, public surface, and documentation
  alignment tests.
- `tests/contracts.rs` loads public behavior and integration contract tests.
- `tests/guardrails.rs` runs the shared crate guardrail policy from the root.
- `tests/workspace_paths.rs` provides repository path helpers for integration
  tests and boundary aggregators.

## Boundary Tests

- `tests/boundaries/architecture.rs` protects the crate root shape, 10-docs
  allowance, absence of test READMEs, source tree layout, and v1 namespace tree.
- `tests/boundaries/guardrails.rs` runs the shared guardrail harness.
- `tests/boundaries/guardrails/args_module.rs` forbids vague `args.rs` modules.
- `tests/boundaries/guardrails/policies.rs` applies shared policy checks.
- `tests/boundaries/v1_cross_guardrails.rs` keeps cross-domain v1 modules from
  hard-coding stage id literals.

## Schema Tests

- `tests/schemas/v1_cross_api_stability.rs` locks response schema behavior.
- `tests/schemas/v1_cross_contract_handshake.rs` keeps contract fixtures aligned.
- `tests/schemas/v1_cross_docs_schema_snapshots.rs` ensures `docs/API.md`
  references the public schema snapshots.
- `tests/schemas/v1_cross_public_surface.rs` snapshots the public root export.
- `tests/schemas/v1_operator_failure_contract.rs` locks operator-facing failure
  envelopes.

## Contract Tests

- `tests/contracts/v1_cross_contract_spine.rs` checks the plan, manifest, and
  report contract spine.
- `tests/contracts/v1_cross_explain_roundtrip.rs` checks explainability
  round-tripping.
- `tests/contracts/v1_cross_public_contract.rs` checks curated public API usage.
- `tests/contracts/v1_dry_run_manifest.rs` checks dry-run manifest output.
- `tests/contracts/v1_fastq_small_integration.rs` checks a small FASTQ flow.
- `tests/contracts/fastq_amplicon_governance_contract.rs` checks governed FASTQ
  amplicon inputs and locks.

## Recommended Commands

Run from the `bijux-genomics` repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-api --test boundaries --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-api --test schemas --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-api --test contracts --no-default-features
```

Run `cargo test -p bijux-dna-api --all-features` before release-facing handoff
when feature-gated code changed.
