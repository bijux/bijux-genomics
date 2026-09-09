# bijux-dna-bench-model Boundary Contract

Owner: benchmark model contracts and persisted decision records.

## Owns

- Benchmark model records: suites, observations, summaries, decisions, graph
  nodes, stages, datasets, parameter bindings, and support requirements.
- Permanent benchmark run and attempt identities, append-only lifecycle
  transitions, retry lineage, Slurm job references, terminal accounting, and
  checksummed portable evidence references.
- Versioned non-barrier campaign-cycle preparation and admission outcomes,
  including generated schemas and pure state/payload consistency validation.
- Versioned normalized-result records and execution-metrics envelopes, including
  deterministic schema generation and payload validation.
- The pure registry that maps persisted benchmark schema renderers to their
  checked-in repository paths; drift checking and writes remain test/workflow effects.
- Schema ids and validation entrypoints for suite, observation, summary, and
  gate decision records.
- Deterministic summary comparison reports.
- Gate policy configuration, per-stage overrides, violation records, and
  decision outcome types.
- Robust statistics, seeded bootstrap confidence intervals, and MAD outlier
  detection.
- Stable public exports through `src/public_api/stable_surface.rs`.

## Does Not Own

- Benchmark execution, process management, or runner backends.
- CLI commands, API route handlers, product orchestration, or persistence.
- Filesystem writes, report rendering, artifact publication, or network access.
- Domain-specific stage implementation beyond validating supported ids and
  contract shapes.
- Scientific acceptance evaluation and metric-direction resolution.

## Allowed Inputs

- Typed benchmark payloads built from trusted callers or fixtures.
- Stable suite, stage, dataset, tool, metric, and parameter ids.
- Explicit seeds for bootstrap operations.
- Policy thresholds, regression windows, required metrics, and per-stage
  overrides.

## Forbidden Dependencies

This crate must not depend on downstream orchestration or presentation layers:

- `bijux-dna-api`
- `bijux-dna-bench`
- `bijux-dna-runner`
- runtime or product execution crates
- CLI adapter crates
- report writer or artifact persistence crates

Allowed workspace dependencies must remain model-facing: core ids, stage
contracts, domain ids used by validation, analyze metric semantics, policies in
tests, and testkit fixtures.

## Effects

All crate-owned operations are pure in-memory computation. The only randomness
is `stats::bootstrap_ci`, and callers must provide the seed. Tests, examples,
and local commands may read fixtures through the Rust test harness, but library
code must not perform hidden I/O.

## Validation

Run from the repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo check -p bijux-dna-bench-model --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench-model --test boundaries --no-default-features
```

Stage-scoring policy types and `validate_stage_scoring_config` belong to this pure owner. Admission checks the persisted version, scientific and operational row contracts, finite non-negative weights, and finite completion requirements. The repository caller supplies and validates active-stage inventory and source-path authority; the model performs no discovery or file IO.

FASTQ tool scoring accepts `FastqToolScoringInput` and returns typed rows through `build_fastq_tool_score_rows`. Evidence selection, component normalization, weighted decisions and report counts are pure model policy. Repository loading and raw-evidence interpretation remain with the caller; partial metric views intentionally project larger report documents. Output wire fields and scoring formulas are preserved. This owner boundary does not establish scientific interchangeability between tools.

Runtime, memory and execution-failure observations in FASTQ scoring match the exact stage/tool binding. A default tool does not inherit observations from another registered or unregistered tool. Missing observations remain absent and reduce metric coverage; any scientific substitution requires an explicit, independently admitted adapter contract.

The full benchmark dashboard envelope and metric rows are owned here. Their closed input shapes and schema identity are independent of CLI acquisition, filesystem persistence, and Markdown presentation. Row cross-checks remain with the report evaluator.
