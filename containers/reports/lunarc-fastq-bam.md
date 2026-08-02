# LUNARC FASTQ-to-BAM validation

This report records a real container execution chain for a 100,000-read-pair
subset of public horse whole-genome run `ERR1021817` against the EquCab3.0
reference. Each run used a checksummed input, immutable Apptainer image, clean
commit-addressed checkout, governed output root, and terminal Slurm accounting.

The machine-readable identities are in
[`lunarc-fastq-bam.json`](lunarc-fastq-bam.json). Raw inputs, images, logs, and
outputs remain under the governed LUNARC study root.

## Execution chain

| Stage | Tool | Run | Slurm job | Elapsed | Principal output |
|---|---|---:|---:|---:|---|
| paired-read trimming | AdapterRemoval 2.3.4 | `run-1002` | `3457168` | 4s | two trimmed FASTQ files |
| reference indexing | BWA 0.7.17 | `run-1003` | `3457577` | 30m01s | five EquCab3.0 index files |
| read alignment | BWA 0.7.17 | `run-1004` | `3457596` | 11s | 60,534,232-byte SAM stream |
| SAM-to-BAM conversion | Samtools 1.20 | `run-1011` | `3457606` | 2s | 15,639,030-byte BAM |
| coordinate sorting | Samtools 1.20 | `run-1013` | `3457614` | 3s | 15,570,479-byte sorted BAM |
| CSI indexing | Samtools 1.20 | `run-1027` | `3457657` | 2s | sorted BAM plus CSI index |
| alignment QC | Qualimap 2.3 | `run-1030` | `3457679` | 10s | metrics text and HTML report |

The long BWA indexing run is a one-time reference preparation cost and is kept
separate from the per-sample alignment duration. Downstream inputs cite the
exact upstream output hashes, so the chain can be followed without inferring
lineage from filenames.

## Claim boundary

This proves runtime interfaces and non-fixture execution for trimming,
reference indexing, alignment, BAM transformation, indexing, and alignment
reporting. It does not yet prove read-retention thresholds, mapping quality,
endogenous fraction, duplication, coverage suitability, cross-aligner
agreement, benchmark variance, security acceptance, or release qualification.

The corpus is a modern horse WGS subset selected for fast contract validation.
It is not evidence for ancient-DNA damage, authentication, contamination, or
population-genetic claims. Those tools require an admitted ancient corpus and
predeclared scientific acceptance criteria.
