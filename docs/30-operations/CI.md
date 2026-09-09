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

Full workspace archive and shard stdout/stderr are written directly beneath
`artifacts/ci/full-shards/` as `archive-build.stdout.log`, `archive-build.stderr.log`,
and `hash-N-of-5.stdout.log` / `hash-N-of-5.stderr.log`. Inspect these files while a
run is active. Completed combined logs and timing receipts remain available at
their existing paths. A new invocation replaces logs with the same labels; preserve
needed evidence before rerunning. A missing final receipt after interruption is
not a successful gate, even when partial logs contain passing tests.

The full-shard runner emits `bijux.ci.full_gate_timing.v4`. After archive creation, the producer lists its complete all-feature test inventory using the same source and Cargo target. This is an additional measured discovery operation. Shards emit pinned `libtest-json-plus` 0.1 events with zero retries and execute ignored tests. The final receipt includes the inventory digest, expected ordinary/ignored counts, missing/duplicate/failed cases, and invalid shard reports. Full coverage requires exact observed membership and successful complete event sequences, as well as all declared shards being selected. A selected recovery shard can pass without claiming full coverage.

`all_declared_shards_selected` records scheduling; `complete_contract_coverage` requires observed coverage. A fresh running receipt invalidates earlier success before producer execution; absent, running, failed or mismatched-source evidence cannot establish completion. The source commit and clean-worktree observation accompany the result; final qualification still requires an immutable clean candidate. The pinned event format is experimental upstream and must be qualified alongside any executor upgrade.

Nextest 0.9.100 can repeat suite blocks and retain an ignored counter when ignored cases execute. Admission accepts those bounded suite blocks while checking actual start/finish events and passed/failed counters. Every ignored case still needs its own successful result in the exact global case set; summary counters never substitute for execution.

Full-shard source preparation captures tracked changes once before archive production. Every isolated workspace checks out the captured commit and applies the same binary patch when needed; clean sources require no second file copy. Receipts include the patch digest and archive/shard preparation durations. Untracked files are excluded from the source snapshot.

Before discovery, each full shard excludes binary IDs with no cases in its existing hash partition. The plan uses the qualified Nextest 0.9.100 xxHash64 case-name protocol; the native hash partition remains enabled. Receipts retain discovery plans, and selected-shard success requires exactly the planned case outcomes. A different executor version requires qualification before this optimization is admitted.

The full sharded runner writes `archive-build.receipt.json` and `test-inventory.receipt.json` beneath its retained report root immediately after those operations finish. They bind completed timings to the source commit, tracked patch digest, and archive digest, so interruption during later work does not erase completed build/discovery evidence. The archive receipt also records the build-authority and shard-contract digests. These receipts do not establish test completion or authorize artifact reuse by themselves; the final timing and coverage report remains the completion authority.

The full Nextest profile runs the HPC dry-run readiness suites and the operational benchmark readiness suite in the global worker pool. Each test owns a separate repository sandbox and binds CLI repository, benchmark, and artifact roots to that sandbox. Other CLI generator suites remain in the serialized repository-mutator group until their isolation is qualified; hash shards still have distinct source workspaces.

Readiness coverage, shell command and argv JSONL contracts share the `bench_readiness_coverage` integration target. The five original cases retain their assertions and share one process-wide CWD/environment guard. The full profile keeps this shared-output family in its serialized benchmark group; `bench-adapter-fast` also checks the stage-tool JSON and TSV coverage contracts through the grouped target.

Local harness JSON/file checks and the all-domain operational completion check use independent repository sandboxes in the full-profile global worker pool. Harness commands and persisted-output reads use the same sandbox; their CWD/environment guard remains process-wide. Shared-output generators retain the serialized group until their own isolation is qualified.

Full benchmark report, result collector and dashboard checks bind every CLI child to an explicit independent repository sandbox and use the full-profile global worker pool. Original report assertions and command scopes remain intact; the dashboard independently acquires its expected-result comparison from the same governed inputs.
