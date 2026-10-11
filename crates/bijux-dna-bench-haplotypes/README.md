# Haplotype benchmarks

## Ownership

Reference-panel preparation, phasing, imputation and imputation-quality local
benchmarks own input admission, observed evidence and report publication here.

## Boundaries

This crate does not own command parsing, registry acquisition or native backend implementation.

Native scientific execution belongs to `bijux-dna-stages-vcf`. Reference identity
belongs to `bijux-dna-db-ref`, panel metadata to `bijux-dna-bench-corpus`, and
persisted stage-result schemas to `bijux-dna-bench-model`.

## Public API

[Services](docs/PUBLIC_API.md) consume explicit admitted declarations and tool
bindings. CLI composition acquires these inputs and renders reports.

## Architecture

[Architecture](docs/ARCHITECTURE.md) separates input preparation, native execution,
scientific observations, schema adaptation and filesystem publication. Filesystem
effects use `bijux-dna-infra`; retained native artifacts preserve checksum-linked
execution paths while public reports expose stable output addresses.

## Tests

[Verification](docs/TESTS.md) retains scientific fixture and tool-substitution
contracts, alongside input admission and publication regressions.

Immutable execution admission and stable file publication are owned by
`bijux-dna-bench-artifacts`. This crate supplies the existing haplotype inventory
filename/schema and scientific report identity; native paths/checksums and prior
execution retention remain compatible.
