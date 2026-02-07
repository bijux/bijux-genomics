# LEGACY (Debt Ledger)

## Why it exists
Legacy benchmarks support older datasets and tools.

Status: frozen. Legacy support is a deprecation path only; no new features or schema changes should land in `legacy/`.

## What it blocks
- Simplified schema unification
- Removal of deprecated metrics

## Sunset criteria
- No consumers in two minor releases
- Replacement benchmarks validated

## Replacement
Use current benchmark suite under `bench` module.
