# fastq_trim_reads_bench

## Purpose
Run a deterministic FASTQ trimming benchmark flow for adapter, polyX, and contaminant-aware read trimming.

Canonical invocation: `cargo run -q -p bijux-dna-dev -- examples run run fastq_trim_reads_bench`

## Step 1 Containers
- Ensure image resolution is complete before execution.
- Keep the trim tool surface pinned for reproducible comparisons.

## Step 2 Build/Verify
- Validate the example contract and corpus availability.
- Confirm adapter bank, polyX preset, and contaminant preset governance before running.

## Step 3 Bench
- Execute `fastq.trim_reads` with benchmark persistence enabled.
- Emit stage metrics, stage report, `bench.jsonl`, and `bench.sqlite` outputs.

## Step 4 Collect/Report
- Collect outputs under artifacts/examples/fastq_trim_reads_bench/.
- Use the resulting bench bundle for local review or cluster-side replay.

## HPC Run
- Preferred command:
  `cargo run -q -p bijux-dna bench fastq trim-reads --sample-id trim-reads-hpc --r1 <reads.fastq.gz> --out <bench-dir> --tools auto --replicates 3 --jobs 8 --adapter-bank-preset illumina-default --polyx-preset illumina_twocolor --contaminant-preset illumina_default`
- Increase `--replicates` and `--jobs` at the scheduler layer rather than changing the example contract.
