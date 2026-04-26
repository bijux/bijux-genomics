# Determinism

## Scope

Replay determinism means engine scheduling, graph normalization, contract
verification, run-record shape, and manifest layout remain stable for identical
inputs. It does not mean every byte written during execution is identical,
because execution records include wall-clock timestamps.

## Inputs That Must Match

- Execution graph (including plan policy).
- Engine config after defaults are applied.
- Input artifacts and resolved paths.
- Runner responses: exit codes, durations, stdout/stderr, and artifacts.
- Runner-written output payloads.

## Stable Outputs

- Normalized graph order.
- Step execution order.
- Retry decision order.
- Hook event order for equivalent runner responses.
- Contract verification order.
- Run record structure and values except explicitly excluded timestamps.
- Manifest hash and layout tree in determinism fixtures.

## Ordering Guarantees

- Steps are scheduled in topological order.
- Ready steps are sorted when deterministic scheduling is enabled.
- Artifact scans in test fixtures sort entries before comparison.
- Contract checks run output checks, metrics envelope checks, and run-artifact
  checks in a stable sequence.

## Exclusions

- `started_at` and `finished_at` in engine-written execution records.
- Runtime-reported resource usage and durations.
- External runner behavior. The engine can make runner interaction deterministic
  only when the runner is deterministic.

## Enforced by

- `tests/determinism/replay_determinism.rs`
- `tests/determinism/manifest_layout_snapshot.rs`
- `tests/contracts/runner_tests.rs::execute_plan_orders_dag`

## Why

Determinism enables reproducibility and stable diffs for scientific runs.
