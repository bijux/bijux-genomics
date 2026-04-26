# Boundary

Owner: `bijux-dna-bench`

This crate owns benchmark suite loading, summarization, comparison, gate
evaluation, deterministic benchmark artifact writing, and read-only loading of
finished runtime/analyze artifacts.

## Allowed Inputs

- Benchmark suite TOML files from `crates/bijux-dna-bench/bench/suites/`.
- `BenchmarkObservation` streams from governed runtime/analyze outputs.
- `summary.json`, `decision.json`, `decisions.json`, and `observations.jsonl`
  fixtures or prior benchmark artifacts.
- Run manifests, metric payloads, and run metadata that have already been
  materialized by lower-level crates.

## Allowed Effects

- Read checked-in suite files and declared benchmark input artifacts.
- Write deterministic benchmark artifacts under caller-declared benchmark output
  roots.
- Resolve repository-local benchmark data and suite directories.
- Canonicalize JSON before writing stable artifact files.

## Forbidden Effects

- Planning workflows.
- Executing tools, runners, containers, or product pipelines.
- Opening network connections.
- Writing outside declared benchmark output roots.
- Mutating checked-in suite catalogs during normal benchmark operations.
- Re-exporting planner, runner, API, or domain execution crates as benchmark
  public API.

## Dependency Direction

Allowed normal dependency families:

- `bijux-dna-bench-model` for benchmark contracts, summaries, policies, and
  statistics.
- `bijux-dna-core` for canonical JSON and run artifact contracts.
- `bijux-dna-infra` for repository path and directory helpers.
- `bijux-dna-runtime` for atomic artifact writes and runtime report contracts.

Analyze, policy, domain, and testkit crates belong in dev-dependencies unless
production source imports their contracts.

## Determinism

For the same suite, observations, policy, and options, outputs must have stable
ordering, stable numeric decisions, and stable serialized JSON shape.

## Legacy Policy

Legacy benchmark artifacts may remain as fixtures for historical comparison, but
new behavior must use the current suite catalog and v1 benchmark model
contracts. Do not add new legacy schema shapes.
