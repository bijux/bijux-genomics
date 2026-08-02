# Compilation Cache Policy

## Purpose
Define the local compilation-cache contract without making CI correctness depend on cache state.

## Scope
Applies to developer-oriented Rust build reuse, shared target-root conventions, and optional local
`sccache` use.

## Non-goals
- Remote cache integration in CI.
- Hidden crate-specific compile flags.

## Contracts
- `sccache` is optional and local-only.
- CI correctness must not depend on cache presence.
- Shared or automated dev runs must honor the governed artifact environment in
  [ISOLATION.md](ISOLATION.md).
- Canonical Rust gates and repository-specific gates within one checkout reuse the same
  `artifacts/target` build cache and `artifacts/cargo/home` dependency cache.
- The local helper surface for cache-aware Rust workflows is
  [makes/cargo-dev.mk](../../makes/cargo-dev.mk).
- Crate-local `RUSTFLAGS` overrides require governance review.

## Recommended Local Setup
1. Install `sccache`.
2. Export `RUSTC_WRAPPER=$(command -v sccache)` in your shell profile.
3. Use shared `artifacts/target` reuse for interactive local builds and sequential gates in one
   checkout. Frozen or concurrent independent runs receive isolated artifact roots.

## Diagnostics
- `sccache --show-stats`
- `make -f makes/cargo-dev.mk dev-test`

## Failure modes
- Treating cache presence as required CI behavior hides correctness regressions.
- Reusing shared target roots outside the governed artifact environment causes cross-run
  interference.
