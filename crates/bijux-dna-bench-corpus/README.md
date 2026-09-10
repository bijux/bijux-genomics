# Benchmark corpus admission

## Ownership

This library owns local corpus manifest admission and typed observations of
reads, alignments, references, cohorts and truth tables. Its effects are restricted to explicit repository and manifest paths. The optional
`panel-metadata` feature resolves reference authority and creates metadata-only
markers under an explicit fixture root; it never replaces reference asset bytes. It does not acquire
datasets, execute tools, schedule campaigns, print output or aggregate benchmarks.

## Public API

The `fastq`, `amplicon`, `edna` and `damage` modules are available by default.
`alignment` enables BAM observations; `variants` enables VCF observations.
`panel-metadata` independently enables reference metadata and its bounded writer.
The CLI selects all three features. Other consumers can select one native format
without compiling the other. Services return typed reports and validation errors.
See [public API](docs/PUBLIC_API.md) for the family entrypoints.

## Architecture

Each input family separates wire models, manifest admission, file observations
and scientific truth evaluation. The library delegates native interpretation to
the corresponding domain crate. [Architecture](docs/ARCHITECTURE.md) describes
ownership and the scientific limits of the retained contracts.

## Tests

`tests/contracts.rs` collects the six input families into one integration target;
`tests/boundaries.rs` checks dependency direction and optional native features.
See [test execution](docs/TESTS.md) for commands and coverage.
