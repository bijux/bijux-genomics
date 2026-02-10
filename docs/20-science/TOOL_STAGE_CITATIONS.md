# Tool And Stage Citation Index

Minimal citation index for scientific provenance. Authoritative citation placeholders are in `domain/*/tools/*.yaml` (`citation` field).

## FASTQ
- Trimming/merge: `fastp`, `cutadapt`, `adapterremoval`, `leehom`, `skewer`, `alientrimmer`, `fastx_clipper`.
- Screening/classification: `kraken2`, `krakenuniq`, `bracken`, `centrifuge`, `metaphlan`, `kaiju`.
- QC/validation: `fastqvalidator*`, `fastq-scan`, `seqfu`, `seqkit_stats`, `multiqc`, `fastqc`.

## BAM
- Damage/authenticity: `mapdamage2`, `pydamage`, `damageprofiler`, `ngsbriggs`, `addeam`, `pmdtools`.
- Contamination: `schmutzi`, `verifybamid2`, `contammix`.
- Utilities: `samtools`, `bedtools`, `bamtools`, `mosdepth`.

## Stage-level citation stubs
Each stage should cite method families plus tool-specific papers in future updates; current canonical placeholders live in domain YAML and are validated for presence.
