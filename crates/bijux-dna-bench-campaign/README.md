# Corpus benchmark campaigns

## Ownership

This crate owns corpus sample selection, stage command planning, bounded scheduling,
resume decisions, report-QC preparation, shared index lifecycle and persisted run
publication. Corpus input and run schemas belong to `bijux-dna-bench-corpus`.

## Boundaries

The crate does not depend on the CLI, API, benchmark umbrella, planners or native
stage engines. Callers resolve configuration and supply execution, workspace path
and clock ports. Filesystem effects are confined to campaign publication and cache
lifecycles; tool success records process behavior, not scientific interchangeability.

## Public API

`run_corpus_campaign` consumes a resolved `CorpusCampaignRequest`, a canonical
`CorpusCampaignContract` and injected ports. It returns the persisted typed run
manifest. `artifacts` exposes bundle discovery and digest observations consumed
by campaign and database composition.

## Architecture

[Architecture](docs/ARCHITECTURE.md) describes admission, planning, scheduling,
resume, QC preparation and publication ownership, including retained limitations.

## Tests

Run `cargo nextest run --config-file configs/rust/nextest.toml -p bijux-dna-bench-campaign --all-targets`.
[Tests](docs/TESTS.md) distinguishes retained semantics, lifecycle regression
coverage and the limits of synthetic execution fixtures.
