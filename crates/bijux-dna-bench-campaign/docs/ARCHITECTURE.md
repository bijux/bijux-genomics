# Corpus campaign boundaries

## Input and output contracts

`request.rs` carries resolved paths, source/provenance identity, resources and
ordered argument defaults. `ports.rs` separates execution, canonical workspace
layout and publication time. The only direct workspace dependency is corpus
admission, with its normalized-input and run-contract features. Native scientific
engines, CLI parsing, configuration discovery and output presentation remain
outside this compilation boundary.

The campaign returns `BenchmarkCorpusRunManifest`. Nonzero tool exits remain
sample observations; command composition prints the summary and maps failures to
its exit contract. Transport errors return errors without replacing a prior run
manifest. The clock is sampled after sample execution.

## Internal layout

- `admission.rs` validates path components and publication ancestry.
- `catalog.rs` owns command/report/resume mappings for supported campaign stages.
- `planning/` separates argument precedence, command projection, typed values and
  persisted manifest fields. Projections consume the canonical stage identities.
- `execution/` owns sample records and the bounded work queue behind an execution port.
- `campaign/` owns admission, sample selection, resume, execution and publication.
- `cache/` owns SortMeRNA cache layout, cloning, warmup and promotion.
- `qc/` owns contributor contracts, upstream report admission, preparation and metadata.
- `artifacts.rs` owns artifact bundle observations shared with database composition.

## Effects and failure behavior

Source metadata and digest admission precede output creation. Input and publication
roots must not overlap in either direction. Sample and bundle identities cannot
escape their directory namespaces; symlinked sample roots are refused before
preparation or resume. Execution receives exact argv, environment and cwd through
the caller's runner adapter. Workers are bounded by pending samples.

Dry-run requests do not resume or reset existing sample reports. Ordinary resume
retains report validation and owned stale-output reset semantics. Cache preparation,
QC materialization and campaign publication are explicit effect owners. Publication
is not a multi-file transaction, and ancestry checks do not provide a filesystem
race sandbox. Callers must control concurrent writers to the admitted output tree.

Process success and parseable reports do not prove scientific backend equivalence.
QC contributor and cache contracts preserve the declared stage/tool identities.
