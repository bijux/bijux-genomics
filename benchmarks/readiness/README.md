# Benchmark Readiness

This directory is a quarantined legacy readiness surface. Its tracked `.json`,
`.tsv`, Markdown, rendered commands, fixtures, and local proof preserve migration
evidence, but none of them can admit a benchmark campaign, satisfy scientific
readiness, or override crate-owned publications under `benchmarks/reports/`.

The crate-owned publication registry records the tracked file count and a
content-addressed tree digest for this directory. New production lifecycle,
admission, aggregation, evidence-book, or replay authority must not be added
here. Durable functionality belongs in `bijux-dna-bench-model`,
`bijux-dna-bench`, `bijux-dna-runtime`, and `bijux-dna-runner`.

Deterministic local-ready proof now lives under `benchmarks/readiness/local-ready/`.
Micro-benchmark execution proof now lives under `benchmarks/readiness/micro/`.
All-domain retained-tool proof now lives under `benchmarks/readiness/all-domains/`.
VCF retained-binding scope proof now lives under `benchmarks/readiness/vcf/`.
Disposable-root cleanup proof now lives under `benchmarks/readiness/path-cleanup/`.
The removed binding audit now lives at `benchmarks/readiness/removed-from-scope.tsv` and keeps
candidate rows that are intentionally outside the final active benchmark job surface explicit.

Disposable run products do not belong here. Local smoke runs, fake runs, fixture-regeneration
trees, and SLURM dry runs stay under repository-owned disposable roots until they are
intentionally snapshotted.
