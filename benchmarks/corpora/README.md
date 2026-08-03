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
