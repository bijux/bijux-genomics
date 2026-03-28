# fastq_trim_polyg_tails_bench

## Purpose
Run a deterministic FASTQ polyG/polyX tail trimming benchmark flow for two-color and related chemistries.

Canonical invocation: `cargo run -q -p bijux-dna-dev -- examples run run fastq_trim_polyg_tails_bench`

## Step 1 Containers
- Ensure image resolution is complete before execution.
- Keep the polyG trimming tool surface pinned for reproducible comparisons.

## Step 2 Build/Verify
- Validate the example contract and corpus availability.
- Confirm the selected polyX preset matches the sequencing chemistry under test.

## Step 3 Bench
- Execute `fastq.trim_polyg_tails` with benchmark persistence enabled.
- Emit stage metrics, stage report, `bench.jsonl`, and `bench.sqlite` outputs.

## Step 4 Collect/Report
- Collect outputs under artifacts/examples/fastq_trim_polyg_tails_bench/.
- Review trimmed-length deltas and retained-read summaries before scaling to cluster runs.

## HPC Run
- Preferred command:
  `cargo run -q -p bijux-dna bench fastq trim-polyg-tails --sample-id trim-polyg-hpc --r1 <reads_R1.fastq.gz> --r2 <reads_R2.fastq.gz> --out <bench-dir> --tools auto --replicates 3 --jobs 8 --polyx-preset illumina_twocolor`
- Keep chemistry-specific presets explicit in scheduler submissions so benchmark comparisons remain interpretable.
- Single-end datasets may omit `--r2`; paired-end datasets should pass both mates so trim deltas reflect the full library.
