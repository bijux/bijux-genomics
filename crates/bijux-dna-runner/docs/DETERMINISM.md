# Determinism

Runner determinism is about stable invocation identity, stable records, and replay that verifies existing artifacts without executing tools.

## Invocation Identity
- Invocation hashes include command, args, environment, working directory, mounts, and image digest.
- Hash inputs must be canonical and order-stable.
- Directory input hashing walks paths deterministically.

## Replay
Replay does:
- Read execution manifests and runner-owned records.
- Verify declared artifacts exist and are non-empty where required.
- Reconstruct records deterministically from existing files.

Replay does not:
- Spawn backend commands.
- Pull images.
- Re-run observer commands.
- Modify declared inputs.

## Stable Outputs
- stdout and stderr are captured verbatim.
- Exit codes are recorded without remapping successful exits.
- Nonzero exits remain failures during replay.

## Validation
- `tests/determinism/run_id_determinism.rs` covers run-id stability.
- `tests/determinism/replay.rs` and `tests/determinism/replay/replay_*.rs` cover replay behavior where present.
- `tests/boundaries/backend/invocation_hash.rs` covers invocation identity rules.
