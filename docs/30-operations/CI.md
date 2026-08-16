# CI

## What
CI enforces the minimal deterministic gate for the workspace.

## Purpose
Define the canonical CI gate contract and shared artifact invocation for the repository.

## Command
- `make ci`

## Current `make ci` Gates
- `fmt`
- `lint`
- `audit`
- `test`
- `coverage`

## Fast Local Gate

- `make gate-essential`
- Non-waiting execution policy: `cargo run -q -p bijux-dna-dev -- checks run check-no-wait-gate`
- Dependency graph policy: `cargo run -q -p bijux-dna-dev -- checks run check-ci-dependency-graph`
- The non-waiting check scans tracked scheduler, CI, automation, and lifecycle sources once and writes
  `artifacts/architecture/no-wait-gate.json`. A bounded internal retry/backoff is admissible only
  through an exact, expiring entry in `configs/ci/no_wait_allowlist.toml`; scheduler and completion
  observation must remain one-shot.
- The dependency graph in `configs/ci/dependency-graph.json` binds every workspace crate and local
  Cargo edge, plus generated schemas, contracts, and reports. Impact propagates from a changed
  dependency to all declared consumers. Unmatched changes fail closed to the full gate, and the
  nightly full lane validates reduced planning.
- Coverage: crate boundaries, domain YAML validation, stage-contract snapshots, planner determinism, runtime layout, and evidence discovery basics.
- Intended use: local architectural integrity checks during focused iteration without paying the full `make ci` cost.

## CI Profiles
- Fast CI profile: `cargo run -q -p bijux-dna-dev -- tooling run ci-fast`
- Slow CI profile: `cargo run -q -p bijux-dna-dev -- tooling run ci-slow`
- Fast profile intent: static/policy/contract gates with deterministic runner settings.
- Slow profile intent: heavier coverage/docs/release-readiness checks.
- Every fast invocation writes `artifacts/ci/baseline-timing.json` with the exact target commands
  and dependency graph, per-target wall times, sccache hit/miss deltas, Cargo lock-wait diagnostic
  counts and conservative duration bounds, and the full-workspace, LUNARC integration, and
  nightly/release lanes that the fast profile intentionally does not execute.
- Test/coverage runner defaults are pinned in
  [configs/rust/nextest.toml](../../configs/rust/nextest.toml) and
  [configs/coverage/runner.toml](../../configs/coverage/runner.toml).

## Full Workspace Shards

- `configs/ci/full-shards.json` owns the exactly-once five-way hash partition contract.
- The warm-cache reference allocation is 32 CPUs and 128 GiB. The archive build uses eight jobs;
  shard execution begins only after the immutable archive closes, so these resources do not
  contend with shard processes.
- `ci-test-all-sharded` runs all five partitions by default. Repeated `--shard <id>` arguments are
  reserved for classified affected recovery and must publish a receipt that disclaims complete
  workspace coverage.
- Timing receipts must report archive duration, each selected shard duration, checksum stability,
  wall time, cache authority, and whether the execution covered the complete contract.

## Artifact Contract
- See [docs/30-operations/ISOLATION.md](ISOLATION.md).

## HPC Forward-compat
- With HPC enabled, `make ci` still enforces the same gate order and policy checks.
- Path roots and container storage may resolve to HPC profile locations, not local defaults.
- Use profile-aware commands and avoid hardcoded local paths in docs automation.

## Non-goals
- Documenting non-`make ci` target suites.

## Scope
Applies only to the files and workflows referenced in this document.

## Contracts
- Content here is normative where explicitly stated.

## Examples
- Local: `make ci`
- Local fast architecture gate: `make gate-essential`
- HPC profile enabled: `ARTIFACT_ROOT=artifacts make ci`
- Fast profile: `cargo run -q -p bijux-dna-dev -- tooling run ci-fast`
- Slow profile: `cargo run -q -p bijux-dna-dev -- tooling run ci-slow`

## Failure modes
- Running CI-related scripts outside the shared artifact contract fails by policy.
- HPC path assumptions in docs/scripts can cause false failures when profile roots differ.
