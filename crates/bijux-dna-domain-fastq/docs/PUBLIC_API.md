# bijux-dna-domain-fastq Public API

`src/lib.rs` is the public facade. Consumers should use these exports instead of reaching into
private implementation modules.

## Public Modules

- `banks`
- `bench_repository`
- `execution_support`
- `id_catalog`
- `invariants`
- `metrics`
- `observer`
- `params`
- `pipeline_contract`
- `prelude`
- `run`
- `stage_contract`
- `stage_semantics`
- `stage_specs`
- `stages`
- `types`

## Major Export Groups

- Stage and pipeline contracts: stage IDs, stage contracts, contract JSON/hash, canonical ordering,
  pipeline modes, criticality, transitions, and dependency graph assembly.
- Artifacts: typed report and manifest schemas for governed FASTQ stages.
- Banks: adapter, contaminant, and polyX banks, presets, effective selections, and path helpers.
- Params: descriptors, defaults, canonical parsing, and typed effective params.
- Metrics and invariants: QC summaries, classification metrics, invariant specs, thresholds, and
  invariant evaluation.
- Execution support and governance: execution support catalogs, stage/tool governance profiles,
  input-layout filtering, maturity, benchmark readiness, and benchmark corpus-family routing.
- Observer and run helpers: parser contracts, observer specialization, FASTQ input discovery, and
  benchmark corpus descriptors.

## Stability Rules

- New public exports need docs, tests, and a clear contract owner.
- Removing, renaming, or changing public fields is breaking unless compatibility is preserved.
- Internal modules should stay private unless a downstream consumer needs a stable domain contract.

The bank selection module exposes `adapter_bank_context_at`, `polyx_bank_context_at`
and `contaminant_bank_context_at` for callers with an explicit repository root.
They reuse the same selection and provenance authority as the existing current-directory
entrypoints. All contaminant reference reads use the supplied root. These functions
perform repository IO without changing process CWD; they do not execute tools or
claim atomic observation across concurrently modified source files.
