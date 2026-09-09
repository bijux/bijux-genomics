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
- Explicit matrix requests binding repository, campaign, selected domains, data surfaces,
  image surface, and generation identity.
- Governed data-index/catalog authorities and exact corpus files for expectation derivation.
- Typed checksum-pinned acquisition plans for real FASTQ corpus materialization.
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
- Complete stage-specific execution material consumed by the immutable execution-spec builder.
- Source-locked generic case matrices, admitted observation sets, and deterministic case aggregations.
- Catalog-complete benchmark frontiers with every supported stage/tool pair and explicit
  refusal rows for missing stage contracts or tool bindings.
- Versioned matrix appraisals and deterministic hardening queues admitted against generated
  schemas and exact aggregate closure.
- No-clobber governed data catalogs published only after complete source and FASTQ validation.

## Required Invariants

- Suite specs must validate against `bijux.bench.suite.v1`.
- Observations must validate before summarization.
- Summary and decision artifacts must validate before being written.
- Output ordering must be deterministic.
- Gate decisions must include rationale and metric evidence.
- Benchmark operations must not execute tools or product workflows.
- Corpus materialization must delegate streamed source verification to `bijux-dna-db-ena`,
  reject unsafe paths and existing catalog authority, validate every gzip FASTQ record,
  and make all admitted files read-only before publishing the catalog.
- Corpus expectation publication must re-hash every source file, derive truth through
  `bijux-dna-science`, reject symlinks/conflicts, and publish its manifest last.
- Run-2042 material must bind all 30 FASTQ files, 20 summary/histogram pairs,
  data authorities, parameter evidence, three repetitions, and exact acceptance metrics.
- Permanent attempt identities cannot be reused, and prior transition events
  cannot be replaced.
- Retry initialization must identify an existing terminal attempt with the same
  campaign, run, and tool identity.
- Stage coverage must derive exact retained-tool closure from the environment
  catalog and may mark a stage admitted only from a fully validated passing report
  produced by the required repository commit. Historical reports remain visible as stale.
- Matrix derivation must filter only on product support. Artifact delivery and redistribution
  policy cannot remove a supported tool, and missing catalog/domain bindings must fail closed
  in the planned frontier instead of disappearing.
- Appraisal and hardening-queue loading, construction, validation, grouping, ordering, and atomic
  persistence belong to this crate; the CLI may only adapt arguments and render admitted results.

## Failure Modes

- Invalid suite schema or stage/tool ids.
- Missing or malformed observations.
- Missing required stratification metadata.
- Unknown metrics in gate policies.
- Contract validation failures before artifact persistence.

Local stage inventories retain matrix declaration order, domain prefixes, unique stage identities and the governed matrix schema version. Aggregate reports retain selected-domain order and exact count closure. Repository admission and workflow publication use the existing model-owned inventory schemas and validators. A readiness declaration is not evidence of executed tool compatibility.
