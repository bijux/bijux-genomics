# Makes Public Surface

Public targets (stable contract):
- `fmt`
- `lint`
- `audit`
- `test`
- `test-slow`
- `test-all`
- `test-preflight`
- `test-complete`
- `ci-test-runtime`
- `ci-test-build`
- `ci-test-shard`
- `ci-test-aggregate`
- `ci-test-python`
- `test-all-frozen`
- `lint-frozen`
- `audit-frozen`
- `github-all`
- `github-all-frozen`
- `coverage`
- `ci`
- `doctor`
- `release-gate`
- `gate-essential`
- `gate-execute`
- `gate-evidence`
- `gate-release-essential`
- `refresh-assets-toy`
- `refresh-assets-golden`
- shared Rust targets: `format-rs`, `fmt-rs`, `lint-rs`, `test-rs`, `test-slow-rs`, `test-all-rs`, `audit-rs`, `coverage-rs`, `doctor-rs`, and `rustdoc-check`

Repository-owned implementation targets outside this list are internal and must be prefixed with `_`.

Benchmark note:
- benchmark orchestration and dossier rendering belong to `bijux-dna`
- benchmark make targets are thin wrappers around `bijux-dna bench ...`
- benchmark path authority belongs to `configs/bench/benchmark.toml`

Internal targets can be listed with:
- `SHOW_INTERNAL=1 make help`

Target -> implementation mapping (no hidden magic):
- `fmt`, `lint`, `test`, `test-slow`, and `coverage` -> shared `bijux-makes-rs` targets through `makes/bin/run_genomics_rust_gate.sh`
- `test-all` -> the Genomics complete native Rust shard executor through the repository Rust-gate adapter
- `test-preflight` -> pinned Nextest and full-suite native dependency checks without building test binaries
- `test-complete` -> all declared Rust, doctest and Python lanes with exact aggregate membership
- `audit` -> the shared Rust audit after Genomics audit-governance prerequisites
- `test-all-frozen`, `lint-frozen`, and `audit-frozen` -> shared immutable-ref launcher
- `github-all` -> the unique Make commands declared by `.github/workflows/ci.yml`, discovered from
  the pinned source and run concurrently with isolated logs and live start/completion status.
  Shared developer CLI and workspace test preparation runs only when the discovered workflow
  contains a test gate; the declared gate names are preserved. The current workflow declares
  `fmt`, `lint`, and `audit`, so it does not trigger test preparation or run a test lane.
- `github-all-frozen` -> the shared immutable-ref launcher running `github-all` after verifying that the requested ref provides the gate
- `doctor` -> `cargo run -q -p bijux-dna-dev -- tooling run repo-doctor --fast` + fast parity checks
- `release-gate` -> docs + root layout + registry lock + container version lock/authority checks
- `gate-essential` -> fast architecture + domain schema + planner determinism + runtime/evidence contract lane
- `gate-execute` -> local/container runner + governed execute/dry-run/status/replay lane, including runtime operations contracts for backend descriptors, queue state, run leases, run control, and operator health
- `gate-evidence` -> FASTQ manifest->plan->runtime->artifact->evidence verification lane plus planner-only BAM/VCF evidence contracts
- `gate-release-essential` -> Level 1 release gate across integrity, canonical smoke execution, dry-run/status evidence coverage, refusal coverage, and advisory/enforced admission checks
- `ci` -> `make fmt lint audit test coverage` under the shared `artifacts/` contract

Rust gate artifact layout:
- fast Rust gate reports write under `artifacts/rust/`, while compiled objects and dependencies
  reuse the common `artifacts/target/` and `artifacts/cargo/home/` caches
- `make test-all-frozen`, `make lint-frozen`, `make audit-frozen`, and `make github-all-frozen` start the requested gate from `PINNED_REF` (default `HEAD`) and write gate-owned state under `artifacts/<sha>/gates/<gate>/`
- pinned-ref gate runs materialize the exact source snapshot under `artifacts/<sha>/gates/<gate>/frozen-repo/` so reports and code stay pinned to the same commit
- pinned-ref gate runs isolate Cargo and generated state under `artifacts/<sha>/gates/<gate>/artifacts/`, allowing different gates for the same commit to run concurrently
- pinned-ref gate runs record launcher state under `artifacts/<sha>/background/`, including `<gate>.console.log`, `<gate>.pid`, and `<gate>.exit.status`
- `make github-all-frozen` rejects refs that predate the `github-all` contract before starting a background process
- `make github-all-frozen` records live per-gate progress in the console log and records per-gate logs, exit statuses, and an aggregate summary under `artifacts/<sha>/github-all/`
- When a discovered workflow contains a test gate, `make github-all` records shared preparation
  under `artifacts/github-all/workspace-compile.log`; workflows without test gates skip this work.
- `make test-all` runs the complete workspace, all-target, all-feature suite (including ignored
  tests) as five exact hash partitions through the event-driven Rust supervisor; shard logs and
  the timing receipt remain under `ARTIFACT_ROOT`. Archives, isolated workspaces, build targets,
  and shard runtime products use the disjoint `BIJUX_CI_FULL_SHARD_SCRATCH_ROOT` when supplied and
  are removed before return. This keeps bounded evidence governed while permitting high-volume
  execution on job-local scratch without writing build products into the source checkout.
- `make test-all-frozen` runs the complete native suite from its immutable source snapshot with
  no fast/slow filter expression or slow-test termination; slow-test reporting remains enabled.
- `make test-all` and `make test-all-frozen` exclude doctests and Python tests. Use
  `make test-complete` for the declared Rust, doctest, and Python lanes.
- `make lint` checks the complete workspace, including `bijux-dna-dev`, with all targets and
  features, locked dependencies and warnings denied
- `make lint-workspace` is the CI-sized workspace lane for configuration, documentation, and automation-boundary contracts; Rust formatting and product-crate clippy remain independent CI gates
- exhaustive workspace governance, domain, container, and Rust lint checks remain available through `make lint-governance`
- `make test` is the fast Rust lane: it excludes named plus rostered slow tests above the 1-second threshold
- `make test-slow` is the lane for tests that exceed the fast-lane budget
- `make test-all` is the complete, concurrently sharded full-suite lane with bounded worker admission

The slow roster accounts for fixture setup and child workflows as part of each test’s runtime.
Slow cases remain selected by `test-slow` and `test-all`.

The complete executor accepts larger complete partition rosters, including hundreds of shards.
It admits preparation and test execution together within a detected CPU allocation. Set
`BIJUX_CI_FULL_WORKER_BUDGET` to the allocated total case workers,
`BIJUX_CI_FULL_SHARD_JOBS` to the maximum active shard processes, and
`BIJUX_CI_FULL_SHARD_TEST_THREADS` to each process’s case threads. Positive decimal values
are required. Process jobs and requested case threads are ceilings, not guaranteed occupancy:
their requested product may exceed the worker budget, while weighted admission keeps concurrent
startup and case demand within it. Each command must fit the allocation. Archive compilation
has a separate contract budget. Scratch storage and memory still need capacity
for the selected roster; declaring more partitions does not provide more resources.
A partition whose exact source-bound inventory assignment contains no cases terminates as
`expected_empty`, with no native process, case events, exit code, or copied workspace.
Executed partitions retain their actual native exit and logs. An empty event stream for a
nonempty assignment still fails; empty planned assignments never replace missing native evidence.
The complete receipt retains the full inventory hash, exact plans, source identity, and union
reconciliation. Consumers must distinguish an `expected_empty` row with a null exit from an
executed row; a zero test count alone is not execution evidence.

CI profile automation:
- Fast: `cargo run -q -p bijux-dna-dev -- tooling run ci-fast`
- Slow: `cargo run -q -p bijux-dna-dev -- tooling run ci-slow`
- Complete: `cargo run -q -p bijux-dna-dev -- tooling run ci-test-all-sharded`

The fast profile reads `configs/ci/fast-groups.json`, runs every dependency-ready group through
the bounded event-driven Rust supervisor, and writes exact group logs plus current, p50, p95,
cache-state, reference-runner, and critical-path timing under `artifacts/ci/`. Warm reference
claims require at least five successful matching observations.

Current internal targets surfaced by help:
- `domain-validate`
- `examples-validate`
- `lint-workspace`
- `lint-governance`
- `_policy-fast`
- `_ci-fast`
- `_ci-slow`

`_python-test` runs the Python benchmark and adapter contracts with the root
`pyproject.toml` configuration. Select an interpreter containing its `checks`
dependency group through `PYTHON`; caches and temporary output use `artifacts/`.

## Local test runtimes

Make prefers executable tools provisioned under
`artifacts/runtime/bcftools/install/bin` and `artifacts/runtime/bash/install/bin`.
Set `BIJUX_TEST_RUNTIME_ROOT` to choose another provisioned runtime root; when
these tools are absent, normal `PATH` lookup applies. Nested Make calls retain
each selected directory once. No system installation or shell configuration is
changed. `PYTHON` defaults to `artifacts/python/checks-environment/bin/python3`
when that environment exists, otherwise `python3`; an explicit `PYTHON` wins.
That interpreter must contain the `checks` dependency group from `pyproject.toml`.

Run `make test-preflight` before a full Rust run. The authoritative native version
is declared in `configs/ci/full-shards.json` and checked against the container
catalog: currently `bcftools 1.20`. An installed `1.21` does not satisfy that pin.
Provide the exact release in the selected runtime directory or on `PATH`, or set
`BIJUX_DNA_RUNNER_BINDINGS` to an absolute checksum-bound runner manifest. The
preflight prints expected and observed versions and retains its dependency
report under `artifacts/ci/full-shards/`. Preflight success certifies runtime
availability only; it does not certify passing tests.

Workflow supervision requires Bash 5.1 or newer. The provisioned Bash directory
also makes that interpreter available to `github-all`; workflow unit tests accept
an explicit `BIJUX_GATE_TEST_BASH`. The full local verification controller binds
the selected interpreter and tools into its lane evidence.

## Distributed continuous integration

The hosted test graph builds all native targets and features once, then shares
the compiled archive with independent hash partitions. `ci_partition_count` in
`configs/ci/full-shards.json` governs the hosted matrix; the ordinary local
five-shard roster remains unchanged. Each consumer owns its checkout, runtime,
extraction and output directories. Consumers do not compile workspace code.

`make ci-test-build` uses the `ci` Cargo profile, which inherits development
assertions and overflow checks while omitting debug symbols and incremental
state. The producer also qualifies doctests against the same build cache.
`make ci-test-shard CI_TEST_PARTITION=hash:1/20` consumes only matching source,
configuration, runtime and compiled inputs. `make ci-test-aggregate` requires the
exact expected case union and terminal outcomes from every admitted partition.
Missing, cancelled, failed, duplicate or foreign-source results fail acceptance.

`make ci-test-python` separately runs every Python lane declared in
`configs/ci/local-test-lanes.json`, retaining collection and execution receipts.
The required hosted `tests / complete` check waits for all native partitions and
Python lanes before aggregating. Artifact names include the workflow attempt;
rerun the whole workflow so a partial rerun cannot mix attempts.

`make ci-test-runtime` provisions the exact checksum-pinned BCFtools source
release under `artifacts/runtime`. Linux CI caches the installation; its observed
version is still checked on every use. Existing local installations remain
supported. This command does not install into system directories.

For `github-all`, the hosted producer, consumer and aggregate steps map to one
complete local `test-all` gate, alongside `ci-test-python`. Runtime provisioning
remains a prerequisite. This preserves local coverage without launching
dependent hosted stages as unrelated parallel commands.

The time budget includes compilation, runtime setup, artifact transfer and final
aggregation. Partition counts express available parallel work, not a promise of
five-minute cold builds or runner availability. Hosted measurements must report
queue time and cold or warm cache state separately.
