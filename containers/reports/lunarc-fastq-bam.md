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
| FASTQ read profiling | FASTQ Scan 1.0.1 | `run-1118` | `3458168` | 1s | JSON read-length and quality distributions for 100,000 reads |
| read-quality and adapter profiling | FastQC 0.12.1 | `run-1076` | `3458057` | 4s | HTML report plus machine-readable archive |
| low-complexity filtering | fastp 0.23.4 | `run-1075` | `3458056` | 2s | filtered FASTQ plus JSON and HTML reports |
| paired-read adapter trimming | Atropos 1.1.31 | `run-1098` | `3458119` | 7s | two digest-pinned trimmed FASTQ streams |
| paired-read adapter trimming | Cutadapt 4.9 | `run-1109` | `3458151` | 3s | paired outputs identical to Atropos after decompression |
| k-mer error correction | BayesHammer 4.2.0 | `run-1099` | `3458120` | 38s | corrected paired and unpaired FASTQ streams |
| low-complexity filtering | BBDuk 39.06 | `run-1006` | `3457600` | 3s | paired filtered FASTQ streams plus statistics |
| paired-read merging | BBMerge 39.06 | `run-1007` | `3457601` | 5s | merged reads plus two unmerged-read streams |
| duplicate removal | Clumpify 39.06 | `run-1008` | `3457602` | 4s | paired deduplicated FASTQ streams |
| paired-read merging | FLASH2 2.2.00 | `run-1078` | `3458063` | 1s | 774,658-byte merged-read FASTQ |
| adapter trimming | Skewer 0.2.2 | `run-1079` | `3458064` | 1s | 25,992,094-byte trimmed FASTQ stream |
| adapter trimming | AlienTrimmer 3.2 | `run-1091` | `3458077` | 13s | paired trimmed FASTQ streams |
| adapter clipping | FASTX Clipper 0.0.14 | `run-1083` | `3458067` | 3s | 23,421,049-byte trimmed FASTQ stream |
| paired-read preparation | SeqPrep 1.3.2 | `run-1082` | `3458066` | 8s | merged reads plus two unmerged-read streams |
| sequence clustering | VSEARCH 2.29.3 | `run-1084` | `3458068` | 5s | 5,143,891-byte centroid FASTA |
| paired-read trimming | AdapterRemoval 2.3.4 | `run-1002` | `3457168` | 4s | two trimmed FASTQ files |
| viral taxonomy screening | Kraken 2 2.1.3 | `run-1102` | `3458124` | 2s | classifications plus a digest-pinned Kraken report |
| viral species abundance re-estimation | Bracken 2.9 | `run-1104` | `3458126` | 1s | abundance table plus re-estimated Kraken report |
| single-reference taxonomy indexing | Centrifuge 1.0.4 | `run-1105` | `3458127` | 4s | four digest-pinned Centrifuge index shards |
| single-reference taxonomy screening | Centrifuge 1.0.4 | `run-1106` | `3458128` | 2s | classifications plus a digest-pinned Centrifuge report |
| protein database indexing | DIAMOND 2.1.10 | `run-1116` | `3458166` | 1s | 50,744,508-byte EquCab3.0 RefSeq protein database |
| translated-read protein alignment | DIAMOND 2.1.10 | `run-1117` | `3458167` | 9s | 1,127 top protein alignments from 100,000 real horse reads |
| read alignment | Bowtie 2 2.5.4 | `run-1101` | `3458123` | 5s | 47,621,650-byte SAM stream against a real reference segment |
| reference indexing | BWA 0.7.17 | `run-1003` | `3457577` | 30m01s | five EquCab3.0 index files |
| read alignment | BWA 0.7.17 | `run-1004` | `3457596` | 11s | 60,534,232-byte SAM stream |
| SAM-to-BAM conversion | Samtools 1.20 | `run-1011` | `3457606` | 2s | 15,639,030-byte BAM |
| coordinate sorting | Samtools 1.20 | `run-1013` | `3457614` | 3s | 15,570,479-byte sorted BAM |
| mapping-quality filtering | BamTools 2.5.2 | `run-1086` | `3458069` | 3s | 14,707,241-byte filtered BAM |
| paired-read overlap correction | bamUtil 1.0.15 | `run-1087` | `3458070` | 3s | 15,764,483-byte corrected BAM |
| whole-genome coverage summary | Mosdepth 0.3.11 | `run-1088` | `3458072` | 16s | global depth distribution and per-contig summary |
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
are biologically suitable for a downstream analysis. FASTQ Scan consumed all
100,000 first-mate reads through its governed gzip adapter and emitted valid
JSON with 9,686,172 total bases, mean read length 96.862, and mean quality
36.221. The container passed exact version, help, missing-input, output, and
Slurm-accounting checks without runtime network access. Its current domain
binding names `fastq.profile_overrepresented_sequences`, but native FASTQ Scan
does not count sequence identities or report enrichment. This evidence is
therefore deliberately limited to read-length, quality, and readable-input
profiling; it does not support an overrepresented-sequence claim. That binding
must be corrected independently rather than using a successful runtime as
scientific evidence for an output the tool does not produce. FLASH2 produced
merged reads from the governed pair, but overlap correctness and agreement with
another merger have not been assessed. Skewer exercised its default adapter
contract on the full read stream; retained-read and adapter-removal thresholds
have not yet been declared or assessed. AlienTrimmer consumed both digest-pinned
read streams and the governed UniVec adapter sequence, then emitted 8,756,213-
and 9,005,796-byte trimmed outputs. Preserved `run-1089` proved that the
upstream Java CLI returns success for an unknown long option; the production
wrapper now rejects unsupported long options, and `run-1091` reproduced the
same output digests while satisfying negative-input acceptance. This verifies
the paired trimming interface, not adapter-removal sensitivity or a retention
threshold. FASTX Clipper processed a governed
uncompressed derivative of the same public FASTQ because its native parser
rejected the gzip stream in preserved run `run-1080`; this proves the declared
plain-FASTQ interface and makes the compression limitation explicit. SeqPrep
produced merged and unmerged outputs from the governed pair, but overlap and
adapter-removal correctness have not been compared with truth or another
implementation. VSEARCH clustered the public WGS read subset and emitted
centroids; this proves the `fastq.cluster_otus` runtime contract only. The WGS
corpus is not an amplicon community and therefore cannot support OTU accuracy,
ecological interpretation, or clustering-threshold claims.

Atropos processed all 100,000 governed read pairs with explicitly declared
Illumina adapters, reported adapters in 2,677 first reads and 2,705 second
reads, and retained every pair. The two compressed outputs passed gzip
integrity checks and contain 19,217,721 bases in total. Preserved `run-1097`
shows that Atropos rejects a one-thread parallel request; `run-1098` therefore
uses the native single-process default instead of disguising an invalid
resource setting. This establishes the real paired-read trimming interface and
records observed adapter counts. It does not establish trimming sensitivity,
read-retention policy, or superiority over another trimmer.

Cutadapt independently processed the same 100,000 pairs with the same two
adapter sequences, found adapters in 2,677 first reads and 2,705 second reads,
retained every pair, and emitted 19,217,721 bases. After decompression, its R1
and R2 outputs were byte-for-byte identical to Atropos, with SHA-256 identities
`d4754f6e…169d5c` and `16795392…14ed`. This is cross-tool agreement for one
corpus and one declared adapter contract. It does not establish general
trimming sensitivity, specificity, or a universal retention policy.

BayesHammer processed 200,000 reads from the governed pair and reported
45,485 changed bases in 34,235 reads, while 25,427 bases could not be
corrected. Its output contains 99,653 read pairs and 347 unpaired reads, so all
100,000 input pairs are accounted for without pretending the outputs retain a
strictly paired shape. The paired streams passed gzip integrity checks; the
unpaired stream and native corrected-dataset description remain with the raw
run evidence. These values establish real k-mer error-correction execution and
observable output shape. With no known-truth corpus or independent comparison,
they do not establish correction accuracy or a scientific acceptance threshold.

The three BBTools surfaces used independently admitted images and the same
digest-pinned read pair. BBDuk's declared entropy filter removed 396 of
200,000 reads and retained 99,802 pairs. BBMerge joined 218 of 100,000 pairs
and emitted the other 99,782 pairs through its unmerged outputs. Clumpify
reported 58 duplicates and retained 99,971 pairs. These counts reconcile with
the governed output files and establish the filter, merge, and deduplication
interfaces. They are corpus observations, not evidence that entropy 0.5 is an
appropriate biological threshold, that BBMerge recovered every true overlap,
or that every Clumpify duplicate is a library duplicate.

Bowtie 2 consumed the six digest-pinned shards from `run-1100` and aligned all
100,000 public read pairs against a real one-megabase EquCab3.0 segment. It
reported 190 pairs aligned concordantly once, 64 more than once, 30
discordantly, and a 1.05% overall alignment rate. The deliberately narrow
reference makes this a stage and lineage proof, not a whole-genome mapping
benchmark. Its alignment percentage must not be compared with the full-genome
BWA run or interpreted as sample quality.

Kraken 2 classified the 100,000 public read pairs against the official
viral-only database released on 2026-06-26. The database archive and all
unpacked Kraken and Bracken files were verified against the publisher's MD5
manifest and then recorded with SHA-256 identities in the governed data
catalog. The run classified one sequence to *Gemykibivirus rhina2* and left
99,999 sequences unclassified. Bracken consumed that exact report and the
matching 100-base k-mer distribution, retaining the single species-level
assignment without redistributing additional reads. Preserved `run-1103`
exposed that the first Bracken image lacked upstream's required `python`
command; `run-1104` used the corrected image and passed version, help,
invalid-input, real-execution, output, and accounting checks. These results
prove the offline taxonomy and abundance interfaces and their data lineage.
They do not establish a viral infection, contamination source, taxonomic
accuracy, detection threshold, or absence of non-viral organisms. Those claims
would require a representative database, controls, independent confirmation,
and a predeclared scientific acceptance design.

Centrifuge built a four-shard index from the complete public `NC_038500.1`
reference and the publisher-verified taxonomy files already admitted for the
viral Kraken database. The dependent run cited every index shard by digest and
classified eight of 100,000 read pairs to that sole reference. This proves the
real indexing, compressed paired-FASTQ, classification, and reporting
interfaces. The single-reference index was deliberately chosen to make the
runtime evidence small and reproducible; it is not a representative taxonomy
database. Its count must not be compared directly with Kraken's one assigned
sequence because the reference sets and classification models differ, and it
does not establish infection, contamination, sensitivity, or specificity.

DIAMOND was built from the exact checksummed `v2.1.10` source commit and used
without runtime network access. The reference preparation run indexed all
60,750 proteins in the NCBI RefSeq annotation for EquCab3.0 assembly
`GCF_002863925.1`; the governed catalog records the NCBI checksum manifest,
assembly report, source URL, retrieval time, and SHA-256 identities. The
dependent `blastx` run translated 100,000 first-mate reads from public horse
WGS run `ERR1021817` and emitted one top hit for 1,127 distinct queries. Those
alignments had 83.005% mean identity, 28.626 residues mean aligned length, and
52.016 mean bit score. This proves exact-source image construction, real
protein-database creation, compressed FASTQ ingestion, translated alignment,
and immutable lineage across two Slurm jobs. It does not estimate proteome
coverage, gene expression, sample quality, or annotation completeness; most
random WGS reads are non-coding or do not span a sufficiently long coding
segment, and only one mate and one top hit were retained for this bounded
runtime qualification. The reference and download method follow the
[NCBI EquCab3.0 assembly record](https://www.ncbi.nlm.nih.gov/datasets/genome/GCF_002863925.1/)
and [NCBI genome package guidance](https://www.ncbi.nlm.nih.gov/datasets/docs/v2/how-tos/genomes/download-genome/).

BamTools applied its declared mapping-quality expression to the governed
coordinate-sorted BAM and emitted a readable non-empty BAM. bamUtil executed
its native `clipOverlap` implementation, compiled with phone-home support
disabled, and emitted an overlap-corrected BAM plus run statistics. These runs
establish the stage interfaces and output integrity. They do not establish the
scientific suitability of MAPQ 20, quantify overlap corrections against known
truth, or prove agreement with an independent implementation.

Mosdepth consumed the digest-pinned coordinate-sorted BAM and its CSI index,
then emitted a global depth distribution and per-contig summary without
runtime network access. This establishes the real `bam.coverage` execution
surface and output lineage. It does not establish a minimum usable coverage,
uniformity threshold, or agreement with an independent coverage engine.

The evidence does not yet prove read-retention thresholds, mapping quality,
endogenous fraction, duplication, coverage suitability, cross-aligner
agreement, benchmark variance, security acceptance, or release qualification.

The corpus is a modern horse WGS subset selected for fast contract validation.
It is not evidence for ancient-DNA damage, authentication, contamination, or
population-genetic claims. Those tools require an admitted ancient corpus and
predeclared scientific acceptance criteria.
