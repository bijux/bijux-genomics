# Benchmark Contract

`bijux-dna-bench` consumes benchmark suite specs and benchmark observations, then
emits summaries and gate decisions. Contract structs live in
`bijux-dna-bench-model`; this crate owns loading, summarization, comparison,
gating, and deterministic artifact persistence.

## Inputs

- `BenchmarkSuiteSpec` from checked-in TOML files under `crates/bijux-dna-bench/bench/suites/`.
- `BenchmarkObservation` records from runtime/analyze benchmark runs.
- Typed retained-tool catalog, campaign registry, and admitted stage reports for
  stage-coverage derivation.
- Governed data-index/catalog authorities and exact corpus files for expectation derivation.
- Optional existing `observations.jsonl` when `BenchRunOptions.resume` is true.
- Runtime manifests and metric payloads that have already been produced by other
  crates.

## Outputs

- `BenchmarkSummary` for aggregated benchmark rows.
- `GateDecision` values for policy evaluation.
- `CompareReport` for summary-to-summary comparisons.
- Persisted artifacts documented in `docs/BENCH_FORMAT.md`.
- Reconstructed `BenchmarkAttemptRecordV2` values from immutable event files.
- Versioned stage-coverage JSON, Markdown, and generated JSON Schema publications.
- Immutable exact FASTQ summaries, histograms, row identities, and expectation manifests.

## Required Invariants

- Suite specs must validate against `bijux.bench.suite.v1`.
- Observations must validate before summarization.
- Summary and decision artifacts must validate before being written.
- Output ordering must be deterministic.
- Gate decisions must include rationale and metric evidence.
- Benchmark operations must not execute tools or product workflows.
- Corpus expectation publication must re-hash every source file, derive truth through
  `bijux-dna-science`, reject symlinks/conflicts, and publish its manifest last.
- Permanent attempt identities cannot be reused, and prior transition events
  cannot be replaced.
- Retry initialization must identify an existing terminal attempt with the same
  campaign, run, and tool identity.
- Stage coverage must derive exact retained-tool closure from the environment
  catalog and may mark a stage admitted only from a fully validated passing report
  produced by the required repository commit. Historical reports remain visible as stale.

## Failure Modes

- Invalid suite schema or stage/tool ids.
- Missing or malformed observations.
- Missing required stratification metadata.
- Unknown metrics in gate policies.
- Contract validation failures before artifact persistence.
