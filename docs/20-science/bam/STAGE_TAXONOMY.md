# BAM Stage Taxonomy

## What
Defines the stage-grouping vocabulary used by the governed BAM stage catalog.

## Why
Planning, science review, and runtime promotion all depend on a stable statement of what kind of boundary each BAM stage represents.

## Non-goals
- Repeating the full operational contract for each stage.
- Hiding planned stages just because they are not yet runtime-promoted.

## Contracts
- Every [domain/bam/stages/](../../../domain/bam/stages/) entry must appear exactly once here.
- `Status` must mirror the stage manifest so taxonomy does not drift from the governed surface.
- `Phase` explains where the stage sits in the review workflow; `Class` explains what kind of
  boundary it represents alongside [STAGE_CATALOG.md](STAGE_CATALOG.md) and
  [TOOLS_ROSTER.md](TOOLS_ROSTER.md).

| Stage | Phase | Class | Status | Intent |
| --- | --- | --- | --- | --- |
| bam.align | structural intake | report | supported | Observe alignment quality and mapping-rate baselines from governed BAM inputs. |
| bam.validate | structural intake | report | supported | Confirm BAM structural integrity before later interpretation. |
| bam.qc_pre | structural intake | report | planned | Hold baseline pre-filter QC as a planned observational stage. |
| bam.mapping_summary | structural intake | report | supported | Emit compact mapping summaries used by later evidence gates. |
| bam.filter | cleanup | mutation | supported | Apply general BAM cleanup rules with governed retention reporting. |
| bam.mapq_filter | cleanup | mutation | supported | Apply MAPQ-specific gating as a separate provenance boundary. |
| bam.length_filter | cleanup | mutation | supported | Enforce minimum fragment or read-length rules. |
| bam.markdup | cleanup | mutation | planned | Mark duplicates without yet promoting duplicate-mutating workflows as default. |
| bam.duplication_metrics | cleanup | report | supported | Measure duplicate burden without mutating the BAM. |
| bam.complexity | qc expansion | report | planned | Estimate library complexity once promotion evidence is ready. |
| bam.coverage | qc expansion | report | supported | Summarize depth and breadth. |
| bam.insert_size | qc expansion | report | planned | Summarize insert-size distributions for paired-end libraries. |
| bam.gc_bias | qc expansion | report | planned | Report GC/AT dropout and related GC-bias measures. |
| bam.endogenous_content | qc expansion | report | supported | Estimate endogenous content from governed mapping summaries. |
| bam.overlap_correction | cleanup | mutation | planned | Correct overlapping pairs when explicit clipping policy is requested. |
| bam.damage | ancient DNA evidence | report | supported | Profile terminal damage and misincorporation patterns. |
| bam.authenticity | ancient DNA evidence | report | supported | Estimate authenticity using damage-linked evidence. |
| bam.contamination | ancient DNA evidence | report | supported | Estimate contamination against governed reference resources. |
| bam.sex | biological inference | inference | supported | Infer biological sex from BAM-domain evidence. |
| bam.bias_mitigation | policy mediation | mutation | planned | Record explicit bias-mitigation actions rather than observational bias only. |
| bam.recalibration | policy mediation | mutation | supported | Recalibrate base qualities through the governed low-coverage skip contract and owned known-sites inputs. |
| bam.haplogroups | biological inference | inference | supported | Infer haplogroups from governed Y-panel BAM evidence with explicit readiness guardrails. |
| bam.genotyping | biological inference | inference | planned | Summarize genotype calling from BAM evidence without default promotion. |
| bam.kinship | biological inference | inference | supported | Estimate relatedness from BAM-domain evidence. |

## Failure modes
- Missing stage rows or stale statuses create false assumptions about what BAM science and runtime surfaces are actually governed.
