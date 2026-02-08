# PHASES

## Pre
Purpose: alignment + basic QC readiness.
Required artifacts: alignments.
Required metrics (examples):
- alignment_rate
- mapq distribution
- idxstats coverage summary

## Core
Purpose: canonical BAM outputs and core QC.
Required artifacts: sorted BAM, BAI, dedup BAM.
Required metrics (examples):
- dup_rate
- coverage (breadth/mean/median)
- complexity estimates

## Downstream
Purpose: interpretive assessments and verdicts.
Required artifacts: interpretive reports.
Required metrics (examples):
- damage
- contamination
- authenticity
- sex inference
- kinship sufficiency

Required metrics are defined in `docs/METRICS.md` and described in `docs/METRICS_GLOSSARY.md`.
