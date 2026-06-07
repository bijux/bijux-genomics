# Local Benchmark Summary

- Stage count: `51`
- Ready stages: `51`
- Incomplete stages: `0`
- Failed stages: `0`

## Sources

- Fake-run root: `runs/bench/local-fake-runs/stages`
- Stage command manifest: `benchmarks/readiness/local-ready/rendered-stage-commands.json`
- Manifest completion report: `benchmarks/readiness/local-ready/manifest-completion-report.json`
- Output completion report: `benchmarks/readiness/local-ready/output-completion-report.json`
- Runtime metrics report: `benchmarks/readiness/local-ready/runtime-metrics.json`
- Tool comparison template: `benchmarks/readiness/local-ready/tool-comparison-template.tsv`

## Stage Readiness

| Stage | Tool | Readiness Kind | Readiness Status | Runtime (s) | Memory (MB) | Failure Reason |
| --- | --- | --- | --- | ---: | ---: | --- |
| `fastq.index_reference` | `bowtie2_build` | `dry_run` | `ready` | `1.0` | `8192.0` | `not_applicable` |
| `fastq.validate_reads` | `fastqvalidator` | `smoke` | `ready` | `1.0` | `8192.0` | `not_applicable` |
| `fastq.profile_read_lengths` | `seqkit_stats` | `smoke` | `ready` | `1.0` | `2048.0` | `not_applicable` |
| `fastq.detect_adapters` | `fastqc` | `smoke` | `ready` | `1.0` | `8192.0` | `not_applicable` |
| `fastq.detect_duplicates_premerge` | `bijux_dna` | `smoke` | `ready` | `1.0` | `1024.0` | `not_applicable` |
| `fastq.estimate_library_complexity_prealign` | `bijux_dna` | `smoke` | `ready` | `1.0` | `1024.0` | `not_applicable` |
| `fastq.trim_terminal_damage` | `cutadapt` | `smoke` | `ready` | `1.0` | `8192.0` | `not_applicable` |
| `fastq.normalize_primers` | `cutadapt` | `smoke` | `ready` | `1.0` | `8192.0` | `not_applicable` |
| `fastq.trim_polyg_tails` | `fastp` | `smoke` | `ready` | `1.0` | `8192.0` | `not_applicable` |
| `fastq.trim_reads` | `fastp` | `smoke` | `ready` | `1.0` | `8192.0` | `not_applicable` |
| `fastq.filter_reads` | `fastp` | `smoke` | `ready` | `1.0` | `8192.0` | `not_applicable` |
| `fastq.profile_reads` | `seqkit_stats` | `smoke` | `ready` | `1.0` | `2048.0` | `not_applicable` |
| `fastq.deplete_rrna` | `sortmerna` | `dry_or_smoke` | `ready` | `1.0` | `8192.0` | `not_applicable` |
| `fastq.merge_pairs` | `pear` | `smoke` | `ready` | `1.0` | `8192.0` | `not_applicable` |
| `fastq.remove_duplicates` | `clumpify` | `smoke` | `ready` | `1.0` | `8192.0` | `not_applicable` |
| `fastq.filter_low_complexity` | `bbduk` | `smoke` | `ready` | `1.0` | `8192.0` | `not_applicable` |
| `fastq.deplete_host` | `bowtie2` | `dry_or_smoke` | `ready` | `1.0` | `8192.0` | `not_applicable` |
| `fastq.deplete_reference_contaminants` | `bowtie2` | `dry_or_smoke` | `ready` | `1.0` | `8192.0` | `not_applicable` |
| `fastq.correct_errors` | `rcorrector` | `dry_or_smoke` | `ready` | `1.0` | `8192.0` | `not_applicable` |
| `fastq.extract_umis` | `umi_tools` | `smoke` | `ready` | `1.0` | `4096.0` | `not_applicable` |
| `fastq.profile_overrepresented_sequences` | `seqkit` | `smoke` | `ready` | `1.0` | `4096.0` | `not_applicable` |
| `fastq.report_qc` | `multiqc` | `smoke` | `ready` | `1.0` | `4096.0` | `not_applicable` |
| `fastq.remove_chimeras` | `vsearch` | `smoke` | `ready` | `1.0` | `8192.0` | `not_applicable` |
| `fastq.infer_asvs` | `dada2` | `smoke` | `ready` | `1.0` | `16384.0` | `not_applicable` |
| `fastq.cluster_otus` | `vsearch` | `smoke` | `ready` | `1.0` | `8192.0` | `not_applicable` |
| `fastq.normalize_abundance` | `seqkit` | `smoke` | `ready` | `1.0` | `4096.0` | `not_applicable` |
| `fastq.screen_taxonomy` | `kraken2` | `dry_or_smoke` | `ready` | `1.0` | `16384.0` | `not_applicable` |
| `bam.align` | `bowtie2` | `dry_or_smoke` | `ready` | `1.0` | `8192.0` | `not_applicable` |
| `bam.authenticity` | `authenticct` | `smoke` | `ready` | `1.0` | `1024.0` | `not_applicable` |
| `bam.bias_mitigation` | `mapdamage2` | `smoke` | `ready` | `1.0` | `1024.0` | `not_applicable` |
| `bam.complexity` | `preseq` | `smoke` | `ready` | `1.0` | `1024.0` | `not_applicable` |
| `bam.contamination` | `verifybamid2` | `dry_or_smoke` | `ready` | `1.0` | `8192.0` | `not_applicable` |
| `bam.coverage` | `samtools` | `smoke` | `ready` | `1.0` | `1024.0` | `not_applicable` |
| `bam.damage` | `ngsbriggs` | `smoke` | `ready` | `1.0` | `1024.0` | `not_applicable` |
| `bam.duplication_metrics` | `samtools` | `smoke` | `ready` | `1.0` | `1024.0` | `not_applicable` |
| `bam.endogenous_content` | `samtools` | `smoke` | `ready` | `1.0` | `1024.0` | `not_applicable` |
| `bam.filter` | `samtools` | `dry_or_smoke` | `ready` | `1.0` | `1024.0` | `not_applicable` |
| `bam.gc_bias` | `picard` | `smoke` | `ready` | `1.0` | `1024.0` | `not_applicable` |
| `bam.genotyping` | `angsd` | `dry_or_smoke` | `ready` | `1.0` | `8192.0` | `not_applicable` |
| `bam.haplogroups` | `yleaf` | `dry_or_smoke` | `ready` | `1.0` | `8192.0` | `not_applicable` |
| `bam.insert_size` | `picard` | `smoke` | `ready` | `1.0` | `1024.0` | `not_applicable` |
| `bam.kinship` | `king` | `smoke` | `ready` | `1.0` | `1024.0` | `not_applicable` |
| `bam.length_filter` | `samtools` | `smoke` | `ready` | `1.0` | `1024.0` | `not_applicable` |
| `bam.mapping_summary` | `samtools` | `smoke` | `ready` | `1.0` | `1024.0` | `not_applicable` |
| `bam.mapq_filter` | `samtools` | `smoke` | `ready` | `1.0` | `1024.0` | `not_applicable` |
| `bam.markdup` | `samtools` | `smoke` | `ready` | `1.0` | `1024.0` | `not_applicable` |
| `bam.overlap_correction` | `bamutil` | `dry_or_smoke` | `ready` | `1.0` | `1024.0` | `not_applicable` |
| `bam.qc_pre` | `samtools` | `smoke` | `ready` | `1.0` | `1024.0` | `not_applicable` |
| `bam.recalibration` | `gatk` | `smoke` | `ready` | `1.0` | `1024.0` | `not_applicable` |
| `bam.sex` | `rxy` | `smoke` | `ready` | `1.0` | `1024.0` | `not_applicable` |
| `bam.validate` | `samtools` | `smoke` | `ready` | `1.0` | `1024.0` | `not_applicable` |
