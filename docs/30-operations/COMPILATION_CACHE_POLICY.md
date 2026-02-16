# Compilation Cache Policy

Scope: local developer throughput only.

Rules:
- `sccache` is optional and local-only.
- CI correctness must not depend on cache presence.
- Shared dev runs should use isolate target roots (`$ISO_ROOT/target-*`) to avoid IDE contention.
- Do not add crate-local `RUSTFLAGS` overrides without governance review.

Recommended local setup:
1. Install `sccache`.
2. Export `RUSTC_WRAPPER=$(command -v sccache)` in your shell profile.
3. Keep cargo target under isolate (`./bin/isolate --reuse ...`) for deterministic reuse.

Diagnostics:
- `sccache --show-stats`
- `make -f makefiles/cargo-dev.mk dev-test`

Non-goals:
- No remote cache integration in CI.
- No hidden compile flags per crate.


## Purpose

Contract details are enforced by stage contracts, schema locks, and CI policy gates for this scope.


## Scope

Contract details are enforced by stage contracts, schema locks, and CI policy gates for this scope.


## Non-Goals

Contract details are enforced by stage contracts, schema locks, and CI policy gates for this scope.


## Contracts

Contract details are enforced by stage contracts, schema locks, and CI policy gates for this scope.
