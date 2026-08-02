# LUNARC FASTQ-to-BAM validation

This report records a real container execution chain for a 100,000-read-pair
subset of public horse whole-genome run `ERR1021817` against the EquCab3.0
reference. Each run used a checksummed input, immutable Apptainer image, clean
commit-addressed checkout, governed output root, and terminal Slurm accounting.

The machine-readable identities are in the
[repository evidence index](https://github.com/bijux/bijux-genomics/blob/main/containers/reports/lunarc-fastq-bam.json).
Raw inputs, images, logs, and outputs remain under the governed LUNARC study
root.

## Execution chain

| Stage | Tool | Run | Slurm job | Elapsed | Principal output |
|---|---|---:|---:|---:|---|
| FASTQ profiling | SeqKit 2.8.2 | `run-1071` | `3458024` | 1s | 315-byte sequence statistics table |
| read-length profiling | SeqKit Stats 2.8.2 | `run-1072` | `3458025` | 1s | independently reproduced statistics table |
| FASTQ-to-FASTA conversion | Seqtk 1.4 | `run-1073` | `3458026` | 1s | 16,023,606-byte FASTA stream |
| FASTQ structural validation | fastQValidator 0.1.1 | `run-1077` | `3458062` | 1s | validator success report |
| read-quality and adapter profiling | FastQC 0.12.1 | `run-1076` | `3458057` | 4s | HTML report plus machine-readable archive |
| low-complexity filtering | fastp 0.23.4 | `run-1075` | `3458056` | 2s | filtered FASTQ plus JSON and HTML reports |
| paired-read merging | FLASH2 2.2.00 | `run-1078` | `3458063` | 1s | 774,658-byte merged-read FASTQ |
| adapter trimming | Skewer 0.2.2 | `run-1079` | `3458064` | 1s | 25,992,094-byte trimmed FASTQ stream |
| adapter clipping | FASTX Clipper 0.0.14 | `run-1083` | `3458067` | 3s | 23,421,049-byte trimmed FASTQ stream |
| paired-read preparation | SeqPrep 1.3.2 | `run-1082` | `3458066` | 8s | merged reads plus two unmerged-read streams |
| sequence clustering | VSEARCH 2.29.3 | `run-1084` | `3458068` | 5s | 5,143,891-byte centroid FASTA |
| paired-read trimming | AdapterRemoval 2.3.4 | `run-1002` | `3457168` | 4s | two trimmed FASTQ files |
| reference indexing | BWA 0.7.17 | `run-1003` | `3457577` | 30m01s | five EquCab3.0 index files |
| read alignment | BWA 0.7.17 | `run-1004` | `3457596` | 11s | 60,534,232-byte SAM stream |
| SAM-to-BAM conversion | Samtools 1.20 | `run-1011` | `3457606` | 2s | 15,639,030-byte BAM |
| coordinate sorting | Samtools 1.20 | `run-1013` | `3457614` | 3s | 15,570,479-byte sorted BAM |
| mapping-quality filtering | BamTools 2.5.2 | `run-1086` | `3458069` | 3s | 14,707,241-byte filtered BAM |
| paired-read overlap correction | bamUtil 1.0.15 | `run-1087` | `3458070` | 3s | 15,764,483-byte corrected BAM |
| CSI indexing | Samtools 1.20 | `run-1027` | `3457657` | 2s | sorted BAM plus CSI index |
| alignment QC | Qualimap 2.3 | `run-1030` | `3457679` | 10s | metrics text and HTML report |

The long BWA indexing run is a one-time reference preparation cost and is kept
separate from the per-sample alignment duration. Downstream inputs cite the
exact upstream output hashes, so the chain can be followed without inferring
lineage from filenames.

## Claim boundary

This proves runtime interfaces and non-fixture execution for FASTQ statistics,
FASTQ-to-FASTA conversion, low-complexity filtering, trimming, reference
indexing, alignment, BAM transformation, indexing, and alignment reporting.
The two statistics surfaces reproduced the same governed 315-byte table. That
agreement establishes the container interfaces, not the scientific suitability
of any threshold. The FASTA conversion establishes format transformation, not
read validity. The fastp run exercised its low-complexity filter and produced
machine-readable and human-readable reports, but no biological acceptance
threshold was applied to those observations. FastQC likewise produced its full
quality and adapter report; individual module statuses remain observations
rather than governed pass/fail criteria. fastQValidator accepted the complete
FASTQ stream structurally; this is format evidence, not evidence that the reads
are biologically suitable for a downstream analysis. FLASH2 produced merged
reads from the governed pair, but overlap correctness and agreement with
another merger have not been assessed. Skewer exercised its default adapter
contract on the full read stream; retained-read and adapter-removal thresholds
have not yet been declared or assessed. FASTX Clipper processed a governed
uncompressed derivative of the same public FASTQ because its native parser
rejected the gzip stream in preserved run `run-1080`; this proves the declared
plain-FASTQ interface and makes the compression limitation explicit. SeqPrep
produced merged and unmerged outputs from the governed pair, but overlap and
adapter-removal correctness have not been compared with truth or another
implementation. VSEARCH clustered the public WGS read subset and emitted
centroids; this proves the `fastq.cluster_otus` runtime contract only. The WGS
corpus is not an amplicon community and therefore cannot support OTU accuracy,
ecological interpretation, or clustering-threshold claims.

BamTools applied its declared mapping-quality expression to the governed
coordinate-sorted BAM and emitted a readable non-empty BAM. bamUtil executed
its native `clipOverlap` implementation, compiled with phone-home support
disabled, and emitted an overlap-corrected BAM plus run statistics. These runs
establish the stage interfaces and output integrity. They do not establish the
scientific suitability of MAPQ 20, quantify overlap corrections against known
truth, or prove agreement with an independent implementation.

The evidence does not yet prove read-retention thresholds, mapping quality,
endogenous fraction, duplication, coverage suitability, cross-aligner
agreement, benchmark variance, security acceptance, or release qualification.

The corpus is a modern horse WGS subset selected for fast contract validation.
It is not evidence for ancient-DNA damage, authentication, contamination, or
population-genetic claims. Those tools require an admitted ancient corpus and
predeclared scientific acceptance criteria.
