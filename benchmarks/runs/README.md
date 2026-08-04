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
campaign for `bam.mapping_summary`.

`fastq_profile_read_lengths.py` defines the exact SeqFu and SeqKit comparison
for `fastq.profile_read_lengths`. It derives independent exact read-length
evidence from the admitted 20-run human FASTQ cohort, emits one governed
specification per retained backend, and refuses benchmark admission unless each
sample's raw `record_id<TAB>length` surface and normalized histogram agree with
the independently decoded expectation.

`fastq_profile_reads.py` defines the complete SeqFu, SeqKit, and SeqKit Stats
comparison for `fastq.profile_reads`. It admits all 20 runs and 30 files from
the real human short-read diversity corpus, resolves the shared stage
parameters, emits one specification per interchangeable tool, and checks each
normalized result against independently catalogued read and base counts. The
versioned normalizer under `benchmarks/execution/` is mounted from the frozen
repository commit during every Slurm execution.

Future modules must be named by stable domain and stage responsibility, not by
delivery order.
