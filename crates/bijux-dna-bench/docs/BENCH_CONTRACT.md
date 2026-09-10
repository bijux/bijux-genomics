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

## All-domain local simulation

The simulation service consumes canonical expected-result, rendered-command and output-declaration rows. The model admits their unique identities and output paths before the repository writes any artifacts. The workflow owns orchestration; artifact rendering records illustrative commands without executing them. CLI adapters acquire the current readiness rows and present the report.

`local-all-domain-fake-runs.schema.json` and `local-all-domain-fake-run-metrics.schema.json` are generated from the benchmark model. Existing wire version identifiers and artifact names remain stable. These outputs demonstrate local orchestration only: zero-length binary placeholders and simulated elapsed time do not certify scientific validity, real resource use or tool interchangeability. Byte counts describe produced artifact bytes and exclude unrelated files from earlier runs.

Output admission rejects duplicate results, contradictory row identities, colliding result directories, parent traversal and existing symlink destinations before writes. Concurrent hostile filesystem namespace replacement and atomic publication of an entire multi-file run are not provided by this service.

Campaign copy-back manifests have a closed versioned model, pure entry/path projection, repository publication and a campaign workflow. The CLI adapts requests only. All planned jobs appear in `entries`; the legacy `suggested_copy_command` illustrates the first job only and never executes a transfer. Slurm script filenames and shell quoting share one pure authority with submission.

Scope coverage and command audit wire records belong to `bench-model/model/readiness/coverage`. The VCF scope contract owns normalized imputation parser/schema identity and exact postprocess binding coverage across parser, output, expected-result, report-map, local-job and command declarations. These declarations do not establish successful tool execution or scientific interchangeability. CLI producers still acquire their upstream observations and render TSV/JSON.

Recorded command audits consume a typed command report in the model policy, publish diagnostics through the benchmark repository, and return failing findings after publication. CLI code acquires the upstream command report and adapts the request. Shell text inspection is a lexical diagnostic, not a shell interpreter, permission check, successful execution record or scientific compatibility proof. Empty or duplicate observations and missing command steps fail admission.

Operational binding coverage consumes borrowed expected results, stage/tool rows, rendered commands, output declarations, parser records and resource hints in the model policy. It retains exact five-field binding identities and explicit missing-producer failures. Fake-run parser evidence remains local simulation coverage; it is not proof of backend execution. The CLI owns acquisition until those workflows are separately extracted.

Command artifact publication admits intrinsic row identities in the model, renders shell and argv JSONL in the artifact owner, and checks both repository destinations before creating or replacing files. Ancestors and destinations must have ordinary directory/file types; symbolic links and traversal are refused. Each file is atomic individually; publication is not a two-file transaction and does not protect against concurrent hostile namespace replacement. Full governed-slice coverage remains the upstream collector contract.
