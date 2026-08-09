# Determinism

Runner determinism is about stable invocation identity, stable records, and replay that verifies existing artifacts without executing tools.

## Invocation Identity
- Invocation hashes include command, args, environment, working directory, mounts, and image digest.
- Hash inputs must be canonical and order-stable.
- Directory input hashing walks paths deterministically.
- Container qualification attempt identity includes the permanent run ID, tool ID, campaign ID,
  and attempt ID. Scheduler job IDs remain provenance and never select an attempt directory.
- Current qualification paths are derived through the shared attempt-layout resolver; v3 and v4
  compatibility readers reproduce their historical paths without rewriting evidence.
- Failed current qualification attempts preserve that identity through the shared complete-file
  archive receipt; rollback re-verifies the same inventory before restoring the active path.

## Replay
Replay does:
- Read execution manifests and runner-owned records.
- Verify the recorded output directory exists.
- Verify declared input files still match manifest hashes.
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
- `tests/determinism/replay.rs` and `tests/determinism/replay/replay_*.rs` cover replay behavior.
- `tests/boundaries/backend/invocation_hash.rs` covers invocation identity rules.
