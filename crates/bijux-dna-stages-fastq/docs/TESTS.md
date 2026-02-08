# Tests

## What
Maps tests in this crate to their purpose and failure meaning.

## Why
Tests should explain the contract they enforce.

## Non-goals
- Full test implementation detail.

## Contracts
- `tests/contracts/*` → stage specs, registry, symmetry, and contract snapshots (docs/STAGE_CONTRACTS.md).
- `tests/observer/*` → observer parsing + determinism (docs/OBSERVERS.md).
- `tests/purity/*` → declarative-only and boundary checks (docs/ARCHITECTURE.md).

## Mapping
- `tests/contracts/contract_snapshots.rs` → stage contract snapshots.
- `tests/contracts/registry_completeness.rs` → registry completeness.
- `tests/contracts/symmetry.rs` → contract-level symmetry only.
- `tests/contracts/structure_contract.rs` → stages file structure.
- `tests/observer/observer_parsers.rs` → observer fixture parsing.
- `tests/observer/observer_determinism.rs` → stable observer outputs.
- `tests/purity/architecture.rs` → no execution details in stages-fastq.

## Failure modes
- Missing test documentation causes drift and confusion.

## Testkit patterns
See `crates/bijux-dna-testkit/docs/USAGE.md` for shared fixture and snapshot helpers.
