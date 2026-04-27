# Crate Map

## What
Mapping of crates to roles and guarantees.

## Why
Provides quick architectural orientation.

## Non-goals
- Deep API docs.

## Contracts
- Crate ownership and responsibilities are cataloged in
  [CRATE_AUTHORITY_MAP.md](../10-architecture/CRATE_AUTHORITY_MAP.md).
- Allowed boundary edges are defined in
  [BOUNDARY_MAP.md](../10-architecture/BOUNDARY_MAP.md).
- Required enforcement level by crate maturity is defined in
  [MATURITY_LADDER.md](../40-policies/MATURITY_LADDER.md).

## Examples
- bijux-dna-core defines IDs and contracts.

## Failure modes
- Boundary claims drift from the architecture authorities and reviewers lose a fast map of
  which crate owns which responsibility.

| Crate | Role | SSOT ownership | Purity guarantees | Key types |
| --- | --- | --- | --- | --- |
| bijux-dna-core | Contract bible | IDs + canonicalization | No effects | ExecutionGraph |
| bijux-dna-engine | Orchestrator | None | No execution | Engine |
| bijux-dna-runtime | Recording | Run layout | Effect‑free except layout | RunLayout |
| bijux-dna-runner | Execution backends | None | Allowed effects | Runner |
| bijux-dna-api | Orchestration | None | No direct execution | PlanRequest |
