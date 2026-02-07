# Tests

## What
Maps tests in this crate to their purpose and failure meaning.

## Why
Tests should explain the contract they enforce.

## Non-goals
- Full test implementation detail.

## Contracts
- Each test file should be referenced here.

## Examples
- `tests/guardrails/architecture.rs` → dependency boundary assertions.

## Failure modes
- Missing test documentation causes drift and confusion.

## Golden tests
Run with:
`cargo test -p bijux-cli help dry_run`

## help
- `tests/help/docs_help_snapshots.rs` — help output snapshots.

## dry_run
- `tests/dry_run/fastq_golden.rs` — dry-run golden outputs.
- `tests/dry_run/fastq_pipeline.rs` — dry-run pipeline selection.
- `tests/dry_run/fastq_run.rs` — dry-run run invocation shape.

## banks
- `tests/banks/adapter_bank_schema.rs` — adapter bank schema contract.
- `tests/banks/bank_preset_resolution.rs` — preset resolution contract.
- `tests/banks/cli_preset_interaction.rs` — CLI preset interaction contract.
- `tests/banks/contaminant_bank_schema.rs` — contaminant bank schema contract.
- `tests/banks/polyx_bank_schema.rs` — polyX bank schema contract.

## guardrails
- `tests/guardrails/architecture.rs` — architecture boundary enforcement.
- `tests/guardrails/architecture_guardrail.rs` — tree/layout guardrail.
- `tests/guardrails/ci_contract.rs` — CI contract checks.
- `tests/guardrails/deps.rs` — dependency boundary checks.
- `tests/guardrails/no_process_spawn.rs` — no direct process spawn.
- `tests/guardrails/policies.rs` — shared policy enforcement.
- `tests/guardrails/public_surface.rs` — public surface lock.

## Testkit patterns
See `crates/bijux-testkit/docs/USAGE.md` for shared fixture and snapshot helpers.
