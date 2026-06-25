# Corpus-Centric Benchmark Report

## Summary

- Corpus count: 8
- Assigned stages: 51
- Assigned stage-tool rows: 120
- Benchmark-ready rows: 120
- Blocked rows: 0
- Corpora with blocked stages: 0

| Corpus | Domains | Fixtures | Stages | Tool rows | Ready | Blocked stages | Blocked stage ids |
| --- | --- | --- | ---: | ---: | ---: | ---: | --- |
| corpus-01 | bam, fastq | corpus-01-mini | 21 | 62 | 62 | 0 | none |
| corpus-01-adna-bam | bam | corpus-01-adna-bam-mini, corpus-01-adna-damage-mini | 5 | 16 | 16 | 0 | none |
| corpus-01-bam | bam | corpus-01-bam-mini | 16 | 28 | 28 | 0 | none |
| corpus-01-genotyping | bam | corpus-01-genotyping-mini | 1 | 1 | 1 | 0 | none |
| corpus-01-kinship | bam | corpus-01-kinship-mini | 1 | 2 | 2 | 0 | none |
| corpus-02 | fastq | corpus-02-edna-mini | 1 | 4 | 4 | 0 | none |
| corpus-03 | fastq | corpus-03-amplicon-mini | 5 | 5 | 5 | 0 | none |
| reference-index-assets | fastq | reference-index-assets | 1 | 2 | 2 | 0 | none |

## corpus-01

- Domains: bam, fastq
- Fixtures: corpus-01-mini
- Stages: 21
- Tool rows: 62
- Benchmark-ready rows: 62
- Blocked stages: 0

| Domain | Stage | Fixtures | Report section | Tools | Ready | Blocked | Shared metrics | Blocked tools |
| --- | --- | --- | --- | ---: | ---: | ---: | --- | --- |
| bam | bam.align | corpus-01-mini | Alignment Intake | 2 | 2 | 0 | mapped_reads, alignment_rate | none |
| fastq | fastq.validate_reads | corpus-01-mini | Input Readiness | 5 | 5 | 0 | format_validation_pass_rate | none |
| fastq | fastq.profile_read_lengths | corpus-01-mini | Quality Profiling | 4 | 4 | 0 | not_declared | none |
| fastq | fastq.detect_adapters | corpus-01-mini | Quality Profiling | 1 | 1 | 0 | not_applicable | none |
| fastq | fastq.detect_duplicates_premerge | corpus-01-mini | Quality Profiling | 1 | 1 | 0 | not_applicable | none |
| fastq | fastq.estimate_library_complexity_prealign | corpus-01-mini | Quality Profiling | 1 | 1 | 0 | not_applicable | none |
| fastq | fastq.trim_terminal_damage | corpus-01-mini | Read Cleanup | 3 | 3 | 0 | not_declared | none |
| fastq | fastq.trim_polyg_tails | corpus-01-mini | Read Cleanup | 2 | 2 | 0 | not_declared | none |
| fastq | fastq.trim_reads | corpus-01-mini | Read Cleanup | 13 | 13 | 0 | not_declared | none |
| fastq | fastq.filter_reads | corpus-01-mini | Read Cleanup | 4 | 4 | 0 | not_declared | none |
| fastq | fastq.profile_reads | corpus-01-mini | Quality Profiling | 3 | 3 | 0 | not_declared | none |
| fastq | fastq.deplete_rrna | corpus-01-mini | Contamination Screening | 1 | 1 | 0 | not_applicable | none |
| fastq | fastq.merge_pairs | corpus-01-mini | Read Cleanup | 6 | 6 | 0 | not_declared | none |
| fastq | fastq.remove_duplicates | corpus-01-mini | Read Cleanup | 2 | 2 | 0 | not_declared | none |
| fastq | fastq.filter_low_complexity | corpus-01-mini | Read Cleanup | 3 | 3 | 0 | not_declared | none |
| fastq | fastq.deplete_host | corpus-01-mini | Contamination Screening | 1 | 1 | 0 | not_applicable | none |
| fastq | fastq.deplete_reference_contaminants | corpus-01-mini | Contamination Screening | 1 | 1 | 0 | not_applicable | none |
| fastq | fastq.correct_errors | corpus-01-mini | Read Cleanup | 4 | 4 | 0 | not_declared | none |
| fastq | fastq.extract_umis | corpus-01-mini | Read Cleanup | 1 | 1 | 0 | not_applicable | none |
| fastq | fastq.profile_overrepresented_sequences | corpus-01-mini | Quality Profiling | 3 | 3 | 0 | sequence_count, flagged_sequences, top_fraction | none |
| fastq | fastq.report_qc | corpus-01-mini | Quality Profiling | 1 | 1 | 0 | not_applicable | none |

## corpus-01-adna-bam

- Domains: bam
- Fixtures: corpus-01-adna-bam-mini, corpus-01-adna-damage-mini
- Stages: 5
- Tool rows: 16
- Benchmark-ready rows: 16
- Blocked stages: 0

| Domain | Stage | Fixtures | Report section | Tools | Ready | Blocked | Shared metrics | Blocked tools |
| --- | --- | --- | --- | ---: | ---: | ---: | --- | --- |
| bam | bam.damage | corpus-01-adna-damage-mini | Ancient Signal | 6 | 6 | 0 | terminal_c_to_t_5p, terminal_g_to_a_3p, damage_signal | none |
| bam | bam.authenticity | corpus-01-adna-damage-mini | Ancient Signal | 3 | 3 | 0 | score, confidence, status, pmd_like_signal_present | none |
| bam | bam.contamination | corpus-01-adna-bam-mini | Ancient Signal | 3 | 3 | 0 | estimate, ci_low, ci_high | none |
| bam | bam.sex | corpus-01-adna-bam-mini | Sample Identity | 3 | 3 | 0 | x_coverage, y_coverage, autosomal_coverage, call, confidence, status | none |
| bam | bam.haplogroups | corpus-01-adna-bam-mini | Sample Identity | 1 | 1 | 0 | not_applicable | none |

## corpus-01-bam

- Domains: bam
- Fixtures: corpus-01-bam-mini
- Stages: 16
- Tool rows: 28
- Benchmark-ready rows: 28
- Blocked stages: 0

| Domain | Stage | Fixtures | Report section | Tools | Ready | Blocked | Shared metrics | Blocked tools |
| --- | --- | --- | --- | ---: | ---: | ---: | --- | --- |
| bam | bam.validate | corpus-01-bam-mini | Alignment Intake | 3 | 3 | 0 | validation_status, validation_errors, validation_warnings | none |
| bam | bam.qc_pre | corpus-01-bam-mini | Alignment Intake | 2 | 2 | 0 | total_reads, mapped_reads, unmapped_reads, duplicate_flagged_reads | none |
| bam | bam.mapping_summary | corpus-01-bam-mini | Alignment Intake | 2 | 2 | 0 | mapping_fraction, mapped_reads, unmapped_reads, secondary_reads, supplementary_reads | none |
| bam | bam.filter | corpus-01-bam-mini | Alignment Refinement | 3 | 3 | 0 | input_reads, kept_reads, removed_reads | none |
| bam | bam.mapq_filter | corpus-01-bam-mini | Alignment Refinement | 2 | 2 | 0 | mapq_threshold, kept_reads, removed_reads | none |
| bam | bam.length_filter | corpus-01-bam-mini | Alignment Refinement | 2 | 2 | 0 | min_length_threshold, kept_reads, removed_reads | none |
| bam | bam.markdup | corpus-01-bam-mini | Library Complexity | 2 | 2 | 0 | duplicate_count, duplicate_fraction | none |
| bam | bam.duplication_metrics | corpus-01-bam-mini | Library Complexity | 2 | 2 | 0 | duplicate_count, duplicate_fraction, estimated_library_size | none |
| bam | bam.complexity | corpus-01-bam-mini | Library Complexity | 1 | 1 | 0 | not_applicable | none |
| bam | bam.coverage | corpus-01-bam-mini | Coverage and Quality | 3 | 3 | 0 | mean_depth, breadth_1x, covered_bases | none |
| bam | bam.insert_size | corpus-01-bam-mini | Coverage and Quality | 1 | 1 | 0 | not_applicable | none |
| bam | bam.gc_bias | corpus-01-bam-mini | Coverage and Quality | 1 | 1 | 0 | not_applicable | none |
| bam | bam.endogenous_content | corpus-01-bam-mini | Coverage and Quality | 1 | 1 | 0 | not_applicable | none |
| bam | bam.overlap_correction | corpus-01-bam-mini | Alignment Refinement | 1 | 1 | 0 | not_applicable | none |
| bam | bam.bias_mitigation | corpus-01-bam-mini | Downstream Readiness | 1 | 1 | 0 | not_applicable | none |
| bam | bam.recalibration | corpus-01-bam-mini | Downstream Readiness | 1 | 1 | 0 | not_applicable | none |

## corpus-01-genotyping

- Domains: bam
- Fixtures: corpus-01-genotyping-mini
- Stages: 1
- Tool rows: 1
- Benchmark-ready rows: 1
- Blocked stages: 0

| Domain | Stage | Fixtures | Report section | Tools | Ready | Blocked | Shared metrics | Blocked tools |
| --- | --- | --- | --- | ---: | ---: | ---: | --- | --- |
| bam | bam.genotyping | corpus-01-genotyping-mini | Downstream Readiness | 1 | 1 | 0 | not_applicable | none |

## corpus-01-kinship

- Domains: bam
- Fixtures: corpus-01-kinship-mini
- Stages: 1
- Tool rows: 2
- Benchmark-ready rows: 2
- Blocked stages: 0

| Domain | Stage | Fixtures | Report section | Tools | Ready | Blocked | Shared metrics | Blocked tools |
| --- | --- | --- | --- | ---: | ---: | ---: | --- | --- |
| bam | bam.kinship | corpus-01-kinship-mini | Sample Identity | 2 | 2 | 0 | observed_max_overlap_snps, pair_count, status, pairwise_results | none |

## corpus-02

- Domains: fastq
- Fixtures: corpus-02-edna-mini
- Stages: 1
- Tool rows: 4
- Benchmark-ready rows: 4
- Blocked stages: 0

| Domain | Stage | Fixtures | Report section | Tools | Ready | Blocked | Shared metrics | Blocked tools |
| --- | --- | --- | --- | ---: | ---: | ---: | --- | --- |
| fastq | fastq.screen_taxonomy | corpus-02-edna-mini | Contamination Screening | 4 | 4 | 0 | not_declared | none |

## corpus-03

- Domains: fastq
- Fixtures: corpus-03-amplicon-mini
- Stages: 5
- Tool rows: 5
- Benchmark-ready rows: 5
- Blocked stages: 0

| Domain | Stage | Fixtures | Report section | Tools | Ready | Blocked | Shared metrics | Blocked tools |
| --- | --- | --- | --- | ---: | ---: | ---: | --- | --- |
| fastq | fastq.normalize_primers | corpus-03-amplicon-mini | Amplicon Interpretation | 1 | 1 | 0 | not_applicable | none |
| fastq | fastq.remove_chimeras | corpus-03-amplicon-mini | Amplicon Interpretation | 1 | 1 | 0 | not_applicable | none |
| fastq | fastq.infer_asvs | corpus-03-amplicon-mini | Amplicon Interpretation | 1 | 1 | 0 | not_applicable | none |
| fastq | fastq.cluster_otus | corpus-03-amplicon-mini | Amplicon Interpretation | 1 | 1 | 0 | not_applicable | none |
| fastq | fastq.normalize_abundance | corpus-03-amplicon-mini | Amplicon Interpretation | 1 | 1 | 0 | not_applicable | none |

## reference-index-assets

- Domains: fastq
- Fixtures: reference-index-assets
- Stages: 1
- Tool rows: 2
- Benchmark-ready rows: 2
- Blocked stages: 0

| Domain | Stage | Fixtures | Report section | Tools | Ready | Blocked | Shared metrics | Blocked tools |
| --- | --- | --- | --- | ---: | ---: | ---: | --- | --- |
| fastq | fastq.index_reference | reference-index-assets | Reference Preparation | 2 | 2 | 0 | index_build_exit_code | none |
