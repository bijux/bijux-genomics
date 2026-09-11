# Governed parser proofs

## Ownership

This crate owns the cross-domain family catalog, parser selection, proof observation,
normalized snapshot projection and aggregate parser report. Materializers supply
explicit proof addresses and observed tool identities through a narrow port.

## Boundaries

It does not discover workspace configuration, build stage plans or execute tools.
The default dependency closure contains infrastructure only; `fastq` and `bam`
select their native parser owners independently. Artifact generation stays with
the domain evidence owners. Parser acceptance is not scientific equivalence.

## Public API

`build_report` admits the entire active family selection before generation.
`materialize_row` supports a declared individual family. `publish_report` checks
aggregate consistency before atomically replacing a report. `catalog`, `reports`,
`model` and `ports` expose the contracts used by application composition.

## Architecture

[Architecture](docs/ARCHITECTURE.md) describes optional compilation boundaries,
materialization ownership and the retained strengths of each schema admission.

## Tests

Run `cargo nextest run --config-file configs/rust/nextest.toml -p bijux-dna-bench-parser-proofs --all-targets`.
[Tests](docs/TESTS.md) covers admission before effects, retained publication,
feature refusal, projection and the composed application's compatibility cases.
