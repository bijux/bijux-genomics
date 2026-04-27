# MATURITY_LADDER

## What
Defines which policy classes are required vs optional by maturity level.

## Why
Clarifies the minimum enforcement bar for new or experimental crates.

## Non-goals
- Replacing the policy matrix.

## Contracts
- Required policy classes are cataloged in [POLICY_MATRIX.md](POLICY_MATRIX.md).
- Stability and compatibility expectations are defined in
  [POLICY_STABILITY.md](POLICY_STABILITY.md).
- Workspace enforcement ownership lives in
  [crates/bijux-dna-policies/README.md](../../crates/bijux-dna-policies/README.md).

## Examples
## Required
- effect boundaries
- dependency boundaries
- docs placement

## Optional
- policy snapshots for experimental crates

Rationale: required policies prevent correctness regressions.

## Failure modes
- Unclear maturity expectations lead to inconsistent enforcement.
