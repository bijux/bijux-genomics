# Tool And Stage Citation Index

## What
Minimal citation index for scientific provenance.

## Why
Keeps citation guidance visible while preserving `domain/*/tools/*.yaml` as the canonical metadata source.

## Non-goals
- Replacing domain YAML citation fields.
- Providing exhaustive literature reviews.

## Contracts
- Canonical citation metadata lives in `domain/*/tools/*.yaml` under the `citation` field.
- Stage-level citation policy is enforced by repository policy tests.

## FASTQ
- Trimming/merge: `fastp`, `cutadapt`, `adapterremoval`, `leehom`, `skewer`, `alientrimmer`, `fastx_clipper`.
- Screening/classification: `kraken2`, `krakenuniq`, `bracken`, `centrifuge`, `metaphlan`, `kaiju`.
- QC/validation: `fastqvalidator*`, `fastq-scan`, `seqfu`, `seqkit_stats`, `multiqc`, `fastqc`.

## BAM
- Damage/authenticity: `mapdamage2`, `pydamage`, `damageprofiler`, `ngsbriggs`, `addeam`, `pmdtools`.
- Contamination: `schmutzi`, `verifybamid2`, `contammix`.
- Utilities: `samtools`, `bedtools`, `bamtools`, `mosdepth`.

## Stage-level citation guidance
Each stage should cite method families plus tool-specific papers. Domain YAML remains the canonical source for citation metadata.

## Examples
- `bam.damage` references method-level damage models and tool-level parsers (`mapdamage2`, `pydamage`).
- `fastq.trim` references adapter/quality trimming methods and tool-specific defaults provenance.

## Failure modes
- Missing citations in domain YAML cause provenance gaps and policy failures.
- Divergent citations between docs and YAML create review ambiguity.
