# example-201

`example-201` is BAM stage-series example 01 (`bam-bench-stage-01`) and is fully self-contained.

## Purpose
Validate BAM stage-01 benchmarking on HPC with explicit BAM prerequisites and deterministic artifacts.

## Stage
- Stage ID: `bam.align`
- Series mapping: `example-201` => BAM stage-01

## Corpus Binding Decision
This example uses ENA selection/fetch and then runs BAM stage-01 via pipeline mode (ENA -> FASTQ -> BAM stage path), with corpus rooted at `example-201`.

## BAM Prerequisites
- Reference bank available and indexed for aligner use.
- Mapping preset pinned by benchmark suite/tool defaults.
- Runtime images available under configured HPC container root.

## Expected BAM Artifacts
- aligned BAM
- BAM index (`.bai`)
- `flagstat` summary
- `idxstats` summary

## Acceptance Criteria
- `bijux dna example validate example-201` passes
- `golden/plan.json` is deterministic for `bijux dna example plan example-201`
- `golden/explain.json` exists and matches stage/suite semantics
- `bench-suite.toml` pins exactly BAM stage-01 tool probes

## How To Run On HPC
```bash
bijux dna example run example-201 --hpc
```
