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
- The `ci-fast` DAG gives each concurrently ready group a stable child of the gate-scoped
  `artifacts/ci/ci-fast-target` authority. Per-group Cargo targets retain build output across
  invocations without serializing independent groups on one target lock. Groups share the pinned
  toolchain, workspace lockfile, and sccache authority, and execute Cargo offline after the
  launcher has resolved its own locked dependencies. Timing receipts name every group target. The
  critical-path baseline also counts Cargo lock-wait diagnostics and publishes the sum of affected
  target wall times as an explicit conservative upper bound; it never presents that bound as exact
  lock-held time.
- On Unix, the `ci-fast` supervisor gives all groups in one invocation a single explicit
  `SCCACHE_SERVER_UDS` and a short `/tmp` runtime directory. Both the server socket and sccache's
  startup-notification socket are therefore bounded independently of the persistent cache and
  evidence directories, so deep governed roots cannot prevent the cache server from binding.
- A warm `ci-fast` timing observation is reference-eligible only when pre-run and post-run sccache
  receipts name the same cache authority, that authority is not scoped to the source commit, and
  the measured run either reuses complete per-group build targets or reaches the contract's
  minimum compiler-cache hit ratio without cache errors. A caller's `warm` label is only a
  request; it is never evidence. Missing, disabled, commit-scoped, cold, or invalid cache
  observations remain diagnostic evidence and cannot contribute to the p95 claim.
- Complete-workspace hash shards each use a tracked-source clone and a distinct absolute artifact
  root. Cargo targets, test and coverage targets, temporary directories, and profraw directories
  are disjoint between shards; all shards retain the single toolchain and sccache authority from
  `configs/ci/build-authority.json`. Bounded logs and timing receipts remain beneath
  `ARTIFACT_ROOT`; high-volume archives, clones, targets, extraction trees, and runtime products
  use the disjoint absolute `BIJUX_CI_FULL_SHARD_SCRATCH_ROOT` when configured and are removed
  before the command returns.
- Complete-workspace shards also verify every executable and exact version line in the runtime
  dependency contract before compiling the shared archive. The receipt records the observed
  version and provisioning policy; a missing or drifted dependency fails once at preflight instead
  of cascading through unrelated test shards.
  LUNARC supplies BCFtools by loading `GCC/13.3.0` followed by `BCFtools/1.21` before invoking the
  shard command.
- The local helper surface for cache-aware Rust workflows is
  [makes/cargo-dev.mk](../../makes/cargo-dev.mk).
- Crate-local `RUSTFLAGS` overrides require governance review.

## Recommended Local Setup

1. Install `sccache`.
2. Export `RUSTC_WRAPPER=$(command -v sccache)` in your shell profile.
3. Use shared `artifacts/target` reuse for interactive local builds and sequential gates in one
   checkout. Concurrent independent runs receive isolated artifact roots.
4. For a Slurm full-shard invocation, set
   `BIJUX_CI_FULL_SHARD_SCRATCH_ROOT="$SNIC_TMP/bijux-dna-full-shards-$SLURM_JOB_ID"` while
   keeping `ARTIFACT_ROOT` on the governed evidence filesystem. The runner rejects relative,
   non-normalized, or overlapping scratch and evidence roots before materialization.

## Diagnostics

- `sccache --show-stats`
- `cargo run -q -p bijux-dna-dev -- checks run check-ci-build-authority`
- `make -f makes/cargo-dev.mk dev-test`

## Failure modes

- Treating cache presence as required CI behavior hides correctness regressions.
- Reusing shared target roots outside the governed artifact environment causes cross-run
  interference.
- Reusing one target directory across independent processes can serialize unrelated products;
  concurrent workflow gates and `ci-fast` groups use stable isolated targets beneath their
  respective governed build authorities.
- Letting sccache derive either its server socket or startup-notification socket below a deep
  shared root can exceed the platform path limit before compilation begins; use the
  supervisor-owned bounded socket and runtime-temporary authorities.
