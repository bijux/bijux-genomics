# Variant processing benchmarks

## Ownership

This crate owns complete filtering, damage filtering, postprocessing and cohort
quality benchmark workflows. Each service admits explicit declarations, prepares
inputs, executes a stage, validates scientific observations and publishes reports.

## Boundaries

This crate does not own command parsing, registry acquisition or native backend implementation.

VCF scientific execution belongs to `bijux-dna-stages-vcf`; sample-header
observations belong to `bijux-dna-domain-vcf`. File ownership and immutable
execution publication belong to `bijux-dna-bench-artifacts`. This crate does not
depend on the CLI, API, benchmark umbrella or VCF adapter catalog.

## Public API

[Services](docs/PUBLIC_API.md) accept `VariantBenchmarkInputs`, a repository path
and a tool/report-format selection. Existing serialized report fields are retained.

## Architecture

[Architecture](docs/ARCHITECTURE.md) separates admitted inputs, native execution,
scientific records, report formats and publication. Filesystem effects use
`bijux-dna-infra` through the relevant owner. Native checksums and execution paths
remain immutable while public report paths use stable command addresses.

## Tests

[Verification](docs/TESTS.md) covers original scientific contracts, explicit input
admission, output ownership, native checksum retention and QC execution identity.
