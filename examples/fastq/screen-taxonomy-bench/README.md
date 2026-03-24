# fastq_screen_taxonomy_bench

## Purpose
Run a deterministic FASTQ taxonomic screening benchmark and preserve benchmark evidence for local and HPC comparison.

Canonical invocation: `cargo run -q -p bijux-dev-dna -- examples run run fastq_screen_taxonomy_bench`

## Stage
- Stage ID: `fastq.screen_taxonomy`
- Domain family: `fastq`

## Inputs
- Mini corpus FASTQ from `corpus-01-mini`
- Taxonomic screening backends selected from the stage registry
- Identical input hash across all candidate tools

## Outputs
- `plan.json`
- `explain.json`
- `report.json`
- stage-local `screen_report.tsv`, `classification.report.json`, `metrics.json`, `bench.jsonl`, and `bench.sqlite`

## Acceptance Criteria
- Each tool run records the same input hash and platform contract
- Benchmark output includes structured contamination summary and contamination rate
- The stage remains report-only and does not mutate FASTQ payloads

## HPC Run
- Preferred command:
  `cargo run -q -p bijux-dna bench fastq screen-taxonomy --sample-id screen-taxonomy-hpc --r1 <reads.fastq.gz> --out <bench-dir> --tools auto --replicates 3 --jobs 8`
- Keep taxonomy database selection pinned at the scheduler or image level so cross-tool comparisons stay interpretable.
