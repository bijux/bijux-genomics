# Scientific benchmark runs

This directory temporarily retains the BAM mapping-summary generator while its
execution material moves into `bijux-dna-bench`. New executable campaign logic
must not be added here. Crate-owned campaign workflows bind stable stage
identities to complete retained-tool closure, governed data and image evidence,
commands, resources, acceptance metrics, and immutable result paths.

A run definition is not scientific evidence. Results become admissible only
after Slurm accounting, output checksums, repeated resource observations,
stage-specific quality checks, and cross-tool tolerances pass the benchmark
evidence validator. Reviewed small reports then belong in
`benchmarks/reports/`.

`bam_mapping_summary.py` defines the real Picard and Samtools qualification
campaign for `bam.mapping_summary`.

`bijux_dna_bench::workflow::fastq_profile_read_lengths` owns the exact SeqFu and
SeqKit comparison for `fastq.profile_read_lengths`. It derives independent exact
read-length evidence from the admitted 20-run human FASTQ cohort and refuses
admission unless every record and normalized histogram agrees with that truth.

`bijux_dna_bench::workflow::fastq_profile_reads` owns the SeqFu, SeqKit, and
SeqKit Stats comparison for `fastq.profile_reads`. It admits all 20 runs and 30
files, derives exact read and base counts from independently decoded FASTQ
histograms, and checks every normalized result against that immutable truth.
The versioned normalizers under `benchmarks/execution/` remain declarative
workers mounted from the frozen repository commit during Slurm execution.

Future modules must be named by stable domain and stage responsibility, not by
delivery order.
