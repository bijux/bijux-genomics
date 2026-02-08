# MATURITY_LADDER

## What
Defines which policy classes are required vs optional by maturity level.

## Why
Clarifies the minimum enforcement bar for new or experimental crates.

## Non-goals
- Replacing the policy matrix.

## Contracts
- Required policies must pass for all production crates.
- Optional policies are explicitly documented when deferred.

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
