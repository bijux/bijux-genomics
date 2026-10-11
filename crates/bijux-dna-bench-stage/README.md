# Local benchmark stages

## Ownership

Canonical local stage inventories, readiness kinds and portable command manifests,
semantic admission, plan projection, script publication and synthetic output shapes.

## Boundaries

Default builds contain schemas and validators with no workspace-crate dependency.
`repository` enables matrix admission and aggregate publication through infrastructure;
`execution` enables plan projection and command publication through shared stage
contracts and infrastructure. The crate does not import native genomics engines. Filesystem effects are confined
to the optional repository and execution services, using infrastructure publication.

## Public API

Wire records and validators are root exports. `repository` accepts normalized domain
rows and reads FASTQ/BAM matrices. VCF composition supplies its canonical row adapter.
`execution` accepts planner-owned plans and publishes admitted commands and fixture shapes.

## Architecture

[Architecture](docs/ARCHITECTURE.md) describes ownership, features and publication limits.

## Tests

Run `cargo nextest run --config-file configs/rust/nextest.toml -p bijux-dna-bench-stage --all-targets --all-features`.
[Tests](docs/TESTS.md) describes preserved schemas and observable failure regressions.
