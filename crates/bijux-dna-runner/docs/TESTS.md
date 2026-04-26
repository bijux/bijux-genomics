# Tests

This file maps runner tests to their contract purpose. Directory-local README files are intentionally not used; this document is the test taxonomy for the crate.

## Suite Entrypoints
- `tests/boundaries.rs` loads architectural and backend guardrail coverage.
- `tests/contracts.rs` loads runner-facing behavioral contracts.
- `tests/determinism.rs` loads replay and run-id stability coverage.
- `tests/guardrails.rs`, `tests/schemas.rs`, and `tests/semantics.rs` keep focused crate-level checks visible at the root.
- `tests/support/workspace_paths.rs` provides repository path helpers without creating a helper-only test binary.

## Boundaries Suite
- `tests/boundaries/architecture.rs` checks root, source, and test tree shape.
- `tests/boundaries/command_inventory.rs` checks `docs/COMMANDS.md`.
- `tests/boundaries/dependency_graph.rs` checks `Cargo.toml` dependency boundaries.
- `tests/boundaries/guardrails.rs` checks workspace-level runner guardrails.
- `tests/boundaries/backend/backend_invariants.rs` checks backend invariants are documented.
- `tests/boundaries/backend/invocation_hash.rs` checks stable invocation identity rules.
- `tests/boundaries/backend/network_guardrail.rs` checks network access remains outside runner execution.
- `tests/boundaries/backend/process_guardrail.rs` checks process spawning stays confined.

## Contracts Suite
- `tests/contracts/backend.rs` checks backend contract behavior visible to crate consumers.

## Determinism Suite
- `tests/determinism/determinism.rs` is the suite-level determinism coverage anchor.
- `tests/determinism/run_id_determinism.rs` checks run-id stability.
- `tests/determinism/replay.rs` and `tests/determinism/replay/replay_*.rs` check replay contract behavior where present.

## Schemas and Semantics
- `tests/schemas/docs_backend_invariants.rs` checks execution-spec docs cover backend invariants.
- `tests/semantics/docker_parsing.rs` checks backend command parsing semantics.

## Main Command

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-runner --no-default-features
```

## Failure modes
- Tree-shape failures mean code or docs moved without updating the runner architecture contract.
- Dependency failures mean runtime or dev dependencies crossed the documented boundary.
- Command inventory failures mean `docs/COMMANDS.md` is no longer the command source of truth.
- Determinism failures mean identity or replay behavior changed.
