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

`ScreenTaxonomyReportV1::derived_read_counts` returns classified and unclassified read-count
estimates from the report's existing fields. It prefers `reads_in` to `reads_out` and the
unclassified fraction to the classified fraction, rounds and bounds the selected count, then
derives its saturating complement. Missing totals or both fractions return `(None, None)`.
This pure method does not add serialized fields or turn screening estimates into measured counts.

The bank selection module exposes `adapter_bank_context_at`, `polyx_bank_context_at`
and `contaminant_bank_context_at` for callers with an explicit repository root.
They reuse the same selection and provenance authority as the existing current-directory
entrypoints. All contaminant reference reads use the supplied root. These functions
perform repository IO without changing process CWD; they do not execute tools or
claim atomic observation across concurrently modified source files.

## Abundance Table Truth

`stages::abundance_normalization::normalize_abundance_table_text` accepts borrowed
TSV text and `AbundanceNormalizationEffectiveParams`, returning
`AbundanceTableNormalization` with output text and `AbundanceTableObservation`.
`observe_normalized_abundance_table_text` observes a normalized table without
rewriting it. Both operations are pure; callers own file acquisition and publication.

Present headers must match the declared columns. Short rows are skipped; malformed
numeric cells become zero. Samples are trimmed and sorted, rows within each sample
retain their input order, and duplicate rows contribute individually. Normalization
uses the declared scale factor only when sample totals are positive and emits six
decimal places. Method labels remain metadata. Observations retain arithmetic
values and non-finite values for existing caller composition admission rather than
silently tightening numeric acceptance.

These contracts preserve the API artifact-composition behavior. The existing
`stages::contract::normalize_abundance` path operation retains its separate
eight-decimal formatting and computation. Neither this ownership separation nor
the local smoke envelope establishes scientific correctness or backend execution.
