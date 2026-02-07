# Crate Map

## What
Mapping of crates to roles and guarantees.

## Why
Provides quick architectural orientation.

## Non-goals
- Deep API docs.

## Contracts
- Crate boundaries are enforced by policy tests.

## Examples
- bijux-core defines IDs and contracts.

## Failure modes
- Boundary violations fail CI.

| Crate | Role | SSOT ownership | Purity guarantees | Key types |
| --- | --- | --- | --- | --- |
| bijux-core | Contract bible | IDs + canonicalization | No effects | ExecutionGraph |
| bijux-engine | Orchestrator | None | No execution | Engine |
| bijux-runtime | Recording | Run layout | Effect‑free except layout | RunLayout |
| bijux-runner | Execution backends | None | Allowed effects | Runner |
| bijux-api | Orchestration | None | No direct execution | PlanRequest |
