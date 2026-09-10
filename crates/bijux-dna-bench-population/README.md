# Population benchmarks

## Ownership

PCA, admixture, population structure, ROH, IBD and demography local benchmarks
own their report schemas, input admission, normalized observations and publication here.

## Boundaries

This crate does not route CLI commands or acquire cross-domain readiness.

The CLI acquires declaration observations and presents reports. Scientific execution
belongs to `bijux-dna-stages-vcf`; corpus admission and persisted stage results
belong to their dedicated owners. No dependency leads back to CLI, API or benchmark aggregation.

## Public API

[Services](docs/PUBLIC_API.md) accept an explicit repository root, requested tool
and admitted `PopulationBenchmarkInputs`.

## Architecture

[Architecture](docs/ARCHITECTURE.md) separates declaration admission, execution,
observations, report contracts and artifact ownership.

## Tests

[Verification](docs/TESTS.md) covers original parser and workflow contracts alongside
publication safety and declaration admission.
