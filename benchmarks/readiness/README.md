# Benchmark Readiness

This directory stores tracked deterministic benchmark readiness proof, including governed `.json`,
`.tsv`, and report artifacts that must survive deleting disposable local roots such as `target/`.

Deterministic local-ready proof now lives under `benchmarks/readiness/local-ready/`.

Disposable run products do not belong here. Local smoke runs, fake runs, fixture-regeneration
trees, and SLURM dry runs stay under repository-owned disposable roots until they are
intentionally snapshotted.
