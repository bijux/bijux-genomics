# Architecture

`bijux-dna-bench` is a benchmark contract crate. It reads governed benchmark
observations and suite specifications, then produces deterministic summaries,
gate decisions, comparisons, and benchmark artifacts. It does not plan or execute
product workflows.

## Root Layout

- `Cargo.toml` defines the crate dependency graph.
- `README.md` is the only root documentation file.
- `bench/suites/` contains checked-in benchmark suite TOML files.
- `docs/` contains the 10 authoritative crate docs.
- `src/` contains the library implementation.
- `tests/` contains boundary, contract, determinism, semantic, fixture, and
  snapshot tests.

`bench/` is data/catalog ownership, not a documentation directory. Markdown docs
belong under `docs/`.

## Source Layout

- `src/lib.rs` keeps the crate root thin and re-exports `public_api`.
- `src/public_api/` owns the curated stable surface and stable-surface owner.
- `src/workflow/` owns suite loading, summarization, comparison, gate evaluation,
  suite persistence, fairness checks, scope grouping, and summary statistics.
- `src/workflow/run_suite/` separates suite orchestration from artifact
  persistence.
- `src/workflow/summary/` separates grouping, row metrics, and stratum assembly.
- `src/repo/` owns repository-root discovery, workspace path policy, repository
  contracts, run metadata, run artifact loaders, and sqlite run-index queries.
- `src/repo/run_artifacts/` separates manifest, metrics, and observations
  loading.
- `src/repo/sqlite/queries/run_index/` separates run-index queries from
  metadata-path policy.
- `src/artifacts/` owns deterministic artifact serialization.
- `src/artifacts/writer/` separates observation reading, observation writing,
  and structured JSON writing.

## Test Layout

- `tests/boundaries/` protects source, docs, and root layout.
- `tests/contracts/api/` protects the public API and API hygiene.
- `tests/contracts/benching/` protects benchmark contracts, suite catalog rules,
  workspace paths, and ownership.
- `tests/contracts/docs/` protects docs-to-fixture alignment.
- `tests/determinism/` protects stable ordering, comparison output, and realistic
  snapshots.
- `tests/semantics/` protects gate semantics and metric rejection behavior.
- `tests/fixtures/` contains governed benchmark input fixtures.
- `tests/snapshots/` contains governed public API and compare snapshots.

Test documentation lives in `docs/TESTS.md`; README files are intentionally not
allowed below `tests/`.

## Dependency Direction

Allowed normal dependencies are lower-level model, core, analyze, infra, and
runtime contracts required to load finished artifacts and write deterministic
benchmark outputs. `bijux-dna-bench` must not depend on API, planner, runner, or
domain execution crates for production behavior.

## Guardrails

`tests/boundaries/architecture_tree.rs` enforces the root layout, docs allowance,
source tree, test tree, and Markdown location rules. Contract tests enforce the
public API snapshot, no raw JSON outside repo/artifacts, no public API panics,
and suite catalog governance.
