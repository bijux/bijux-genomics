# DETERMINISM

## Scope
Replay determinism means the engine produces identical outputs for identical inputs,
subject to explicit exclusions.

### Inputs that must be identical
- Execution graph (including plan policy).
- Input artifacts and their resolved paths.
- Runner results (exit codes, outputs, and artifacts).

### Outputs that must be identical
- Graph hash and step hashes.
- Run layout tree (paths and names).
- Recorded manifest and run record (except exclusions below).

### Ordering guarantees
- Steps are scheduled in a stable order derived from the graph.
- Artifact enumeration is deterministic.

## Exclusions
- Wall-clock timestamps.
- Runtime-reported resource usage.

## Enforced by
- `tests/determinism/replay_determinism.rs`
- `tests/determinism/manifest_layout_snapshot.rs`

## Why
Determinism enables reproducibility and stable diffs for scientific runs.
