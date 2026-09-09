# Boundary

Owner: `bijux-dna-bench`

This crate owns benchmark suite loading, generic campaign and case planning,
case-evidence admission and aggregation, summarization, comparison, gate
evaluation, deterministic benchmark artifact writing, and read-only loading of
finished runtime/analyze artifacts. It composes runtime-owned append-only attempt
persistence without defining a parallel state machine.

## Allowed Inputs

- Benchmark suite TOML files from `crates/bijux-dna-bench/bench/suites/`.
- `BenchmarkObservation` streams from governed runtime/analyze outputs.
- `summary.json`, `decision.json`, `decisions.json`, and `observations.jsonl`
  fixtures or prior benchmark artifacts.
- Run manifests, metric payloads, and run metadata that have already been
  materialized by lower-level crates.
- Governed STUDY data-index, catalog-registry, and immutable catalog-snapshot authorities plus
  their exact declared files.

## Allowed Effects

- Read checked-in suite files and declared benchmark input artifacts.
- Write deterministic benchmark artifacts under caller-declared benchmark output
  roots.
- Resolve repository-local benchmark data and suite directories.
- Canonicalize JSON before writing stable artifact files.
- Publish no-clobber scientific expectation artifacts under their catalog digest identity.
- Prepare source-locked stage-specific inputs, commands, parameter evidence, acceptance rules,
  and required-file closures without executing them.
- Inspect caller-declared source files, directories, and index-prefix bundles into
  typed content identities. Compound digests retain sorted logical member paths;
  directory cycles and filesystem failures are rejected before staging.

## Forbidden Effects

- Scheduler submission, status, collection, or cancellation.
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
- `bijux-dna-containers` for exact-commit container catalog evidence identity.
- `bijux-dna-runtime` for atomic artifact writes and runtime report contracts.
- `bijux-dna-science` for independent scientific truth derivation from admitted observations.

Analyze, policy, domain, and testkit crates belong in dev-dependencies unless
production source imports their contracts.

## Determinism

For the same suite, observations, policy, and options, outputs must have stable
ordering, stable numeric decisions, and stable serialized JSON shape.

## Legacy Policy

Legacy benchmark artifacts may remain as fixtures for historical comparison, but
new behavior must use the current suite catalog and v1 benchmark model
contracts. Do not add new legacy schema shapes.

The benchmark model owns local all-domain SLURM script inventory types, aggregate construction and coverage admission. Script writers provide typed entries; model validation is pure and checks the serialized inventory against independently expected job counts. Path admission, script bytes and filesystem effects remain adapter responsibilities.

`load_fastq_tool_score_evidence` keeps observed-report IO in `repo` and format adapters in `artifacts/fastq_score_evidence`; raw JSON remains confined to those owners. The loader returns typed model evidence for pure scoring. Missing optional reports contribute no observations, malformed reports and overflowing case totals reject, and authored fixture-reference names never substitute for observed tool outcomes.

`workflow/readiness` evaluates a borrowed `FullBenchmarkDashboardEvidence` view. Coverage counts and cross-report consistency belong to this pure service; the CLI acquires reports and adapts paths and output. Markdown rendering consumes the evaluated report without IO. Readiness coverage alone does not establish backend execution or scientific validity.

## Container qualification capability

`container-qualification` enables composition of admitted benchmark, container,
security and release facts through the policy and science owners. Repository
acquisition is separate from pure assessment and publication decisions. The API
and developer commands enable this capability explicitly; ordinary benchmark
consumers do not gain a mandatory policy-library dependency. Policy guardrails
retain an independent development dependency, so testing a minimal benchmark
build does not require enabling the production qualification capability.
