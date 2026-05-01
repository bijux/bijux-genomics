# Tool And Stage Citation Index

## What
Root navigation hub for the governed citation ledgers and upstream evidence maps.

## Why
Keeps citation guidance visible while preserving per-domain ledgers and machine-readable upstream evidence maps as the real authorities.

## Non-goals
- Replacing domain YAML citation fields.
- Providing exhaustive literature reviews.

## Contracts
- Canonical citation metadata lives in `domain/*/tools/*.yaml` under the `citation` field.
- Stage-level citation policy is enforced by repository policy tests.
- Domain science docs remain the readable SSOT for stage-to-tool applicability.
- Upstream paper and repository maps remain the machine-readable SSOT for archive planning.

## FASTQ
- Authority: [FASTQ References](fastq/REFERENCES.md).
- Covers trimming, merge, QC, classifier, database, and reporting surfaces for governed FASTQ stages.

## BAM
- Authority: [BAM References](bam/REFERENCES.md).
- Covers damage, authenticity, contamination, QC, and utility surfaces for governed BAM stages.

## VCF
- Authority: [VCF References](vcf/REFERENCES.md).
- Covers supported calling/filtering plus planned downstream phasing, imputation, structure, IBD, ROH, and demography surfaces.

## Upstream Evidence Maps
- Paper authority map: [TOOL_PAPER_MAP.tsv](../../science/docs/upstream/papers/TOOL_PAPER_MAP.tsv).
- Repository evidence manifest: [MANIFEST.tsv](../../science/docs/upstream/github-repos/MANIFEST.tsv).

## Stage-level citation guidance
Each stage should cite method families plus tool-specific papers. Domain YAML remains the canonical source for citation metadata.

## Examples
- `bam.damage` references method-level damage models and tool-level parsers (`mapdamage2`, `pydamage`).
- `fastq.trim_reads` references adapter/quality trimming methods and tool-specific defaults provenance.
- `vcf.impute` references both the admitted backend family and the pinned panel-backed workflow context.

## Failure modes
- Missing citations in domain YAML cause provenance gaps and policy failures.
- Divergent citations between docs and YAML create review ambiguity.
