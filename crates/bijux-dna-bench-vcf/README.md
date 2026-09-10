# VCF benchmark adapters

## Ownership

This crate owns VCF benchmark tool-family adapter reports, command/output
contracts and local fixture observations. Catalog and supported binding inputs
are supplied explicitly. Reference FASTA materialization is an injected effect;
the composition owner selects its implementation.

## Boundaries

The normal dependency graph does not include CLI, API, benchmark aggregation,
BAM or FASTQ domain libraries. The shared benchmark contract dependency disables
its default FASTQ feature. Fixture effects write only caller-selected locations
or governed local artifact paths. Panel markers remain metadata fixtures.

## Public API

`inputs::VcfAdapterInputs` admits catalog and binding observations before effects.
`adapters` exposes bcftools, ANGSD, PLINK, phasing, imputation, EIGENSOFT and descent
families. [Public API](docs/PUBLIC_API.md) describes inputs and returned reports.

## Architecture

Family modules separate wire models, registry acquisition, scientific command
contracts, input/output admission, fixture preparation and report publication.
See [architecture](docs/ARCHITECTURE.md) for dependency direction and limitations.

## Tests

`tests/contracts.rs` retains original adapter observations; `tests/boundaries.rs`
checks admitted input and dependency behavior. [Tests](docs/TESTS.md) lists
execution commands and what the fixtures establish.
