# Benchmark corpora

This directory owns reviewed cohort membership and scientific selection
contracts for real benchmark data. A corpus contract declares why each sample
belongs in a comparison; it does not contain downloaded reads or claim that
the source files have been admitted.

[`human-short-read-diversity.toml`](human-short-read-diversity.toml) selects
twenty public human ENA runs across ancient and modern contexts, single- and
paired-end layouts, and several input-size bands. Its stable `corpus-01`
identifier is retained by existing execution and publication interfaces. The
semantic filename describes the scientific role used by the portfolio.

Downloaded data belongs under the configured study data root. Before a
campaign uses it, the acquisition plan, source checksums, local checksums,
read counts, access and redistribution facts, and known limitations must be
recorded in the governed data catalog.

Every explicitly selected FASTQ collection declares `sample_era` as `ancient`, `modern`, or
`not_applicable`. Acquisition refuses to infer that scientific context from an accession or a
collection name, and resolves each collection independently when a portfolio spans multiple ENA
scientific names.

## Production classes

The production portfolio accepts only `production_real` corpora for comparative evidence and
`qualification_real` corpora for qualification evidence. The typed portfolio validator rejects
`test_fixture`, `toy`, `synthetic`, `smoke`, and `local_development` classes before campaign
alignment or readiness derivation. Corpus class is carried into the governed data audit and
scientific-readiness report.

Fixtures remain in test-owned fixture trees and may exercise parsing, failure handling, and output
contracts. They must not be copied into this production selector namespace, bound to an admitted
portfolio corpus, or used to satisfy a publishable benchmark gate. Existing readiness-era fixture
and local-ready outputs remain quarantined historical material until their owning migration removes
them; they confer no admission state.
