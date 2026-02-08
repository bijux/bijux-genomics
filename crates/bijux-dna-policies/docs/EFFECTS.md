# Effects

## What
Defines the effects that `bijux-dna-policies` is allowed to use.

## Why
Policies must be deterministic and side‑effect free.

## Non-goals
- Running tools or modifying workspace state.

## Contracts
Allowed:
- Filesystem read access to inspect workspace state.

Forbidden:
- Process execution.
- Network access.
- Docker or container APIs.
- Randomness or time‑dependent behavior in policy logic.

## Examples
- Reading `Cargo.toml` to verify dependencies.

## Failure modes
- Any effectful API usage fails policy scans.

## Proof
- Policies are implemented as deterministic tests with file inspection only.
- CI enforces dependency budgets and effect boundaries.
