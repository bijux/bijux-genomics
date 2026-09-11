# FASTQ benchmark evidence

## Ownership

This crate owns synthetic FASTQ parser probes and typed report admission. Application composition selects governed stage plans; producers consume explicit plans and publish synthetic fixtures.

## Boundaries

Normal dependencies are limited to the FASTQ domain, shared stage contracts and filesystem primitives. This crate does not depend on the CLI, API, planners or other benchmark service crates. Fixture effects use caller-selected roots and atomic per-file publication. Synthetic unchanged/empty outputs establish parser contracts, not backend execution or scientific performance.

## Public API

`probes` accepts stage plans and correction settings; `probes::reports` selects canonical domain parsers. See [public contracts](docs/PUBLIC_API.md).

## Architecture

Input admission, record observation, fixture publication, stage-specific report production and parser selection have separate owners. See [architecture](docs/ARCHITECTURE.md).

## Tests

Run `cargo nextest run -p bijux-dna-bench-fastq --all-targets` with the repository nextest configuration. [Tests](docs/TESTS.md) describes malformed-input, retained-evidence and compression regressions.
