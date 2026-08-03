# Scientific benchmark runs

This directory owns executable stage benchmark definitions. Each module binds
one stable stage identity to its complete retained-tool closure, governed data
and image evidence, commands, resources, acceptance metrics, and result paths.
It emits replayable pipeline and credential-reference contracts beneath
`STUDY/results/benchmark-specifications/`; it does not store raw results.

A run definition is not scientific evidence. Results become admissible only
after Slurm accounting, output checksums, repeated resource observations,
stage-specific quality checks, and cross-tool tolerances pass the benchmark
evidence validator. Reviewed small reports then belong in
`benchmarks/reports/`.

`bam_mapping_summary.py` defines the real Picard and Samtools qualification
campaign for `bam.mapping_summary`. Future modules must be named by stable
domain and stage responsibility, not by delivery order.
