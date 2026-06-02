# BAM Tools Roster

## What
Supported tools for each BAM stage.

## Why
Clarifies tool coverage and rationale.

## Non-goals
- Exhaustive tool survey.

## Contracts
- Every BAM stage must appear exactly once.
- `status` must mirror [domain/bam/stages/](../../../domain/bam/stages/).
- `supported tools` must stay within the governed stage/tool manifests under
  [domain/bam/stages/](../../../domain/bam/stages/) and [domain/bam/tools/](../../../domain/bam/tools/).
- Default-tool rationale stays pinned in
  [../../../domain/bam/docs/DEFAULT_SETTINGS.md](../../../domain/bam/docs/DEFAULT_SETTINGS.md).

## Examples
- `bam.align` admits `bwa, bowtie2`.
- `bam.markdup` stays visible as `planned` until its runtime surface is promoted.

## Failure modes
- Missing stage rows or stale status values misrepresent the BAM execution surface.

| Stage | Status | Supported tools | Rationale |
| --- | --- | --- | --- |
| bam.align | supported | bwa, bowtie2 | Alignment is the admitted mapping boundary before any BAM-level interpretation happens. |
| bam.validate | supported | samtools, bedtools, bamtools | Validation checks structural soundness before downstream metrics are trusted. |
| bam.qc_pre | planned | samtools | Baseline pre-filter QC remains planned and report-only in the current governed surface. |
| bam.mapping_summary | supported | samtools | Mapping summaries remain a compact observational baseline for downstream gates. |
| bam.filter | supported | samtools, bedtools, bamtools | Read-retention filtering keeps the admissible BAM cleanup surface explicit. |
| bam.mapq_filter | supported | samtools, bamtools | MAPQ gating stays separate from broader filtering to keep provenance legible. |
| bam.length_filter | supported | samtools, picard | Fragment-length gating has its own admitted boundary and tool choices. |
| bam.markdup | planned | picard, samtools | Duplicate marking stays planned until the governed runtime/reporting contract is promoted. |
| bam.duplication_metrics | supported | samtools, picard | Duplicate-rate reporting is supported even while full markdup mutation remains planned. |
| bam.complexity | planned | preseq | Library-complexity extrapolation remains planned until promotion evidence closes. |
| bam.coverage | supported | mosdepth, samtools, bedtools | Coverage reporting keeps low-overhead, depth-derived, and interval-coverage summaries visible inside one governed contract. |
| bam.insert_size | planned | picard | Insert-size reporting remains planned alongside other QC-expansion stages. |
| bam.gc_bias | planned | picard | GC-bias analysis stays planned until the broader QC bundle is promoted. |
| bam.endogenous_content | supported | samtools | Endogenous-content estimation currently resolves through governed samtools summaries. |
| bam.overlap_correction | planned | bamutil | Overlap correction remains planned until post-correction comparability is governed. |
| bam.damage | supported | mapdamage2, pydamage, damageprofiler, ngsbriggs, addeam, pmdtools | Damage profiling admits several aDNA-oriented backends for comparable report generation. |
| bam.authenticity | supported | authenticct, pmdtools, damageprofiler | Authenticity scoring stays distinct from general damage profiling while sharing some evidence tools. |
| bam.contamination | supported | schmutzi, verifybamid2, contammix | Contamination estimation requires multiple model families for method comparison. |
| bam.sex | supported | rxy, yleaf, angsd | Sex inference admits ratio-, haplogroup-, and GL-aware backends. |
| bam.bias_mitigation | planned | samtools | Bias-mitigation remains planned until its mutation/reporting contract is promoted. |
| bam.recalibration | planned | gatk | BQSR remains visible as a planned stage, not an admitted default surface. |
| bam.haplogroups | planned | yleaf | Haplogroup inference stays planned until reference and scientific acceptance are closed. |
| bam.genotyping | planned | gatk | BAM-driven genotype summaries remain planned in the current pre-HPC surface. |
| bam.kinship | supported | king, angsd | Kinship inference admits pairwise and GL-aware method families. |
