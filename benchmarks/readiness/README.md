# Benchmark Readiness

This directory stores tracked deterministic benchmark readiness proof, including governed `.json`,
`.tsv`, and report artifacts that must survive deleting disposable local roots such as `target/`.

Deterministic local-ready proof now lives under `benchmarks/readiness/local-ready/`.
All-domain retained-tool proof now lives under `benchmarks/readiness/all-domains/`.
Disposable-root cleanup proof now lives under `benchmarks/readiness/path-cleanup/`.
The removed binding audit now lives at `benchmarks/readiness/removed-from-scope.tsv` and keeps
candidate rows that are intentionally outside the final active benchmark job surface explicit.

Disposable run products do not belong here. Local smoke runs, fake runs, fixture-regeneration
trees, and SLURM dry runs stay under repository-owned disposable roots until they are
intentionally snapshotted.
