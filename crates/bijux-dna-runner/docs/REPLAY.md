# REPLAY

Replay never executes tools. It only verifies artifacts on disk and reconstructs
records deterministically.

## Does
- Validate artifacts exist and are non-empty.
- Reconstruct records deterministically.

## Does not
- Spawn processes
- Pull images
- Execute tool commands

## Enforced by
- `tests/determinism/replay/replay_contract.rs`
- `tests/determinism/replay/replay_determinism.rs`

## Integrity failure
Missing or mismatched artifacts cause failure.
