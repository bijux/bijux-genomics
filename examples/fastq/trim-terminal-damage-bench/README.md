# fastq_trim_terminal_damage_bench

## Purpose
Run a deterministic terminal-damage-aware FASTQ trimming benchmark flow for ancient DNA and damage-profile-sensitive datasets.

Canonical invocation: `cargo run -q -p bijux-dna-dev -- examples run run fastq_trim_terminal_damage_bench`

## Step 1 Containers
- Ensure image resolution is complete before execution.
- Keep the terminal-damage trimming tool surface pinned for reproducible comparisons.

## Step 2 Build/Verify
- Validate the example contract and corpus availability.
- Confirm the damage mode and trim-window parameters match the assay and damage model under study.

## Step 3 Bench
- Execute `fastq.trim_terminal_damage` with benchmark persistence enabled.
- Emit stage metrics, stage report, `bench.jsonl`, and `bench.sqlite` outputs.

## Step 4 Collect/Report
- Collect outputs under artifacts/examples/fastq_trim_terminal_damage_bench/.
- Review terminal-damage summaries before scaling to broader cluster experiments.

## HPC Run
- Preferred command:
  `cargo run -q -p bijux-dna bench fastq trim-terminal-damage --sample-id trim-terminal-damage-hpc --r1 <reads_R1.fastq.gz> --r2 <reads_R2.fastq.gz> --out <bench-dir> --tools auto --replicates 3 --jobs 8 --damage-mode ancient --trim-5p-bases 2 --trim-3p-bases 2`
- Keep damage-mode and trim-window choices explicit in scheduler submissions so downstream interpretation stays defensible.
- Single-end datasets may omit `--r2`; paired-end datasets should pass both mates so damage summaries remain mate-aware.
