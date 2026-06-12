# Stage-Centric Benchmark Report

## Summary

- Stage count: 51
- Multi-tool stages: 30
- Stage-tool rows: 123
- Benchmark-ready rows: 116
- Blocked rows: 7
- Stages with blockers: 5

| Domain | Stage | Report section | Tools | Ready | Blocked | Shared metrics | Blocked tools |
| --- | --- | --- | ---: | ---: | ---: | --- | --- |
| bam | bam.align | Alignment Intake | 2 | 2 | 0 | mapped_reads, alignment_rate | none |
| bam | bam.validate | Alignment Intake | 3 | 3 | 0 | validation_status, validation_errors, validation_warnings, input_bam_identity | none |
| bam | bam.qc_pre | Alignment Intake | 2 | 2 | 0 | total_reads, mapped_reads, unmapped_reads, duplicate_flagged_reads, contig_summary | none |
| bam | bam.mapping_summary | Alignment Intake | 2 | 2 | 0 | mapping_fraction, mapped_reads, unmapped_reads, secondary_reads, supplementary_reads | none |
| bam | bam.filter | Alignment Refinement | 3 | 3 | 0 | input_reads, kept_reads, removed_reads, active_filters | none |
| bam | bam.mapq_filter | Alignment Refinement | 2 | 2 | 0 | mapq_threshold, kept_reads, removed_reads, filtered_bam | none |
| bam | bam.length_filter | Alignment Refinement | 2 | 2 | 0 | min_length_threshold, kept_reads, removed_reads, filtered_bam | none |
| bam | bam.markdup | Library Complexity | 2 | 2 | 0 | marked_bam, duplicate_metrics, duplicate_count, duplicate_fraction | none |
| bam | bam.duplication_metrics | Library Complexity | 2 | 2 | 0 | duplicate_count, duplicate_fraction, estimated_library_size | none |
| bam | bam.complexity | Library Complexity | 1 | 1 | 0 | not_applicable | none |
| bam | bam.coverage | Coverage and Quality | 3 | 3 | 0 | mean_depth, breadth_1x, covered_bases, observed_region_count, region_ids | none |
| bam | bam.insert_size | Coverage and Quality | 1 | 1 | 0 | not_applicable | none |
| bam | bam.gc_bias | Coverage and Quality | 1 | 1 | 0 | not_applicable | none |
| bam | bam.endogenous_content | Coverage and Quality | 1 | 1 | 0 | not_applicable | none |
| bam | bam.overlap_correction | Alignment Refinement | 1 | 1 | 0 | not_applicable | none |
| bam | bam.damage | Ancient Signal | 6 | 6 | 0 | terminal_c_to_t_5p, terminal_g_to_a_3p, damage_signal, runtime_s, memory_mb | none |
| bam | bam.authenticity | Ancient Signal | 3 | 3 | 0 | score, confidence, pmd_like_signal_present, consumed_metric_ids, missing_metric_ids | none |
| bam | bam.contamination | Ancient Signal | 3 | 3 | 0 | scope, prerequisites_passed, estimate, ci_low, ci_high | none |
| bam | bam.sex | Sample Identity | 3 | 3 | 0 | x_coverage, y_coverage, autosomal_coverage, call, confidence, status | none |
| bam | bam.bias_mitigation | Downstream Readiness | 1 | 1 | 0 | not_applicable | none |
| bam | bam.recalibration | Downstream Readiness | 1 | 1 | 0 | not_applicable | none |
| bam | bam.haplogroups | Sample Identity | 1 | 1 | 0 | not_applicable | none |
| bam | bam.genotyping | Downstream Readiness | 1 | 1 | 0 | not_applicable | none |
| bam | bam.kinship | Sample Identity | 2 | 2 | 0 | observed_max_overlap_snps, pair_count, status, pairwise_results | none |
| fastq | fastq.validate_reads | Input Readiness | 5 | 5 | 0 | format_validation_pass_rate | none |
| fastq | fastq.profile_read_lengths | Quality Profiling | 4 | 4 | 0 | not_declared | none |
| fastq | fastq.detect_adapters | Quality Profiling | 1 | 1 | 0 | not_applicable | none |
| fastq | fastq.detect_duplicates_premerge | Quality Profiling | 1 | 1 | 0 | not_applicable | none |
| fastq | fastq.estimate_library_complexity_prealign | Quality Profiling | 1 | 1 | 0 | not_applicable | none |
| fastq | fastq.trim_terminal_damage | Read Cleanup | 3 | 3 | 0 | not_declared | none |
| fastq | fastq.normalize_primers | Amplicon Interpretation | 1 | 1 | 0 | not_applicable | none |
| fastq | fastq.trim_polyg_tails | Read Cleanup | 2 | 2 | 0 | not_declared | none |
| fastq | fastq.trim_reads | Read Cleanup | 14 | 13 | 1 | not_declared | seqpurge (support) |
| fastq | fastq.filter_reads | Read Cleanup | 4 | 4 | 0 | not_declared | none |
| fastq | fastq.profile_reads | Quality Profiling | 3 | 3 | 0 | not_declared | none |
| fastq | fastq.deplete_rrna | Contamination Screening | 1 | 1 | 0 | not_applicable | none |
| fastq | fastq.merge_pairs | Read Cleanup | 6 | 6 | 0 | not_declared | none |
| fastq | fastq.remove_duplicates | Read Cleanup | 2 | 2 | 0 | not_declared | none |
| fastq | fastq.filter_low_complexity | Read Cleanup | 4 | 2 | 2 | not_declared | dustmasker (support), fastp (support) |
| fastq | fastq.deplete_host | Contamination Screening | 1 | 1 | 0 | not_applicable | none |
| fastq | fastq.deplete_reference_contaminants | Contamination Screening | 1 | 1 | 0 | not_applicable | none |
| fastq | fastq.correct_errors | Read Cleanup | 4 | 4 | 0 | not_declared | none |
| fastq | fastq.extract_umis | Read Cleanup | 1 | 1 | 0 | not_applicable | none |
| fastq | fastq.profile_overrepresented_sequences | Quality Profiling | 3 | 3 | 0 | sequence_count, flagged_sequences, top_fraction | none |
| fastq | fastq.remove_chimeras | Amplicon Interpretation | 1 | 1 | 0 | not_applicable | none |
| fastq | fastq.infer_asvs | Amplicon Interpretation | 1 | 1 | 0 | not_applicable | none |
| fastq | fastq.cluster_otus | Amplicon Interpretation | 1 | 1 | 0 | not_applicable | none |
| fastq | fastq.normalize_abundance | Amplicon Interpretation | 2 | 1 | 1 | not_declared | seqfu (support) |
| fastq | fastq.screen_taxonomy | Contamination Screening | 4 | 4 | 0 | not_declared | none |
| fastq | fastq.report_qc | Quality Profiling | 1 | 0 | 1 | not_applicable | multiqc (corpus) |
| fastq | fastq.index_reference | Reference Preparation | 2 | 0 | 2 | index_build_exit_code | bowtie2_build (corpus), star (corpus) |

## bam.align

- Domain: bam
- Report section: Alignment Intake
- Summary table: Alignment Baseline
- Anchor tool: bwa (supported)
- Tools: 2
- Ready tools: 2
- Blocked tools: 0
- Shared metric contract: declared
- Shared metrics: mapped_reads, alignment_rate

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| bowtie2 | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-mini | not_required |
| bwa | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-mini | not_required |

## bam.validate

- Domain: bam
- Report section: Alignment Intake
- Summary table: Alignment Baseline
- Anchor tool: samtools (supported)
- Tools: 3
- Ready tools: 3
- Blocked tools: 0
- Shared metric contract: declared
- Shared metrics: validation_status, validation_errors, validation_warnings, input_bam_identity

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| bamtools | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |
| bedtools | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |
| samtools | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |

## bam.qc_pre

- Domain: bam
- Report section: Alignment Intake
- Summary table: Alignment Baseline
- Anchor tool: samtools (supported)
- Tools: 2
- Ready tools: 2
- Blocked tools: 0
- Shared metric contract: declared
- Shared metrics: total_reads, mapped_reads, unmapped_reads, duplicate_flagged_reads, contig_summary

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| multiqc | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |
| samtools | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |

## bam.mapping_summary

- Domain: bam
- Report section: Alignment Intake
- Summary table: Alignment Baseline
- Anchor tool: samtools (supported)
- Tools: 2
- Ready tools: 2
- Blocked tools: 0
- Shared metric contract: declared
- Shared metrics: mapping_fraction, mapped_reads, unmapped_reads, secondary_reads, supplementary_reads

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| picard | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |
| samtools | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |

## bam.filter

- Domain: bam
- Report section: Alignment Refinement
- Summary table: Filter and Retention
- Anchor tool: samtools (supported)
- Tools: 3
- Ready tools: 3
- Blocked tools: 0
- Shared metric contract: declared
- Shared metrics: input_reads, kept_reads, removed_reads, active_filters

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| bamtools | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |
| bedtools | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |
| samtools | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |

## bam.mapq_filter

- Domain: bam
- Report section: Alignment Refinement
- Summary table: Filter and Retention
- Anchor tool: samtools (supported)
- Tools: 2
- Ready tools: 2
- Blocked tools: 0
- Shared metric contract: declared
- Shared metrics: mapq_threshold, kept_reads, removed_reads, filtered_bam

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| bamtools | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |
| samtools | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |

## bam.length_filter

- Domain: bam
- Report section: Alignment Refinement
- Summary table: Filter and Retention
- Anchor tool: samtools (supported)
- Tools: 2
- Ready tools: 2
- Blocked tools: 0
- Shared metric contract: declared
- Shared metrics: min_length_threshold, kept_reads, removed_reads, filtered_bam

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| picard | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |
| samtools | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |

## bam.markdup

- Domain: bam
- Report section: Library Complexity
- Summary table: Duplicate and Complexity
- Anchor tool: samtools (supported)
- Tools: 2
- Ready tools: 2
- Blocked tools: 0
- Shared metric contract: declared
- Shared metrics: marked_bam, duplicate_metrics, duplicate_count, duplicate_fraction

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| picard | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |
| samtools | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |

## bam.duplication_metrics

- Domain: bam
- Report section: Library Complexity
- Summary table: Duplicate and Complexity
- Anchor tool: samtools (supported)
- Tools: 2
- Ready tools: 2
- Blocked tools: 0
- Shared metric contract: declared
- Shared metrics: duplicate_count, duplicate_fraction, estimated_library_size

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| picard | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |
| samtools | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |

## bam.complexity

- Domain: bam
- Report section: Library Complexity
- Summary table: Duplicate and Complexity
- Anchor tool: preseq (supported)
- Tools: 1
- Ready tools: 1
- Blocked tools: 0
- Shared metric contract: not_applicable
- Shared metrics: none

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| preseq | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |

## bam.coverage

- Domain: bam
- Report section: Coverage and Quality
- Summary table: Coverage, Bias, and QC
- Anchor tool: mosdepth (supported)
- Tools: 3
- Ready tools: 3
- Blocked tools: 0
- Shared metric contract: declared
- Shared metrics: mean_depth, breadth_1x, covered_bases, observed_region_count, region_ids

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| bedtools | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |
| mosdepth | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |
| samtools | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |

## bam.insert_size

- Domain: bam
- Report section: Coverage and Quality
- Summary table: Coverage, Bias, and QC
- Anchor tool: picard (supported)
- Tools: 1
- Ready tools: 1
- Blocked tools: 0
- Shared metric contract: not_applicable
- Shared metrics: none

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| picard | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |

## bam.gc_bias

- Domain: bam
- Report section: Coverage and Quality
- Summary table: Coverage, Bias, and QC
- Anchor tool: picard (supported)
- Tools: 1
- Ready tools: 1
- Blocked tools: 0
- Shared metric contract: not_applicable
- Shared metrics: none

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| picard | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |

## bam.endogenous_content

- Domain: bam
- Report section: Coverage and Quality
- Summary table: Coverage, Bias, and QC
- Anchor tool: samtools (supported)
- Tools: 1
- Ready tools: 1
- Blocked tools: 0
- Shared metric contract: not_applicable
- Shared metrics: none

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| samtools | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |

## bam.overlap_correction

- Domain: bam
- Report section: Alignment Refinement
- Summary table: Filter and Retention
- Anchor tool: bamutil (supported)
- Tools: 1
- Ready tools: 1
- Blocked tools: 0
- Shared metric contract: not_applicable
- Shared metrics: none

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| bamutil | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |

## bam.damage

- Domain: bam
- Report section: Ancient Signal
- Summary table: Damage and Authenticity
- Anchor tool: mapdamage2 (supported)
- Tools: 6
- Ready tools: 6
- Blocked tools: 0
- Shared metric contract: declared
- Shared metrics: terminal_c_to_t_5p, terminal_g_to_a_3p, damage_signal, runtime_s, memory_mb

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| addeam | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-adna-damage-mini | not_required |
| damageprofiler | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-adna-damage-mini | not_required |
| mapdamage2 | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-adna-damage-mini | not_required |
| ngsbriggs | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-adna-damage-mini | not_required |
| pmdtools | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-adna-damage-mini | not_required |
| pydamage | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-adna-damage-mini | not_required |

## bam.authenticity

- Domain: bam
- Report section: Ancient Signal
- Summary table: Damage and Authenticity
- Anchor tool: authenticct (supported)
- Tools: 3
- Ready tools: 3
- Blocked tools: 0
- Shared metric contract: declared
- Shared metrics: score, confidence, pmd_like_signal_present, consumed_metric_ids, missing_metric_ids

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| authenticct | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-adna-damage-mini | not_required |
| damageprofiler | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-adna-damage-mini | not_required |
| pmdtools | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-adna-damage-mini | not_required |

## bam.contamination

- Domain: bam
- Report section: Ancient Signal
- Summary table: Damage and Authenticity
- Anchor tool: schmutzi (supported)
- Tools: 3
- Ready tools: 3
- Blocked tools: 0
- Shared metric contract: declared
- Shared metrics: scope, prerequisites_passed, estimate, ci_low, ci_high

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| contammix | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-adna-bam-mini | assigned |
| schmutzi | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-adna-bam-mini | assigned |
| verifybamid2 | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-adna-bam-mini | assigned |

## bam.sex

- Domain: bam
- Report section: Sample Identity
- Summary table: Identity and Relatedness
- Anchor tool: rxy (supported)
- Tools: 3
- Ready tools: 3
- Blocked tools: 0
- Shared metric contract: declared
- Shared metrics: x_coverage, y_coverage, autosomal_coverage, call, confidence, status

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| angsd | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-adna-bam-mini | assigned |
| rxy | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-adna-bam-mini | assigned |
| yleaf | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-adna-bam-mini | assigned |

## bam.bias_mitigation

- Domain: bam
- Report section: Downstream Readiness
- Summary table: Variant and Bias Readiness
- Anchor tool: mapdamage2 (supported)
- Tools: 1
- Ready tools: 1
- Blocked tools: 0
- Shared metric contract: not_applicable
- Shared metrics: none

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| mapdamage2 | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |

## bam.recalibration

- Domain: bam
- Report section: Downstream Readiness
- Summary table: Variant and Bias Readiness
- Anchor tool: gatk (supported)
- Tools: 1
- Ready tools: 1
- Blocked tools: 0
- Shared metric contract: not_applicable
- Shared metrics: none

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| gatk | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | assigned |

## bam.haplogroups

- Domain: bam
- Report section: Sample Identity
- Summary table: Identity and Relatedness
- Anchor tool: yleaf (supported)
- Tools: 1
- Ready tools: 1
- Blocked tools: 0
- Shared metric contract: not_applicable
- Shared metrics: none

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| yleaf | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-adna-bam-mini | assigned |

## bam.genotyping

- Domain: bam
- Report section: Downstream Readiness
- Summary table: Variant and Bias Readiness
- Anchor tool: angsd (supported)
- Tools: 1
- Ready tools: 1
- Blocked tools: 0
- Shared metric contract: not_applicable
- Shared metrics: none

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| angsd | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-genotyping-mini | assigned |

## bam.kinship

- Domain: bam
- Report section: Sample Identity
- Summary table: Identity and Relatedness
- Anchor tool: king (supported)
- Tools: 2
- Ready tools: 2
- Blocked tools: 0
- Shared metric contract: declared
- Shared metrics: observed_max_overlap_snps, pair_count, status, pairwise_results

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| angsd | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-kinship-mini | assigned |
| king | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-kinship-mini | assigned |

## fastq.validate_reads

- Domain: fastq
- Report section: Input Readiness
- Summary table: Validation and Intake
- Anchor tool: fastqvalidator (supported)
- Tools: 5
- Ready tools: 5
- Blocked tools: 0
- Shared metric contract: declared
- Shared metrics: format_validation_pass_rate

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| fastq_scan | benchmark_ready | none | observer_specialized_benchmark | runnable | comparable | fixture:corpus-01-mini | not_required |
| fastqc | benchmark_ready | none | observer_specialized_benchmark | runnable | comparable | fixture:corpus-01-mini | not_required |
| fastqvalidator | benchmark_ready | none | observer_specialized_benchmark | runnable | comparable | fixture:corpus-01-mini | not_required |
| fqtools | benchmark_ready | none | observer_specialized_benchmark | runnable | comparable | fixture:corpus-01-mini | not_required |
| seqtk | benchmark_ready | none | observer_specialized_benchmark | runnable | comparable | fixture:corpus-01-mini | not_required |

## fastq.profile_read_lengths

- Domain: fastq
- Report section: Quality Profiling
- Summary table: QC Signal Profiles
- Anchor tool: seqkit_stats (supported)
- Tools: 4
- Ready tools: 4
- Blocked tools: 0
- Shared metric contract: not_declared
- Shared metrics: none

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| fastp | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| prinseq | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| seqfu | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| seqkit_stats | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |

## fastq.detect_adapters

- Domain: fastq
- Report section: Quality Profiling
- Summary table: QC Signal Profiles
- Anchor tool: fastqc (supported)
- Tools: 1
- Ready tools: 1
- Blocked tools: 0
- Shared metric contract: not_applicable
- Shared metrics: none

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| fastqc | benchmark_ready | none | observer_specialized_benchmark | runnable | comparable | fixture:corpus-01-mini | not_required |

## fastq.detect_duplicates_premerge

- Domain: fastq
- Report section: Quality Profiling
- Summary table: Pre-merge Complexity
- Anchor tool: bijux_dna (supported)
- Tools: 1
- Ready tools: 1
- Blocked tools: 0
- Shared metric contract: not_applicable
- Shared metrics: none

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| bijux_dna | benchmark_ready | none | governed_execution | runnable | parse_normalized | fixture:corpus-01-mini | not_required |

## fastq.estimate_library_complexity_prealign

- Domain: fastq
- Report section: Quality Profiling
- Summary table: Pre-merge Complexity
- Anchor tool: bijux_dna (supported)
- Tools: 1
- Ready tools: 1
- Blocked tools: 0
- Shared metric contract: not_applicable
- Shared metrics: none

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| bijux_dna | benchmark_ready | none | governed_execution | runnable | parse_normalized | fixture:corpus-01-mini | not_required |

## fastq.trim_terminal_damage

- Domain: fastq
- Report section: Read Cleanup
- Summary table: Cleanup and Retention
- Anchor tool: cutadapt (supported)
- Tools: 3
- Ready tools: 3
- Blocked tools: 0
- Shared metric contract: not_declared
- Shared metrics: none

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| adapterremoval | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| cutadapt | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| seqkit | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |

## fastq.normalize_primers

- Domain: fastq
- Report section: Amplicon Interpretation
- Summary table: Amplicon Feature Tables
- Anchor tool: cutadapt (supported)
- Tools: 1
- Ready tools: 1
- Blocked tools: 0
- Shared metric contract: not_applicable
- Shared metrics: none

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| cutadapt | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-03-amplicon-mini | not_required |

## fastq.trim_polyg_tails

- Domain: fastq
- Report section: Read Cleanup
- Summary table: Cleanup and Retention
- Anchor tool: fastp (supported)
- Tools: 2
- Ready tools: 2
- Blocked tools: 0
- Shared metric contract: not_declared
- Shared metrics: none

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| bbduk | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| fastp | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |

## fastq.trim_reads

- Domain: fastq
- Report section: Read Cleanup
- Summary table: Cleanup and Retention
- Anchor tool: fastp (supported)
- Tools: 14
- Ready tools: 13
- Blocked tools: 1
- Shared metric contract: not_declared
- Shared metrics: none

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| adapterremoval | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| alientrimmer | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| atropos | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| bbduk | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| cutadapt | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| fastp | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| fastx_clipper | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| leehom | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| prinseq | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| seqkit | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| seqpurge | not_benchmark_ready | support | planned_contract | declared_only | not_normalized | fixture:corpus-01-mini | not_required |
| skewer | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| trim_galore | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| trimmomatic | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |

## fastq.filter_reads

- Domain: fastq
- Report section: Read Cleanup
- Summary table: Cleanup and Retention
- Anchor tool: fastp (supported)
- Tools: 4
- Ready tools: 4
- Blocked tools: 0
- Shared metric contract: not_declared
- Shared metrics: none

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| bbduk | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| fastp | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| prinseq | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| seqkit | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |

## fastq.profile_reads

- Domain: fastq
- Report section: Quality Profiling
- Summary table: QC Signal Profiles
- Anchor tool: seqkit_stats (supported)
- Tools: 3
- Ready tools: 3
- Blocked tools: 0
- Shared metric contract: not_declared
- Shared metrics: none

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| seqfu | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| seqkit | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| seqkit_stats | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |

## fastq.deplete_rrna

- Domain: fastq
- Report section: Contamination Screening
- Summary table: Screening and Contamination
- Anchor tool: sortmerna (supported)
- Tools: 1
- Ready tools: 1
- Blocked tools: 0
- Shared metric contract: not_applicable
- Shared metrics: none

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| sortmerna | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | assigned |

## fastq.merge_pairs

- Domain: fastq
- Report section: Read Cleanup
- Summary table: Cleanup and Retention
- Anchor tool: pear (supported)
- Tools: 6
- Ready tools: 6
- Blocked tools: 0
- Shared metric contract: not_declared
- Shared metrics: none

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| adapterremoval | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| bbmerge | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| flash2 | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| leehom | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| pear | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| vsearch | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |

## fastq.remove_duplicates

- Domain: fastq
- Report section: Read Cleanup
- Summary table: Cleanup and Retention
- Anchor tool: clumpify (supported)
- Tools: 2
- Ready tools: 2
- Blocked tools: 0
- Shared metric contract: not_declared
- Shared metrics: none

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| clumpify | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| fastuniq | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |

## fastq.filter_low_complexity

- Domain: fastq
- Report section: Read Cleanup
- Summary table: Cleanup and Retention
- Anchor tool: bbduk (supported)
- Tools: 4
- Ready tools: 2
- Blocked tools: 2
- Shared metric contract: not_declared
- Shared metrics: none

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| bbduk | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| dustmasker | not_benchmark_ready | support | planned_contract | declared_only | not_normalized | fixture:corpus-01-mini | not_required |
| fastp | not_benchmark_ready | support | planned_contract | declared_only | not_normalized | fixture:corpus-01-mini | not_required |
| prinseq | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |

## fastq.deplete_host

- Domain: fastq
- Report section: Contamination Screening
- Summary table: Screening and Contamination
- Anchor tool: bowtie2 (supported)
- Tools: 1
- Ready tools: 1
- Blocked tools: 0
- Shared metric contract: not_applicable
- Shared metrics: none

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| bowtie2 | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | assigned |

## fastq.deplete_reference_contaminants

- Domain: fastq
- Report section: Contamination Screening
- Summary table: Screening and Contamination
- Anchor tool: bowtie2 (supported)
- Tools: 1
- Ready tools: 1
- Blocked tools: 0
- Shared metric contract: not_applicable
- Shared metrics: none

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| bowtie2 | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | assigned |

## fastq.correct_errors

- Domain: fastq
- Report section: Read Cleanup
- Summary table: Cleanup and Retention
- Anchor tool: rcorrector (supported)
- Tools: 4
- Ready tools: 4
- Blocked tools: 0
- Shared metric contract: not_declared
- Shared metrics: none

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| bayeshammer | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| lighter | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| musket | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| rcorrector | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |

## fastq.extract_umis

- Domain: fastq
- Report section: Read Cleanup
- Summary table: Cleanup and Retention
- Anchor tool: umi_tools (supported)
- Tools: 1
- Ready tools: 1
- Blocked tools: 0
- Shared metric contract: not_applicable
- Shared metrics: none

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| umi_tools | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |

## fastq.profile_overrepresented_sequences

- Domain: fastq
- Report section: Quality Profiling
- Summary table: QC Signal Profiles
- Anchor tool: fastqc (supported)
- Tools: 3
- Ready tools: 3
- Blocked tools: 0
- Shared metric contract: declared
- Shared metrics: sequence_count, flagged_sequences, top_fraction

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| fastq_scan | benchmark_ready | none | observer_specialized_benchmark | runnable | comparable | fixture:corpus-01-mini | not_required |
| fastqc | benchmark_ready | none | observer_specialized_benchmark | runnable | comparable | fixture:corpus-01-mini | not_required |
| seqkit | benchmark_ready | none | observer_specialized_benchmark | runnable | comparable | fixture:corpus-01-mini | not_required |

## fastq.remove_chimeras

- Domain: fastq
- Report section: Amplicon Interpretation
- Summary table: Amplicon Feature Tables
- Anchor tool: vsearch (supported)
- Tools: 1
- Ready tools: 1
- Blocked tools: 0
- Shared metric contract: not_applicable
- Shared metrics: none

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| vsearch | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-03-amplicon-mini | not_required |

## fastq.infer_asvs

- Domain: fastq
- Report section: Amplicon Interpretation
- Summary table: Amplicon Feature Tables
- Anchor tool: dada2 (supported)
- Tools: 1
- Ready tools: 1
- Blocked tools: 0
- Shared metric contract: not_applicable
- Shared metrics: none

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| dada2 | benchmark_ready | none | governed_execution | runnable | parse_normalized | fixture:corpus-03-amplicon-mini | not_required |

## fastq.cluster_otus

- Domain: fastq
- Report section: Amplicon Interpretation
- Summary table: Amplicon Feature Tables
- Anchor tool: vsearch (supported)
- Tools: 1
- Ready tools: 1
- Blocked tools: 0
- Shared metric contract: not_applicable
- Shared metrics: none

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| vsearch | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-03-amplicon-mini | not_required |

## fastq.normalize_abundance

- Domain: fastq
- Report section: Amplicon Interpretation
- Summary table: Amplicon Feature Tables
- Anchor tool: seqkit (supported)
- Tools: 2
- Ready tools: 1
- Blocked tools: 1
- Shared metric contract: not_declared
- Shared metrics: none

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| seqfu | not_benchmark_ready | support | planned_contract | declared_only | not_normalized | fixture:corpus-03-amplicon-mini | not_required |
| seqkit | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-03-amplicon-mini | not_required |

## fastq.screen_taxonomy

- Domain: fastq
- Report section: Contamination Screening
- Summary table: Screening and Contamination
- Anchor tool: kraken2 (supported)
- Tools: 4
- Ready tools: 4
- Blocked tools: 0
- Shared metric contract: not_declared
- Shared metrics: none

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| centrifuge | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-02-edna-mini | assigned |
| kaiju | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-02-edna-mini | assigned |
| kraken2 | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-02-edna-mini | assigned |
| krakenuniq | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-02-edna-mini | assigned |

## fastq.report_qc

- Domain: fastq
- Report section: Quality Profiling
- Summary table: QC Signal Profiles
- Anchor tool: multiqc (supported)
- Tools: 1
- Ready tools: 0
- Blocked tools: 1
- Shared metric contract: not_applicable
- Shared metrics: none

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| multiqc | not_benchmark_ready | corpus | observer_specialized_benchmark | runnable | comparable | planner_only | not_required |

## fastq.index_reference

- Domain: fastq
- Report section: Reference Preparation
- Summary table: Reference Index Assets
- Anchor tool: bowtie2_build (supported)
- Tools: 2
- Ready tools: 0
- Blocked tools: 2
- Shared metric contract: declared
- Shared metrics: index_build_exit_code

| Tool | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- |
| bowtie2_build | not_benchmark_ready | corpus | observer_specialized_benchmark | runnable | comparable | planner_only | assigned |
| star | not_benchmark_ready | corpus | observer_specialized_benchmark | runnable | comparable | planner_only | not_required |
