# PIPELINES

Pipeline IDs, profiles, and intended scientific use cases.

## FASTQ
- `fastq-to-fastq__default__v1`: Default FASTQ preprocessing and QC.
- `fastq-to-fastq__minimal__v1`: Minimal FASTQ QC for quick validation.
- `fastq-to-fastq__adna__v1`: aDNA-oriented FASTQ preprocessing defaults.

## BAM
- `bam-to-bam__default__v1`: Default BAM QC and reporting.
- `bam-to-bam__adna_shotgun__v1`: aDNA shotgun BAM processing.
- `bam-to-bam__adna_capture__v1`: aDNA capture BAM processing.

## CROSS
- `fastq-to-bam__default__v1`: FASTQ preprocess → BAM processing (modern defaults).
- `fastq-to-bam__adna_shotgun__v1`: FASTQ preprocess → BAM processing (aDNA shotgun).
