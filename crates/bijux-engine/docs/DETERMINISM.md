# DETERMINISM

## Scope
Given the same execution graph, inputs, and policy:

- Graph hash is stable.
- Step hashes are stable.
- Layout tree is stable (paths and names).

## Exclusions
- Wall-clock timestamps.
- Runtime-reported resource usage.

## Why
Determinism enables reproducibility and stable diffs for scientific runs.
