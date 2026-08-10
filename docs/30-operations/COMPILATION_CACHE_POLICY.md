# Compilation Cache Policy

## Purpose
Define the compilation-cache and Cargo target-isolation contract without making correctness depend
on cache state.

## Scope
Applies to local Rust build reuse, CI toolchain and lock identity, sccache receipts, shared prebuild
targets, and isolated concurrent target roots.

## Non-goals
- Hidden crate-specific compile flags.

## Contracts
- `configs/ci/build-authority.json` is the machine-evaluable toolchain, lock, cache, and target
  authority.
- CI uses the exact channel in `rust-toolchain.toml`, resolves the single workspace `Cargo.lock`
  with locked metadata, and uses `sccache` as its compiler wrapper.
- CI correctness must not depend on cache presence.
- Each GitHub-workflow gate publishes JSON `sccache-before.json` and `sccache-after.json` receipts,
  including a final receipt on failure. Deltas between the receipts provide request, hit, miss,
  write, error, and cache-size metrics for the run.
- Shared or automated dev runs must honor the governed artifact environment in
  [ISOLATION.md](ISOLATION.md).
- Sequential canonical Rust gates within one checkout reuse `artifacts/target` and
  `artifacts/cargo/home`. The GitHub-workflow supervisor prebuilds the shared developer binary,
  then gives each concurrent gate its own `artifacts/github-all/targets/<gate>` target root so no
  useful concurrency is serialized by one Cargo target lock. Gates reuse compiler output through
  sccache and reuse only the immutable path to the prebuilt developer binary.
- The local helper surface for cache-aware Rust workflows is
  [makes/cargo-dev.mk](../../makes/cargo-dev.mk).
- Crate-local `RUSTFLAGS` overrides require governance review.

## Recommended Local Setup
1. Install `sccache`.
2. Export `RUSTC_WRAPPER=$(command -v sccache)` in your shell profile.
3. Use shared `artifacts/target` reuse for interactive local builds and sequential gates in one
   checkout. Concurrent independent runs receive isolated artifact roots.

## Diagnostics
- `sccache --show-stats`
- `cargo run -q -p bijux-dna-dev -- checks run check-ci-build-authority`
- `make -f makes/cargo-dev.mk dev-test`

## Failure modes
- Treating cache presence as required CI behavior hides correctness regressions.
- Reusing shared target roots outside the governed artifact environment causes cross-run
  interference.
- Reusing one target root across concurrent Cargo gates serializes on Cargo's target lock and
  defeats the declared DAG.
