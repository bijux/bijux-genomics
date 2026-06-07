# Benchmark Readiness

This directory stores tracked deterministic benchmark readiness proof, including governed `.json`,
`.tsv`, and report artifacts that must survive deleting disposable local roots such as `target/`.

Disposable run products do not belong here. Local-ready outputs, smoke runs, fake runs, and SLURM
dry runs stay under repository-owned disposable roots until they are intentionally snapshotted.
