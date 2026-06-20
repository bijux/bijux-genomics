# Micro Benchmark Report

- Result rows: 77
- Complete rows: 68
- Failed rows: 0
- Missing rows: 0
- Unavailable rows: 9
- Insufficient-data rows: 0
- Runtime rows: 77
- Memory-source rows: 77
- Science-threshold rows: 72

## Complete

| Execution ID | Component | Domain | Stage | Tool | Metrics | Outputs | Logs | Reason |
| --- | --- | --- | --- | --- | ---: | ---: | ---: | --- |
| bam.align | adna_micro_pipeline | bam | bam.align | bowtie2 | 2 | 6 | 0 | aligned duplicate-aware aDNA-like reads against the governed ancient-DNA reference |
| bam.authenticity | adna_micro_pipeline | bam | bam.authenticity | pmdtools | 3 | 2 | 0 | composed authenticity advisory evidence from damage-bearing aligned molecules |
| bam.coverage | adna_micro_pipeline | bam | bam.coverage | samtools | 2 | 3 | 0 | measured coverage across the governed ancient-DNA reference contigs |
| bam.damage | adna_micro_pipeline | bam | bam.damage | mapdamage2 | 3 | 3 | 0 | derived ancient-DNA terminal damage evidence from the aligned and trimmed read support |
| bam.mapping_summary | adna_micro_pipeline | bam | bam.mapping_summary | samtools | 2 | 2 | 0 | summarized mapping behavior from the aligned aDNA-like BAM |
| bam.validate | adna_micro_pipeline | bam | bam.validate | samtools | 2 | 2 | 0 | validated the aligned BAM against coordinate sort, index, and reference coherence rules |
| fastq.remove_duplicates | adna_micro_pipeline | fastq | fastq.remove_duplicates | bijux | 3 | 6 | 0 | removed exact duplicate aDNA-like read pairs while preserving first-observed order |
| fastq.trim_terminal_damage | adna_micro_pipeline | fastq | fastq.trim_terminal_damage | cutadapt | 3 | 5 | 0 | trimmed one base from each read end while preserving residual terminal damage signal |
| fastq.validate_reads | adna_micro_pipeline | fastq | fastq.validate_reads | fastqvalidator | 2 | 5 | 0 | validated synthetic aDNA-like read pairs |
| vcf.call_pseudohaploid | adna_micro_pipeline | vcf | vcf.call_pseudohaploid | bcftools | 3 | 4 | 0 | called a governed pseudohaploid VCF from the aligned aDNA-like BAM |
| vcf.damage_filter | adna_micro_pipeline | vcf | vcf.damage_filter | bcftools | 3 | 5 | 0 | applied damage-aware proxy filtering to the pseudohaploid VCF |
| vcf.stats | adna_micro_pipeline | vcf | vcf.stats | bcftools | 3 | 3 | 0 | summarized the retained ancient-DNA pseudohaploid calls after damage filtering |
| benchmark.amplicon_corpus_fixture | amplicon_micro_pipeline | benchmark | benchmark.amplicon_corpus_fixture | bijux | 7 | 5 | 0 | validated governed amplicon corpus fixture contract |
| benchmark.amplicon_output_judgment | amplicon_micro_pipeline | benchmark | benchmark.amplicon_output_judgment | bijux | 12 | 2 | 0 | validated amplicon primer, ASV, chimera, OTU, and abundance outputs against governed truth |
| benchmark.amplicon_truth_fixture | amplicon_micro_pipeline | benchmark | benchmark.amplicon_truth_fixture | bijux | 3 | 2 | 0 | validated governed amplicon truth bundle contract |
| fastq.cluster_otus | amplicon_micro_pipeline | fastq | fastq.cluster_otus | vsearch | 4 | 7 | 0 | copied governed OTU-clustering smoke outputs into the amplicon micro pipeline |
| fastq.infer_asvs | amplicon_micro_pipeline | fastq | fastq.infer_asvs | dada2 | 3 | 7 | 0 | copied governed ASV inference smoke outputs into the amplicon micro pipeline |
| fastq.normalize_abundance | amplicon_micro_pipeline | fastq | fastq.normalize_abundance | seqkit | 5 | 4 | 0 | copied governed abundance-normalization smoke outputs into the amplicon micro pipeline |
| fastq.normalize_primers | amplicon_micro_pipeline | fastq | fastq.normalize_primers | cutadapt | 6 | 5 | 0 | copied governed primer-normalization smoke outputs into the amplicon micro pipeline |
| fastq.remove_chimeras | amplicon_micro_pipeline | fastq | fastq.remove_chimeras | vsearch | 3 | 7 | 0 | copied governed chimera-removal smoke outputs into the amplicon micro pipeline |
| bam.complexity | bam_micro_smoke_subset | bam | bam.complexity | preseq | 0 | 1 | 0 | binding `bam.complexity` / `preseq` matches the governed BAM local-smoke contract tool, so the exact tiny-fixture stage smoke wrapper is available on host |
| bam.contamination_sex_haplogroups | bam_micro_smoke_subset | bam | bam.sex | rxy | 0 | 1 | 0 | binding `bam.sex` / `rxy` matches the governed BAM local-smoke contract tool, so the exact tiny-fixture stage smoke wrapper is available on host |
| bam.damage_authenticity | bam_micro_smoke_subset | bam | bam.authenticity | authenticct | 0 | 1 | 0 | binding `bam.authenticity` / `authenticct` matches the governed BAM local-smoke contract tool, so the exact tiny-fixture stage smoke wrapper is available on host |
| bam.duplicate_handling | bam_micro_smoke_subset | bam | bam.markdup | samtools | 0 | 1 | 0 | binding `bam.markdup` / `samtools` matches the governed BAM local-smoke contract tool, so the exact tiny-fixture stage smoke wrapper is available on host |
| bam.filtering | bam_micro_smoke_subset | bam | bam.filter | samtools | 0 | 1 | 0 | binding `bam.filter` / `samtools` matches the governed BAM local-smoke contract tool, so the exact tiny-fixture stage smoke wrapper is available on host |
| bam.insert_size_gc_bias | bam_micro_smoke_subset | bam | bam.insert_size | picard | 0 | 1 | 0 | binding `bam.insert_size` / `picard` matches the governed BAM local-smoke contract tool, so the exact tiny-fixture stage smoke wrapper is available on host |
| bam.overlap_endogenous_content | bam_micro_smoke_subset | bam | bam.overlap_correction | bamutil | 0 | 1 | 0 | binding `bam.overlap_correction` / `bamutil` matches the governed BAM local-smoke contract tool, so the exact tiny-fixture stage smoke wrapper is available on host |
| bam.recalibration_genotyping | bam_micro_smoke_subset | bam | bam.recalibration | gatk | 0 | 1 | 0 | binding `bam.recalibration` / `gatk` matches the governed BAM local-smoke contract tool, so the exact tiny-fixture stage smoke wrapper is available on host |
| bam.validation_core_qc | bam_micro_smoke_subset | bam | bam.validate | samtools | 0 | 1 | 0 | binding `bam.validate` / `samtools` matches the governed BAM local-smoke contract tool, so the exact tiny-fixture stage smoke wrapper is available on host |
| bam.align | core_germline_micro_pipeline | bam | bam.align | bowtie2 | 3 | 7 | 0 | core_germline_pipeline_execution |
| bam.coverage | core_germline_micro_pipeline | bam | bam.coverage | samtools | 3 | 3 | 0 | core_germline_pipeline_execution |
| bam.qc_pre | core_germline_micro_pipeline | bam | bam.qc_pre | samtools | 4 | 2 | 0 | core_germline_pipeline_execution |
| bam.validate | core_germline_micro_pipeline | bam | bam.validate | samtools | 4 | 2 | 0 | core_germline_pipeline_execution |
| fastq.filter_reads | core_germline_micro_pipeline | fastq | fastq.filter_reads | fastp | 5 | 5 | 0 | core_germline_pipeline_execution |
| fastq.profile_reads | core_germline_micro_pipeline | fastq | fastq.profile_reads | seqkit_stats | 4 | 2 | 0 | core_germline_pipeline_execution |
| fastq.trim_reads | core_germline_micro_pipeline | fastq | fastq.trim_reads | fastp | 4 | 5 | 0 | core_germline_pipeline_execution |
| fastq.validate_reads | core_germline_micro_pipeline | fastq | fastq.validate_reads | fastqvalidator | 5 | 5 | 0 | core_germline_pipeline_execution |
| vcf.call | core_germline_micro_pipeline | vcf | vcf.call | bcftools | 5 | 5 | 0 | core_germline_pipeline_execution |
| vcf.filter | core_germline_micro_pipeline | vcf | vcf.filter | bcftools | 3 | 5 | 0 | core_germline_pipeline_execution |
| vcf.qc | core_germline_micro_pipeline | vcf | vcf.qc | plink2 | 4 | 4 | 0 | core_germline_pipeline_execution |
| vcf.stats | core_germline_micro_pipeline | vcf | vcf.stats | bcftools | 5 | 4 | 0 | core_germline_pipeline_execution |
| benchmark.edna_corpus_fixture | edna_micro_pipeline | benchmark | benchmark.edna_corpus_fixture | bijux | 3 | 6 | 0 | validated the governed eDNA corpus fixture and expected taxa table |
| benchmark.taxonomy_database_fixture | edna_micro_pipeline | benchmark | benchmark.taxonomy_database_fixture | bijux | 3 | 8 | 0 | validated the governed local taxonomy database fixture |
| benchmark.taxonomy_output_judgment | edna_micro_pipeline | benchmark | benchmark.taxonomy_output_judgment | bijux | 7 | 2 | 0 | validated expected taxa, unclassified reads, and false-positive absence against governed eDNA truth |
| fastq.screen_taxonomy | edna_micro_pipeline | fastq | fastq.screen_taxonomy | kraken2 | 5 | 7 | 0 | materialized governed classifier reports and unclassified FASTQ outputs for each eDNA sample |
| fastq.validate_reads | edna_micro_pipeline | fastq | fastq.validate_reads | fastqvalidator | 3 | 5 | 0 | validated each governed eDNA FASTQ sample before taxonomy screening |
| fastq.adapter_detection | fastq_micro_smoke_subset | fastq | fastq.detect_adapters | fastqc | 0 | 1 | 0 | binding `fastq.detect_adapters` / `fastqc` is the governed FASTQ default tool and `materialize-stage` writes a real local smoke artifact for that stage, so this family is proved with host-local evidence |
| fastq.amplicon | fastq_micro_smoke_subset | fastq | fastq.normalize_primers | cutadapt | 0 | 1 | 0 | binding `fastq.normalize_primers` / `cutadapt` is the governed FASTQ default tool and `materialize-stage` writes a real local smoke artifact for that stage, so this family is proved with host-local evidence |
| fastq.complexity_correction | fastq_micro_smoke_subset | fastq | fastq.estimate_library_complexity_prealign | bijux_dna | 0 | 1 | 0 | binding `fastq.estimate_library_complexity_prealign` / `bijux_dna` is the governed FASTQ default tool and `materialize-stage` writes a real local smoke artifact for that stage, so this family is proved with host-local evidence |
| fastq.duplicate_handling | fastq_micro_smoke_subset | fastq | fastq.detect_duplicates_premerge | bijux_dna | 0 | 1 | 0 | binding `fastq.detect_duplicates_premerge` / `bijux_dna` is the governed FASTQ default tool and `materialize-stage` writes a real local smoke artifact for that stage, so this family is proved with host-local evidence |
| fastq.filtering | fastq_micro_smoke_subset | fastq | fastq.filter_low_complexity | bbduk | 0 | 1 | 0 | binding `fastq.filter_low_complexity` / `bbduk` is the governed FASTQ default tool and `materialize-stage` writes a real local smoke artifact for that stage, so this family is proved with host-local evidence |
| fastq.merge_umi | fastq_micro_smoke_subset | fastq | fastq.merge_pairs | pear | 0 | 1 | 0 | binding `fastq.merge_pairs` / `pear` is the governed FASTQ default tool and `materialize-stage` writes a real local smoke artifact for that stage, so this family is proved with host-local evidence |
| fastq.qc_reporting | fastq_micro_smoke_subset | fastq | fastq.report_qc | multiqc | 0 | 1 | 0 | binding `fastq.report_qc` / `multiqc` is the governed FASTQ default tool and `materialize-stage` writes a real local smoke artifact for that stage, so this family is proved with host-local evidence |
| fastq.read_profiling | fastq_micro_smoke_subset | fastq | fastq.profile_overrepresented_sequences | fastqc | 0 | 1 | 0 | binding `fastq.profile_overrepresented_sequences` / `fastqc` is the governed FASTQ default tool and `materialize-stage` writes a real local smoke artifact for that stage, so this family is proved with host-local evidence |
| fastq.trimming | fastq_micro_smoke_subset | fastq | fastq.trim_terminal_damage | cutadapt | 0 | 1 | 0 | binding `fastq.trim_terminal_damage` / `cutadapt` is the governed FASTQ default tool and `materialize-stage` writes a real local smoke artifact for that stage, so this family is proved with host-local evidence |
| fastq.validate_reads | fastq_micro_smoke_subset | fastq | fastq.validate_reads | fastqvalidator | 0 | 1 | 0 | binding `fastq.validate_reads` / `fastqvalidator` is the governed FASTQ default tool and `materialize-stage` writes a real local smoke artifact for that stage, so this family is proved with host-local evidence |
| bam.validate | real_smoke_core_subset | bam | bam.validate | samtools | 4 | 1 | 0 | real_smoke_execution |
| bridge:bam-to-vcf.call | real_smoke_core_subset | vcf | vcf.call | bcftools | 5 | 4 | 0 | real_smoke_execution |
| fastq.validate_reads | real_smoke_core_subset | fastq | fastq.validate_reads | fastqc | 3 | 1 | 0 | real_smoke_execution |
| vcf.stats | real_smoke_core_subset | vcf | vcf.stats | bcftools | 7 | 4 | 0 | real_smoke_execution |
| vcf.calling | vcf_micro_smoke_subset | vcf | vcf.call | bcftools | 0 | 1 | 0 | binding `vcf.call` / `bcftools` matches the governed VCF stage-matrix default tool, so the exact tiny-fixture stage smoke wrapper is available on host |
| vcf.descent_and_demography | vcf_micro_smoke_subset | vcf | vcf.ibd | germline | 0 | 1 | 0 | binding `vcf.ibd` / `germline` matches the governed VCF stage-matrix default tool, so the exact tiny-fixture stage smoke wrapper is available on host |
| vcf.imputation | vcf_micro_smoke_subset | vcf | vcf.impute | beagle | 0 | 1 | 0 | binding `vcf.impute` / `beagle` matches the governed VCF stage-matrix default tool, so the exact tiny-fixture stage smoke wrapper is available on host |
| vcf.phasing | vcf_micro_smoke_subset | vcf | vcf.phasing | shapeit5 | 0 | 1 | 0 | binding `vcf.phasing` / `shapeit5` matches the governed VCF stage-matrix default tool, so the exact tiny-fixture stage smoke wrapper is available on host |
| vcf.population_structure | vcf_micro_smoke_subset | vcf | vcf.population_structure | plink2 | 0 | 1 | 0 | binding `vcf.population_structure` / `plink2` matches the governed VCF stage-matrix default tool, so the exact tiny-fixture stage smoke wrapper is available on host |
| vcf.quality_control | vcf_micro_smoke_subset | vcf | vcf.stats | bcftools | 0 | 1 | 0 | binding `vcf.stats` / `bcftools` matches the governed VCF stage-matrix default tool, so the exact tiny-fixture stage smoke wrapper is available on host |
| vcf.reference_panel_preparation | vcf_micro_smoke_subset | vcf | vcf.prepare_reference_panel | bcftools | 0 | 1 | 0 | binding `vcf.prepare_reference_panel` / `bcftools` matches the governed VCF stage-matrix default tool, so the exact tiny-fixture stage smoke wrapper is available on host |
| vcf.variant_curation | vcf_micro_smoke_subset | vcf | vcf.damage_filter | bcftools | 0 | 1 | 0 | binding `vcf.damage_filter` / `bcftools` matches the governed VCF stage-matrix default tool, so the exact tiny-fixture stage smoke wrapper is available on host |

## Failed

| Row ID | Source Surface | Component | Execution ID | Domain | Stage | Tool | Detail |
| --- | --- | --- | --- | --- | --- | --- | --- |

## Missing

| Component | Execution ID | Domain | Stage | Tool | Source Report | Detail |
| --- | --- | --- | --- | --- | --- | --- |

## Unavailable

| Execution ID | Component | Domain | Stage | Tool | Status | Reason |
| --- | --- | --- | --- | --- | --- | --- |
| bam.contamination | adna_micro_pipeline | bam | bam.contamination | verifybamid2 | unavailable | synthetic aDNA micro execution does not claim panel-backed contamination evidence; authenticity stays damage-driven in this run |
| vcf.call_gl | adna_micro_pipeline | vcf | vcf.call_gl | angsd | unavailable | aDNA micro execution chooses the governed pseudohaploid branch; likelihood-bearing calling remains covered by dedicated GL smoke proof |
| vcf.gl_propagation | adna_micro_pipeline | vcf | vcf.gl_propagation | angsd | unavailable | aDNA micro execution does not produce a GL-bearing VCF because the pseudohaploid branch is the active governed path for this summary |
| bam.align | bam_micro_smoke_subset | bam | bam.align | bwa | container_needed | stage `bam.align` keeps governed local-ready plan coverage but no BAM tiny-fixture smoke wrapper, so the governed container smoke wrapper is the available local exercise path for `bam.align` / `bwa` |
| bam.coverage | bam_micro_smoke_subset | bam | bam.coverage | mosdepth | container_needed | binding `bam.coverage` / `mosdepth` does not match the governed BAM local-smoke contract tool `samtools`, so the governed container smoke wrapper is the available local exercise path for `bam.coverage` / `mosdepth` |
| bam.kinship | bam_micro_smoke_subset | bam | bam.kinship | king | container_needed | retained tool `king` has no exact BAM tiny-fixture smoke wrapper, so the governed container smoke wrapper is the available local exercise path for `bam.kinship` / `king` |
| fastq.depletion | fastq_micro_smoke_subset | fastq | fastq.deplete_host | bowtie2 | container_needed | binding `fastq.deplete_host` / `bowtie2` is the governed FASTQ default tool, but the current local path is still planner-backed rather than a real smoke artifact, so the governed container smoke wrapper remains the honest micro-benchmark path |
| fastq.index_reference | fastq_micro_smoke_subset | fastq | fastq.index_reference | bowtie2_build | container_needed | binding `fastq.index_reference` / `bowtie2_build` is the governed FASTQ default tool, but the current local path is still planner-backed rather than a real smoke artifact, so the governed container smoke wrapper remains the honest micro-benchmark path |
| fastq.taxonomy | fastq_micro_smoke_subset | fastq | fastq.screen_taxonomy | kraken2 | container_needed | binding `fastq.screen_taxonomy` / `kraken2` is the governed FASTQ default tool, but the current local path is still planner-backed rather than a real smoke artifact, so the governed container smoke wrapper remains the honest micro-benchmark path |

## Insufficient Data

| Execution ID | Component | Domain | Stage | Tool | Detail |
| --- | --- | --- | --- | --- | --- |

## Runtime

| Execution ID | Component | Domain | Stage | Tool | Status | Elapsed Seconds | Source |
| --- | --- | --- | --- | --- | --- | ---: | --- |
| bam.align | adna_micro_pipeline | bam | bam.align | bowtie2 | succeeded |  | not_available |
| bam.authenticity | adna_micro_pipeline | bam | bam.authenticity | pmdtools | succeeded |  | not_available |
| bam.contamination | adna_micro_pipeline | bam | bam.contamination | verifybamid2 | unavailable |  | not_applicable |
| bam.coverage | adna_micro_pipeline | bam | bam.coverage | samtools | succeeded |  | not_available |
| bam.damage | adna_micro_pipeline | bam | bam.damage | mapdamage2 | succeeded |  | not_available |
| bam.mapping_summary | adna_micro_pipeline | bam | bam.mapping_summary | samtools | succeeded |  | not_available |
| bam.validate | adna_micro_pipeline | bam | bam.validate | samtools | succeeded |  | not_available |
| fastq.remove_duplicates | adna_micro_pipeline | fastq | fastq.remove_duplicates | bijux | succeeded |  | not_available |
| fastq.trim_terminal_damage | adna_micro_pipeline | fastq | fastq.trim_terminal_damage | cutadapt | succeeded |  | not_available |
| fastq.validate_reads | adna_micro_pipeline | fastq | fastq.validate_reads | fastqvalidator | succeeded |  | not_available |
| vcf.call_gl | adna_micro_pipeline | vcf | vcf.call_gl | angsd | unavailable |  | not_applicable |
| vcf.call_pseudohaploid | adna_micro_pipeline | vcf | vcf.call_pseudohaploid | bcftools | succeeded |  | not_available |
| vcf.damage_filter | adna_micro_pipeline | vcf | vcf.damage_filter | bcftools | succeeded |  | not_available |
| vcf.gl_propagation | adna_micro_pipeline | vcf | vcf.gl_propagation | angsd | unavailable |  | not_applicable |
| vcf.stats | adna_micro_pipeline | vcf | vcf.stats | bcftools | succeeded |  | not_available |
| benchmark.amplicon_corpus_fixture | amplicon_micro_pipeline | benchmark | benchmark.amplicon_corpus_fixture | bijux | succeeded |  | not_available |
| benchmark.amplicon_output_judgment | amplicon_micro_pipeline | benchmark | benchmark.amplicon_output_judgment | bijux | succeeded |  | not_available |
| benchmark.amplicon_truth_fixture | amplicon_micro_pipeline | benchmark | benchmark.amplicon_truth_fixture | bijux | succeeded |  | not_available |
| fastq.cluster_otus | amplicon_micro_pipeline | fastq | fastq.cluster_otus | vsearch | succeeded |  | not_available |
| fastq.infer_asvs | amplicon_micro_pipeline | fastq | fastq.infer_asvs | dada2 | succeeded |  | not_available |
| fastq.normalize_abundance | amplicon_micro_pipeline | fastq | fastq.normalize_abundance | seqkit | succeeded |  | not_available |
| fastq.normalize_primers | amplicon_micro_pipeline | fastq | fastq.normalize_primers | cutadapt | succeeded |  | not_available |
| fastq.remove_chimeras | amplicon_micro_pipeline | fastq | fastq.remove_chimeras | vsearch | succeeded |  | not_available |
| bam.align | bam_micro_smoke_subset | bam | bam.align | bwa | container_needed |  | not_applicable |
| bam.complexity | bam_micro_smoke_subset | bam | bam.complexity | preseq | succeeded |  | not_available |
| bam.contamination_sex_haplogroups | bam_micro_smoke_subset | bam | bam.sex | rxy | succeeded |  | not_available |
| bam.coverage | bam_micro_smoke_subset | bam | bam.coverage | mosdepth | container_needed |  | not_applicable |
| bam.damage_authenticity | bam_micro_smoke_subset | bam | bam.authenticity | authenticct | succeeded |  | not_available |
| bam.duplicate_handling | bam_micro_smoke_subset | bam | bam.markdup | samtools | succeeded |  | not_available |
| bam.filtering | bam_micro_smoke_subset | bam | bam.filter | samtools | succeeded |  | not_available |
| bam.insert_size_gc_bias | bam_micro_smoke_subset | bam | bam.insert_size | picard | succeeded |  | not_available |
| bam.kinship | bam_micro_smoke_subset | bam | bam.kinship | king | container_needed |  | not_applicable |
| bam.overlap_endogenous_content | bam_micro_smoke_subset | bam | bam.overlap_correction | bamutil | succeeded |  | not_available |
| bam.recalibration_genotyping | bam_micro_smoke_subset | bam | bam.recalibration | gatk | succeeded |  | not_available |
| bam.validation_core_qc | bam_micro_smoke_subset | bam | bam.validate | samtools | succeeded |  | not_available |
| bam.align | core_germline_micro_pipeline | bam | bam.align | bowtie2 | succeeded |  | not_available |
| bam.coverage | core_germline_micro_pipeline | bam | bam.coverage | samtools | succeeded |  | not_available |
| bam.qc_pre | core_germline_micro_pipeline | bam | bam.qc_pre | samtools | succeeded |  | not_available |
| bam.validate | core_germline_micro_pipeline | bam | bam.validate | samtools | succeeded |  | not_available |
| fastq.filter_reads | core_germline_micro_pipeline | fastq | fastq.filter_reads | fastp | succeeded |  | not_available |
| fastq.profile_reads | core_germline_micro_pipeline | fastq | fastq.profile_reads | seqkit_stats | succeeded |  | not_available |
| fastq.trim_reads | core_germline_micro_pipeline | fastq | fastq.trim_reads | fastp | succeeded |  | not_available |
| fastq.validate_reads | core_germline_micro_pipeline | fastq | fastq.validate_reads | fastqvalidator | succeeded |  | not_available |
| vcf.call | core_germline_micro_pipeline | vcf | vcf.call | bcftools | succeeded |  | not_available |
| vcf.filter | core_germline_micro_pipeline | vcf | vcf.filter | bcftools | succeeded |  | not_available |
| vcf.qc | core_germline_micro_pipeline | vcf | vcf.qc | plink2 | succeeded |  | not_available |
| vcf.stats | core_germline_micro_pipeline | vcf | vcf.stats | bcftools | succeeded |  | not_available |
| benchmark.edna_corpus_fixture | edna_micro_pipeline | benchmark | benchmark.edna_corpus_fixture | bijux | succeeded |  | not_available |
| benchmark.taxonomy_database_fixture | edna_micro_pipeline | benchmark | benchmark.taxonomy_database_fixture | bijux | succeeded |  | not_available |
| benchmark.taxonomy_output_judgment | edna_micro_pipeline | benchmark | benchmark.taxonomy_output_judgment | bijux | succeeded |  | not_available |
| fastq.screen_taxonomy | edna_micro_pipeline | fastq | fastq.screen_taxonomy | kraken2 | succeeded |  | not_available |
| fastq.validate_reads | edna_micro_pipeline | fastq | fastq.validate_reads | fastqvalidator | succeeded |  | not_available |
| fastq.adapter_detection | fastq_micro_smoke_subset | fastq | fastq.detect_adapters | fastqc | succeeded |  | not_available |
| fastq.amplicon | fastq_micro_smoke_subset | fastq | fastq.normalize_primers | cutadapt | succeeded |  | not_available |
| fastq.complexity_correction | fastq_micro_smoke_subset | fastq | fastq.estimate_library_complexity_prealign | bijux_dna | succeeded |  | not_available |
| fastq.depletion | fastq_micro_smoke_subset | fastq | fastq.deplete_host | bowtie2 | container_needed |  | not_applicable |
| fastq.duplicate_handling | fastq_micro_smoke_subset | fastq | fastq.detect_duplicates_premerge | bijux_dna | succeeded |  | not_available |
| fastq.filtering | fastq_micro_smoke_subset | fastq | fastq.filter_low_complexity | bbduk | succeeded |  | not_available |
| fastq.index_reference | fastq_micro_smoke_subset | fastq | fastq.index_reference | bowtie2_build | container_needed |  | not_applicable |
| fastq.merge_umi | fastq_micro_smoke_subset | fastq | fastq.merge_pairs | pear | succeeded |  | not_available |
| fastq.qc_reporting | fastq_micro_smoke_subset | fastq | fastq.report_qc | multiqc | succeeded |  | not_available |
| fastq.read_profiling | fastq_micro_smoke_subset | fastq | fastq.profile_overrepresented_sequences | fastqc | succeeded |  | not_available |
| fastq.taxonomy | fastq_micro_smoke_subset | fastq | fastq.screen_taxonomy | kraken2 | container_needed |  | not_applicable |
| fastq.trimming | fastq_micro_smoke_subset | fastq | fastq.trim_terminal_damage | cutadapt | succeeded |  | not_available |
| fastq.validate_reads | fastq_micro_smoke_subset | fastq | fastq.validate_reads | fastqvalidator | succeeded |  | not_available |
| bam.validate | real_smoke_core_subset | bam | bam.validate | samtools | succeeded |  | not_available |
| bridge:bam-to-vcf.call | real_smoke_core_subset | vcf | vcf.call | bcftools | succeeded |  | governed_runtime |
| fastq.validate_reads | real_smoke_core_subset | fastq | fastq.validate_reads | fastqc | succeeded |  | not_available |
| vcf.stats | real_smoke_core_subset | vcf | vcf.stats | bcftools | succeeded |  | governed_runtime |
| vcf.calling | vcf_micro_smoke_subset | vcf | vcf.call | bcftools | succeeded |  | not_available |
| vcf.descent_and_demography | vcf_micro_smoke_subset | vcf | vcf.ibd | germline | succeeded |  | not_available |
| vcf.imputation | vcf_micro_smoke_subset | vcf | vcf.impute | beagle | succeeded |  | not_available |
| vcf.phasing | vcf_micro_smoke_subset | vcf | vcf.phasing | shapeit5 | succeeded |  | not_available |
| vcf.population_structure | vcf_micro_smoke_subset | vcf | vcf.population_structure | plink2 | succeeded |  | not_available |
| vcf.quality_control | vcf_micro_smoke_subset | vcf | vcf.stats | bcftools | succeeded |  | not_available |
| vcf.reference_panel_preparation | vcf_micro_smoke_subset | vcf | vcf.prepare_reference_panel | bcftools | succeeded |  | not_available |
| vcf.variant_curation | vcf_micro_smoke_subset | vcf | vcf.damage_filter | bcftools | succeeded |  | not_available |

## Memory Sources

| Execution ID | Component | Domain | Stage | Tool | Status | Declared Memory MB | Declared CPU Threads | Observed Memory MB | Observed CPU Threads | Source |
| --- | --- | --- | --- | --- | --- | ---: | ---: | ---: | ---: | --- |
| bam.align | adna_micro_pipeline | bam | bam.align | bowtie2 | succeeded | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam.authenticity | adna_micro_pipeline | bam | bam.authenticity | pmdtools | succeeded | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam.contamination | adna_micro_pipeline | bam | bam.contamination | verifybamid2 | unavailable |  |  |  |  | not_applicable |
| bam.coverage | adna_micro_pipeline | bam | bam.coverage | samtools | succeeded | 1024.000 | 1 |  |  | declared_stage_tool_resource |
| bam.damage | adna_micro_pipeline | bam | bam.damage | mapdamage2 | succeeded | 1024.000 | 1 |  |  | declared_stage_tool_resource |
| bam.mapping_summary | adna_micro_pipeline | bam | bam.mapping_summary | samtools | succeeded | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam.validate | adna_micro_pipeline | bam | bam.validate | samtools | succeeded | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| fastq.remove_duplicates | adna_micro_pipeline | fastq | fastq.remove_duplicates | bijux | succeeded |  |  |  | 1 | evidence_report |
| fastq.trim_terminal_damage | adna_micro_pipeline | fastq | fastq.trim_terminal_damage | cutadapt | succeeded |  |  |  | 1 | evidence_report |
| fastq.validate_reads | adna_micro_pipeline | fastq | fastq.validate_reads | fastqvalidator | succeeded | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| vcf.call_gl | adna_micro_pipeline | vcf | vcf.call_gl | angsd | unavailable |  |  |  |  | not_applicable |
| vcf.call_pseudohaploid | adna_micro_pipeline | vcf | vcf.call_pseudohaploid | bcftools | succeeded | 4096.000 | 2 |  |  | declared_stage_tool_resource |
| vcf.damage_filter | adna_micro_pipeline | vcf | vcf.damage_filter | bcftools | succeeded | 4096.000 | 2 |  |  | declared_stage_tool_resource |
| vcf.gl_propagation | adna_micro_pipeline | vcf | vcf.gl_propagation | angsd | unavailable |  |  |  |  | not_applicable |
| vcf.stats | adna_micro_pipeline | vcf | vcf.stats | bcftools | succeeded | 4096.000 | 2 |  |  | declared_stage_tool_resource |
| benchmark.amplicon_corpus_fixture | amplicon_micro_pipeline | benchmark | benchmark.amplicon_corpus_fixture | bijux | succeeded |  |  |  |  | not_available |
| benchmark.amplicon_output_judgment | amplicon_micro_pipeline | benchmark | benchmark.amplicon_output_judgment | bijux | succeeded |  |  |  |  | not_available |
| benchmark.amplicon_truth_fixture | amplicon_micro_pipeline | benchmark | benchmark.amplicon_truth_fixture | bijux | succeeded |  |  |  |  | not_available |
| fastq.cluster_otus | amplicon_micro_pipeline | fastq | fastq.cluster_otus | vsearch | succeeded | 8192.000 | 2 |  |  | declared_stage_tool_resource |
| fastq.infer_asvs | amplicon_micro_pipeline | fastq | fastq.infer_asvs | dada2 | succeeded | 16384.000 | 4 |  |  | declared_stage_tool_resource |
| fastq.normalize_abundance | amplicon_micro_pipeline | fastq | fastq.normalize_abundance | seqkit | succeeded | 4096.000 | 2 |  |  | declared_stage_tool_resource |
| fastq.normalize_primers | amplicon_micro_pipeline | fastq | fastq.normalize_primers | cutadapt | succeeded | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| fastq.remove_chimeras | amplicon_micro_pipeline | fastq | fastq.remove_chimeras | vsearch | succeeded | 8192.000 | 2 |  |  | declared_stage_tool_resource |
| bam.align | bam_micro_smoke_subset | bam | bam.align | bwa | container_needed |  |  |  |  | not_applicable |
| bam.complexity | bam_micro_smoke_subset | bam | bam.complexity | preseq | succeeded | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam.contamination_sex_haplogroups | bam_micro_smoke_subset | bam | bam.sex | rxy | succeeded | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam.coverage | bam_micro_smoke_subset | bam | bam.coverage | mosdepth | container_needed |  |  |  |  | not_applicable |
| bam.damage_authenticity | bam_micro_smoke_subset | bam | bam.authenticity | authenticct | succeeded | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam.duplicate_handling | bam_micro_smoke_subset | bam | bam.markdup | samtools | succeeded | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam.filtering | bam_micro_smoke_subset | bam | bam.filter | samtools | succeeded | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam.insert_size_gc_bias | bam_micro_smoke_subset | bam | bam.insert_size | picard | succeeded | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam.kinship | bam_micro_smoke_subset | bam | bam.kinship | king | container_needed |  |  |  |  | not_applicable |
| bam.overlap_endogenous_content | bam_micro_smoke_subset | bam | bam.overlap_correction | bamutil | succeeded | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam.recalibration_genotyping | bam_micro_smoke_subset | bam | bam.recalibration | gatk | succeeded | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam.validation_core_qc | bam_micro_smoke_subset | bam | bam.validate | samtools | succeeded | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam.align | core_germline_micro_pipeline | bam | bam.align | bowtie2 | succeeded | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam.coverage | core_germline_micro_pipeline | bam | bam.coverage | samtools | succeeded | 1024.000 | 1 |  |  | declared_stage_tool_resource |
| bam.qc_pre | core_germline_micro_pipeline | bam | bam.qc_pre | samtools | succeeded | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bam.validate | core_germline_micro_pipeline | bam | bam.validate | samtools | succeeded | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| fastq.filter_reads | core_germline_micro_pipeline | fastq | fastq.filter_reads | fastp | succeeded |  |  |  | 1 | evidence_report |
| fastq.profile_reads | core_germline_micro_pipeline | fastq | fastq.profile_reads | seqkit_stats | succeeded | 2048.000 | 2 |  |  | declared_stage_tool_resource |
| fastq.trim_reads | core_germline_micro_pipeline | fastq | fastq.trim_reads | fastp | succeeded |  |  |  | 1 | evidence_report |
| fastq.validate_reads | core_germline_micro_pipeline | fastq | fastq.validate_reads | fastqvalidator | succeeded | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| vcf.call | core_germline_micro_pipeline | vcf | vcf.call | bcftools | succeeded | 4096.000 | 2 |  |  | declared_stage_tool_resource |
| vcf.filter | core_germline_micro_pipeline | vcf | vcf.filter | bcftools | succeeded | 4096.000 | 2 |  |  | declared_stage_tool_resource |
| vcf.qc | core_germline_micro_pipeline | vcf | vcf.qc | plink2 | succeeded | 4096.000 | 2 |  |  | declared_stage_tool_resource |
| vcf.stats | core_germline_micro_pipeline | vcf | vcf.stats | bcftools | succeeded | 4096.000 | 2 |  |  | declared_stage_tool_resource |
| benchmark.edna_corpus_fixture | edna_micro_pipeline | benchmark | benchmark.edna_corpus_fixture | bijux | succeeded |  |  |  |  | not_available |
| benchmark.taxonomy_database_fixture | edna_micro_pipeline | benchmark | benchmark.taxonomy_database_fixture | bijux | succeeded |  |  |  |  | not_available |
| benchmark.taxonomy_output_judgment | edna_micro_pipeline | benchmark | benchmark.taxonomy_output_judgment | bijux | succeeded |  |  |  |  | not_available |
| fastq.screen_taxonomy | edna_micro_pipeline | fastq | fastq.screen_taxonomy | kraken2 | succeeded | 16384.000 | 4 |  |  | declared_stage_tool_resource |
| fastq.validate_reads | edna_micro_pipeline | fastq | fastq.validate_reads | fastqvalidator | succeeded | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| fastq.adapter_detection | fastq_micro_smoke_subset | fastq | fastq.detect_adapters | fastqc | succeeded | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| fastq.amplicon | fastq_micro_smoke_subset | fastq | fastq.normalize_primers | cutadapt | succeeded | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| fastq.complexity_correction | fastq_micro_smoke_subset | fastq | fastq.estimate_library_complexity_prealign | bijux_dna | succeeded | 1024.000 | 1 |  |  | declared_stage_tool_resource |
| fastq.depletion | fastq_micro_smoke_subset | fastq | fastq.deplete_host | bowtie2 | container_needed |  |  |  |  | not_applicable |
| fastq.duplicate_handling | fastq_micro_smoke_subset | fastq | fastq.detect_duplicates_premerge | bijux_dna | succeeded | 1024.000 | 1 |  |  | declared_stage_tool_resource |
| fastq.filtering | fastq_micro_smoke_subset | fastq | fastq.filter_low_complexity | bbduk | succeeded | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| fastq.index_reference | fastq_micro_smoke_subset | fastq | fastq.index_reference | bowtie2_build | container_needed |  |  |  |  | not_applicable |
| fastq.merge_umi | fastq_micro_smoke_subset | fastq | fastq.merge_pairs | pear | succeeded | 8192.000 | 2 |  |  | declared_stage_tool_resource |
| fastq.qc_reporting | fastq_micro_smoke_subset | fastq | fastq.report_qc | multiqc | succeeded | 4096.000 | 2 |  |  | declared_stage_tool_resource |
| fastq.read_profiling | fastq_micro_smoke_subset | fastq | fastq.profile_overrepresented_sequences | fastqc | succeeded | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| fastq.taxonomy | fastq_micro_smoke_subset | fastq | fastq.screen_taxonomy | kraken2 | container_needed |  |  |  |  | not_applicable |
| fastq.trimming | fastq_micro_smoke_subset | fastq | fastq.trim_terminal_damage | cutadapt | succeeded | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| fastq.validate_reads | fastq_micro_smoke_subset | fastq | fastq.validate_reads | fastqvalidator | succeeded | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| bam.validate | real_smoke_core_subset | bam | bam.validate | samtools | succeeded | 2048.000 | 3 |  |  | declared_stage_tool_resource |
| bridge:bam-to-vcf.call | real_smoke_core_subset | vcf | vcf.call | bcftools | succeeded | 4096.000 | 2 |  |  | declared_stage_tool_resource |
| fastq.validate_reads | real_smoke_core_subset | fastq | fastq.validate_reads | fastqc | succeeded | 8192.000 | 4 |  |  | declared_stage_tool_resource |
| vcf.stats | real_smoke_core_subset | vcf | vcf.stats | bcftools | succeeded | 4096.000 | 2 |  |  | declared_stage_tool_resource |
| vcf.calling | vcf_micro_smoke_subset | vcf | vcf.call | bcftools | succeeded | 4096.000 | 2 |  |  | declared_stage_tool_resource |
| vcf.descent_and_demography | vcf_micro_smoke_subset | vcf | vcf.ibd | germline | succeeded |  |  |  |  | not_available |
| vcf.imputation | vcf_micro_smoke_subset | vcf | vcf.impute | beagle | succeeded | 16384.000 | 8 |  |  | declared_stage_tool_resource |
| vcf.phasing | vcf_micro_smoke_subset | vcf | vcf.phasing | shapeit5 | succeeded | 4096.000 | 8 |  |  | declared_stage_tool_resource |
| vcf.population_structure | vcf_micro_smoke_subset | vcf | vcf.population_structure | plink2 | succeeded | 4096.000 | 2 |  |  | declared_stage_tool_resource |
| vcf.quality_control | vcf_micro_smoke_subset | vcf | vcf.stats | bcftools | succeeded | 4096.000 | 2 |  |  | declared_stage_tool_resource |
| vcf.reference_panel_preparation | vcf_micro_smoke_subset | vcf | vcf.prepare_reference_panel | bcftools | succeeded | 4096.000 | 2 |  |  | declared_stage_tool_resource |
| vcf.variant_curation | vcf_micro_smoke_subset | vcf | vcf.damage_filter | bcftools | succeeded | 4096.000 | 2 |  |  | declared_stage_tool_resource |

## Science Thresholds

| Domain | Stage | Metric ID | Metric Name | Unit | Direction | Tolerance Kind | Tolerance | Pass Rule | Insufficiency Behavior | Required | Covered Tools |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| bam | bam.align | alignment_rate | alignment_rate |  | range | relative_fraction | 0.020000 | must_remain_within_reference_range | refuse_stage_comparison | true | bowtie2, bwa |
| bam | bam.align | mapped_reads | mapped_reads |  | range | relative_fraction | 0.020000 | must_remain_within_reference_range | refuse_stage_comparison | true | bowtie2, bwa |
| bam | bam.authenticity | confidence | confidence |  | range | absolute_delta | 0.100000 | must_remain_within_reference_range | warn_and_exclude_stage | true | authenticct, pmdtools |
| bam | bam.authenticity | pmd_like_signal_present | pmd_like_signal_present |  | exact_match_preferred | exact_match | 0.000000 | must_match_reference | warn_and_exclude_stage | true | authenticct, pmdtools |
| bam | bam.authenticity | score | score |  | range | relative_fraction | 0.100000 | must_remain_within_reference_range | warn_and_exclude_stage | true | authenticct, pmdtools |
| bam | bam.authenticity | status | status |  | exact_match_preferred | exact_match | 0.000000 | must_match_reference | warn_and_exclude_stage | true | authenticct, pmdtools |
| bam | bam.contamination | ci_high | ci_high |  | maximum | absolute_delta | 0.020000 | must_not_exceed_reference | warn_and_exclude_stage | true | verifybamid2 |
| bam | bam.contamination | ci_low | ci_low |  | maximum | absolute_delta | 0.020000 | must_not_exceed_reference | warn_and_exclude_stage | true | verifybamid2 |
| bam | bam.contamination | estimate | estimate |  | maximum | absolute_delta | 0.010000 | must_not_exceed_reference | warn_and_exclude_stage | true | verifybamid2 |
| bam | bam.coverage | breadth_1x | breadth_1x |  | range | relative_fraction | 0.020000 | must_remain_within_reference_range | warn_and_exclude_stage | true | mosdepth, samtools |
| bam | bam.coverage | covered_bases | covered_bases |  | range | relative_fraction | 0.020000 | must_remain_within_reference_range | warn_and_exclude_stage | true | mosdepth, samtools |
| bam | bam.coverage | mean_depth | mean_depth |  | range | relative_fraction | 0.100000 | must_remain_within_reference_range | warn_and_exclude_stage | true | mosdepth, samtools |
| bam | bam.damage | damage_signal | damage_signal |  | exact_match_preferred | exact_match | 0.000000 | must_match_reference | warn_and_exclude_stage | true | mapdamage2 |
| bam | bam.damage | terminal_c_to_t_5p | terminal_c_to_t_5p |  | range | absolute_delta | 0.020000 | must_remain_within_reference_range | warn_and_exclude_stage | true | mapdamage2 |
| bam | bam.damage | terminal_g_to_a_3p | terminal_g_to_a_3p |  | range | absolute_delta | 0.020000 | must_remain_within_reference_range | warn_and_exclude_stage | true | mapdamage2 |
| bam | bam.filter | input_reads | input_reads |  | exact_match_preferred | exact_match | 0.000000 | must_match_reference | refuse_stage_comparison | true | samtools |
| bam | bam.filter | kept_reads | kept_reads |  | range | relative_fraction | 0.020000 | must_remain_within_reference_range | refuse_stage_comparison | true | samtools |
| bam | bam.filter | removed_reads | removed_reads |  | range | relative_fraction | 0.020000 | must_remain_within_reference_range | refuse_stage_comparison | true | samtools |
| bam | bam.kinship | observed_max_overlap_snps | observed_max_overlap_snps |  | minimum | absolute_delta | 100.000000 | must_meet_or_exceed_reference | warn_and_exclude_stage | true | king |
| bam | bam.kinship | pair_count | pair_count |  | exact_match_preferred | exact_match | 0.000000 | must_match_reference | refuse_stage_comparison | true | king |
| bam | bam.kinship | pairwise_results | pairwise_results |  | structured_match | normalized_record_overlap | 1.000000 | must_match_reference_structure | warn_and_exclude_stage | true | king |
| bam | bam.kinship | status | status |  | exact_match_preferred | exact_match | 0.000000 | must_match_reference | warn_and_exclude_stage | true | king |
| bam | bam.mapping_summary | mapped_reads | mapped_reads |  | range | relative_fraction | 0.020000 | must_remain_within_reference_range | refuse_stage_comparison | true | samtools |
| bam | bam.mapping_summary | mapping_fraction | mapping_fraction |  | range | relative_fraction | 0.020000 | must_remain_within_reference_range | refuse_stage_comparison | true | samtools |
| bam | bam.mapping_summary | secondary_reads | secondary_reads |  | range | relative_fraction | 0.050000 | must_remain_within_reference_range | refuse_stage_comparison | true | samtools |
| bam | bam.mapping_summary | supplementary_reads | supplementary_reads |  | range | relative_fraction | 0.050000 | must_remain_within_reference_range | refuse_stage_comparison | true | samtools |
| bam | bam.mapping_summary | unmapped_reads | unmapped_reads |  | range | relative_fraction | 0.020000 | must_remain_within_reference_range | refuse_stage_comparison | true | samtools |
| bam | bam.markdup | duplicate_count | duplicate_count |  | range | relative_fraction | 0.050000 | must_remain_within_reference_range | refuse_stage_comparison | true | samtools |
| bam | bam.markdup | duplicate_fraction | duplicate_fraction |  | maximum | absolute_delta | 0.020000 | must_not_exceed_reference | refuse_stage_comparison | true | samtools |
| bam | bam.qc_pre | duplicate_flagged_reads | duplicate_flagged_reads |  | range | relative_fraction | 0.050000 | must_remain_within_reference_range | refuse_stage_comparison | true | samtools |
| bam | bam.qc_pre | mapped_reads | mapped_reads |  | range | relative_fraction | 0.020000 | must_remain_within_reference_range | refuse_stage_comparison | true | samtools |
| bam | bam.qc_pre | total_reads | total_reads |  | exact_match_preferred | exact_match | 0.000000 | must_match_reference | refuse_stage_comparison | true | samtools |
| bam | bam.qc_pre | unmapped_reads | unmapped_reads |  | range | relative_fraction | 0.020000 | must_remain_within_reference_range | refuse_stage_comparison | true | samtools |
| bam | bam.sex | autosomal_coverage | autosomal_coverage |  | range | relative_fraction | 0.100000 | must_remain_within_reference_range | warn_and_exclude_stage | true | rxy |
| bam | bam.sex | call | call |  | exact_match_preferred | exact_match | 0.000000 | must_match_reference | warn_and_exclude_stage | true | rxy |
| bam | bam.sex | confidence | confidence |  | minimum | absolute_delta | 0.100000 | must_meet_or_exceed_reference | warn_and_exclude_stage | true | rxy |
| bam | bam.sex | status | status |  | exact_match_preferred | exact_match | 0.000000 | must_match_reference | warn_and_exclude_stage | true | rxy |
| bam | bam.sex | x_coverage | x_coverage |  | range | relative_fraction | 0.150000 | must_remain_within_reference_range | warn_and_exclude_stage | true | rxy |
| bam | bam.sex | y_coverage | y_coverage |  | range | relative_fraction | 0.150000 | must_remain_within_reference_range | warn_and_exclude_stage | true | rxy |
| bam | bam.validate | validation_errors | validation_errors |  | structured_match | normalized_set_overlap | 1.000000 | must_match_reference_structure | refuse_stage_comparison | true | samtools |
| bam | bam.validate | validation_status | validation_status |  | exact_match_preferred | exact_match | 0.000000 | must_match_reference | refuse_stage_comparison | true | samtools |
| bam | bam.validate | validation_warnings | validation_warnings |  | structured_match | normalized_set_overlap | 1.000000 | must_match_reference_structure | drop_metric_from_stage | true | samtools |
| fastq | fastq.index_reference | index_build_exit_code | index_build_exit_code | exit_code | exact_match_preferred | exact_match | 0.000000 | must_match_reference | refuse_stage_comparison | true | bowtie2_build |
| fastq | fastq.profile_overrepresented_sequences | flagged_sequences | flagged_sequences | sequences | exact_match_preferred | absolute_delta | 1.000000 | must_match_reference | warn_and_exclude_stage | true | fastqc |
| fastq | fastq.profile_overrepresented_sequences | sequence_count | sequence_count | reads | exact_match_preferred | exact_match | 0.000000 | must_match_reference | warn_and_exclude_stage | true | fastqc |
| fastq | fastq.profile_overrepresented_sequences | top_fraction | top_fraction | fraction | exact_match_preferred | absolute_delta | 0.050000 | must_match_reference | warn_and_exclude_stage | true | fastqc |
| fastq | fastq.validate_reads | format_validation_pass_rate | format_validation_pass_rate | fraction | higher_is_better | absolute_delta | 0.010000 | must_meet_or_exceed_reference | refuse_stage_comparison | true | fastqc, fastqvalidator |
| vcf | vcf.call_gl | missing_likelihoods | missing likelihoods | sites | lower_is_better | relative_fraction | 0.050000 | must_not_exceed_reference | refuse_stage_comparison | true | angsd |
| vcf | vcf.call_gl | sites_with_likelihoods | sites with likelihoods | sites | higher_is_better | relative_fraction | 0.050000 | must_meet_or_exceed_reference | refuse_stage_comparison | true | angsd |
| vcf | vcf.call_pseudohaploid | called_sites | called sites | sites | higher_is_better | relative_fraction | 0.050000 | must_meet_or_exceed_reference | refuse_stage_comparison | true | bcftools |
| vcf | vcf.call_pseudohaploid | missing_sites | missing sites | sites | lower_is_better | relative_fraction | 0.050000 | must_not_exceed_reference | refuse_stage_comparison | true | bcftools |
| vcf | vcf.damage_filter | removed_variants | removed variants | variants | exact_match_preferred | exact_match | 0.000000 | must_match_reference | refuse_stage_comparison | true | bcftools |
| vcf | vcf.damage_filter | retained_variants | retained variants | variants | exact_match_preferred | exact_match | 0.000000 | must_match_reference | refuse_stage_comparison | true | bcftools |
| vcf | vcf.damage_filter | terminal_damage_filtered_variants | terminal damage filtered variants | variants | exact_match_preferred | exact_match | 0.000000 | must_match_reference | refuse_stage_comparison | true | bcftools |
| vcf | vcf.gl_propagation | sample_count | sample count | samples | exact_match_preferred | exact_match | 0.000000 | must_match_reference | refuse_stage_comparison | true | angsd |
| vcf | vcf.gl_propagation | site_count_after | site count after propagation | sites | exact_match_preferred | exact_match | 0.000000 | must_match_reference | refuse_stage_comparison | true | angsd |
| vcf | vcf.gl_propagation | site_count_before | site count before propagation | sites | exact_match_preferred | exact_match | 0.000000 | must_match_reference | refuse_stage_comparison | true | angsd |
| vcf | vcf.ibd | pair_count | pair count | pairs | exact_match_preferred | exact_match | 0.000000 | must_match_reference | refuse_stage_comparison | true | germline |
| vcf | vcf.impute | imputed_genotypes | imputed genotypes | genotypes | higher_is_better | relative_fraction | 0.050000 | must_meet_or_exceed_reference | refuse_stage_comparison | true | beagle |
| vcf | vcf.impute | low_confidence_count | low-confidence sites | sites | lower_is_better | relative_fraction | 0.050000 | must_not_exceed_reference | refuse_stage_comparison | true | beagle |
| vcf | vcf.impute | masked_truth_match_count | masked-truth matches | sites | higher_is_better | relative_fraction | 0.050000 | must_meet_or_exceed_reference | refuse_stage_comparison | true | beagle |
| vcf | vcf.impute | missing_after | missing genotypes after imputation | genotypes | lower_is_better | relative_fraction | 0.050000 | must_not_exceed_reference | refuse_stage_comparison | true | beagle |
| vcf | vcf.impute | missing_before | missing genotypes before imputation | genotypes | lower_is_better | relative_fraction | 0.050000 | must_not_exceed_reference | refuse_stage_comparison | true | beagle |
| vcf | vcf.impute | unresolved_count | unresolved sites | sites | lower_is_better | relative_fraction | 0.050000 | must_not_exceed_reference | refuse_stage_comparison | true | beagle |
| vcf | vcf.phasing | phase_block_n50 | phase block n50 | bases | higher_is_better | relative_fraction | 0.150000 | must_meet_or_exceed_reference | refuse_stage_comparison | true | shapeit5 |
| vcf | vcf.phasing | switch_error_proxy | switch error proxy | fraction | lower_is_better | absolute_delta | 0.020000 | must_not_exceed_reference | refuse_stage_comparison | true | shapeit5 |
| vcf | vcf.population_structure | pair_count | pair count | pairs | exact_match_preferred | exact_match | 0.000000 | must_match_reference | refuse_stage_comparison | true | plink2 |
| vcf | vcf.population_structure | sample_count | sample count | samples | exact_match_preferred | exact_match | 0.000000 | must_match_reference | refuse_stage_comparison | true | plink2 |
| vcf | vcf.qc | concordance | concordance | fraction | higher_is_better | absolute_delta | 0.020000 | must_meet_or_exceed_reference | refuse_stage_comparison | true | plink2 |
| vcf | vcf.qc | imputation_info_mean | mean imputation info | score | higher_is_better | absolute_delta | 0.050000 | must_meet_or_exceed_reference | refuse_stage_comparison | true | plink2 |
| vcf | vcf.qc | missingness_post | post-qc missingness | fraction | lower_is_better | absolute_delta | 0.020000 | must_not_exceed_reference | refuse_stage_comparison | true | plink2 |
| vcf | vcf.qc | rsq_mean | mean r-squared | score | higher_is_better | absolute_delta | 0.050000 | must_meet_or_exceed_reference | refuse_stage_comparison | true | plink2 |
