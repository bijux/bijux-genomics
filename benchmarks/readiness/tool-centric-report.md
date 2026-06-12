# Tool-Centric Benchmark Report

## Summary

- Tool count: 67
- Stage-tool rows: 123
- Benchmark-ready rows: 115
- Blocked rows: 8
- Tools with blockers: 8

| Tool | Domains | Stage rows | Ready | Blocked | Blocked stages |
| --- | --- | ---: | ---: | ---: | --- |
| adapterremoval | fastq | 3 | 3 | 0 | none |
| addeam | bam | 1 | 1 | 0 | none |
| alientrimmer | fastq | 1 | 1 | 0 | none |
| angsd | bam | 3 | 3 | 0 | none |
| atropos | fastq | 1 | 1 | 0 | none |
| authenticct | bam | 1 | 1 | 0 | none |
| bamtools | bam | 3 | 3 | 0 | none |
| bamutil | bam | 1 | 1 | 0 | none |
| bayeshammer | fastq | 1 | 1 | 0 | none |
| bbduk | fastq | 4 | 4 | 0 | none |
| bbmerge | fastq | 1 | 1 | 0 | none |
| bedtools | bam | 3 | 3 | 0 | none |
| bijux_dna | fastq | 2 | 1 | 1 | fastq.estimate_library_complexity_prealign (support) |
| bowtie2 | bam, fastq | 3 | 3 | 0 | none |
| bowtie2_build | fastq | 1 | 0 | 1 | fastq.index_reference (corpus) |
| bwa | bam | 1 | 1 | 0 | none |
| centrifuge | fastq | 1 | 1 | 0 | none |
| clumpify | fastq | 1 | 1 | 0 | none |
| contammix | bam | 1 | 1 | 0 | none |
| cutadapt | fastq | 3 | 3 | 0 | none |
| dada2 | fastq | 1 | 1 | 0 | none |
| damageprofiler | bam | 2 | 2 | 0 | none |
| dustmasker | fastq | 1 | 0 | 1 | fastq.filter_low_complexity (support) |
| fastp | fastq | 5 | 4 | 1 | fastq.filter_low_complexity (support) |
| fastq_scan | fastq | 2 | 2 | 0 | none |
| fastqc | fastq | 3 | 3 | 0 | none |
| fastqvalidator | fastq | 1 | 1 | 0 | none |
| fastuniq | fastq | 1 | 1 | 0 | none |
| fastx_clipper | fastq | 1 | 1 | 0 | none |
| flash2 | fastq | 1 | 1 | 0 | none |
| fqtools | fastq | 1 | 1 | 0 | none |
| gatk | bam | 1 | 1 | 0 | none |
| kaiju | fastq | 1 | 1 | 0 | none |
| king | bam | 1 | 1 | 0 | none |
| kraken2 | fastq | 1 | 1 | 0 | none |
| krakenuniq | fastq | 1 | 1 | 0 | none |
| leehom | fastq | 2 | 2 | 0 | none |
| lighter | fastq | 1 | 1 | 0 | none |
| mapdamage2 | bam | 2 | 2 | 0 | none |
| mosdepth | bam | 1 | 1 | 0 | none |
| multiqc | bam, fastq | 2 | 1 | 1 | fastq.report_qc (corpus) |
| musket | fastq | 1 | 1 | 0 | none |
| ngsbriggs | bam | 1 | 1 | 0 | none |
| pear | fastq | 1 | 1 | 0 | none |
| picard | bam | 6 | 6 | 0 | none |
| pmdtools | bam | 2 | 2 | 0 | none |
| preseq | bam | 1 | 1 | 0 | none |
| prinseq | fastq | 4 | 4 | 0 | none |
| pydamage | bam | 1 | 1 | 0 | none |
| rcorrector | fastq | 1 | 1 | 0 | none |
| rxy | bam | 1 | 1 | 0 | none |
| samtools | bam | 10 | 10 | 0 | none |
| schmutzi | bam | 1 | 1 | 0 | none |
| seqfu | fastq | 3 | 2 | 1 | fastq.normalize_abundance (support) |
| seqkit | fastq | 6 | 6 | 0 | none |
| seqkit_stats | fastq | 2 | 2 | 0 | none |
| seqpurge | fastq | 1 | 0 | 1 | fastq.trim_reads (support) |
| seqtk | fastq | 1 | 1 | 0 | none |
| skewer | fastq | 1 | 1 | 0 | none |
| sortmerna | fastq | 1 | 1 | 0 | none |
| star | fastq | 1 | 0 | 1 | fastq.index_reference (corpus) |
| trim_galore | fastq | 1 | 1 | 0 | none |
| trimmomatic | fastq | 1 | 1 | 0 | none |
| umi_tools | fastq | 1 | 1 | 0 | none |
| verifybamid2 | bam | 1 | 1 | 0 | none |
| vsearch | fastq | 3 | 3 | 0 | none |
| yleaf | bam | 2 | 2 | 0 | none |

## adapterremoval

- Domains: fastq
- Stage rows: 3
- Benchmark-ready rows: 3
- Blocked rows: 0
- Report sections: read_cleanup

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.merge_pairs | Read Cleanup | Cleanup and Retention | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| fastq | fastq.trim_reads | Read Cleanup | Cleanup and Retention | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| fastq | fastq.trim_terminal_damage | Read Cleanup | Cleanup and Retention | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |

## addeam

- Domains: bam
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: ancient_signal

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| bam | bam.damage | Ancient Signal | Damage and Authenticity | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-adna-damage-mini | not_required |

## alientrimmer

- Domains: fastq
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: read_cleanup

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.trim_reads | Read Cleanup | Cleanup and Retention | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |

## angsd

- Domains: bam
- Stage rows: 3
- Benchmark-ready rows: 3
- Blocked rows: 0
- Report sections: downstream_readiness, sample_identity

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| bam | bam.genotyping | Downstream Readiness | Variant and Bias Readiness | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-genotyping-mini | assigned |
| bam | bam.kinship | Sample Identity | Identity and Relatedness | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-kinship-mini | assigned |
| bam | bam.sex | Sample Identity | Identity and Relatedness | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-adna-bam-mini | assigned |

## atropos

- Domains: fastq
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: read_cleanup

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.trim_reads | Read Cleanup | Cleanup and Retention | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |

## authenticct

- Domains: bam
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: ancient_signal

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| bam | bam.authenticity | Ancient Signal | Damage and Authenticity | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-adna-damage-mini | not_required |

## bamtools

- Domains: bam
- Stage rows: 3
- Benchmark-ready rows: 3
- Blocked rows: 0
- Report sections: alignment_intake, alignment_refinement

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| bam | bam.filter | Alignment Refinement | Filter and Retention | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |
| bam | bam.mapq_filter | Alignment Refinement | Filter and Retention | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |
| bam | bam.validate | Alignment Intake | Alignment Baseline | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |

## bamutil

- Domains: bam
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: alignment_refinement

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| bam | bam.overlap_correction | Alignment Refinement | Filter and Retention | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |

## bayeshammer

- Domains: fastq
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: read_cleanup

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.correct_errors | Read Cleanup | Cleanup and Retention | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |

## bbduk

- Domains: fastq
- Stage rows: 4
- Benchmark-ready rows: 4
- Blocked rows: 0
- Report sections: read_cleanup

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.filter_low_complexity | Read Cleanup | Cleanup and Retention | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| fastq | fastq.filter_reads | Read Cleanup | Cleanup and Retention | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| fastq | fastq.trim_polyg_tails | Read Cleanup | Cleanup and Retention | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| fastq | fastq.trim_reads | Read Cleanup | Cleanup and Retention | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |

## bbmerge

- Domains: fastq
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: read_cleanup

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.merge_pairs | Read Cleanup | Cleanup and Retention | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |

## bedtools

- Domains: bam
- Stage rows: 3
- Benchmark-ready rows: 3
- Blocked rows: 0
- Report sections: alignment_intake, alignment_refinement, coverage_quality

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| bam | bam.coverage | Coverage and Quality | Coverage, Bias, and QC | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |
| bam | bam.filter | Alignment Refinement | Filter and Retention | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |
| bam | bam.validate | Alignment Intake | Alignment Baseline | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |

## bijux_dna

- Domains: fastq
- Stage rows: 2
- Benchmark-ready rows: 1
- Blocked rows: 1
- Report sections: quality_profiling

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.detect_duplicates_premerge | Quality Profiling | Pre-merge Complexity | benchmark_ready | none | governed_execution | runnable | parse_normalized | fixture:corpus-01-mini | not_required |
| fastq | fastq.estimate_library_complexity_prealign | Quality Profiling | Pre-merge Complexity | not_benchmark_ready | support | planned_contract | declared_only | not_normalized | fixture:corpus-01-mini | not_required |

## bowtie2

- Domains: bam, fastq
- Stage rows: 3
- Benchmark-ready rows: 3
- Blocked rows: 0
- Report sections: alignment_intake, contamination_screening

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| bam | bam.align | Alignment Intake | Alignment Baseline | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-mini | not_required |
| fastq | fastq.deplete_host | Contamination Screening | Screening and Contamination | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | assigned |
| fastq | fastq.deplete_reference_contaminants | Contamination Screening | Screening and Contamination | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | assigned |

## bowtie2_build

- Domains: fastq
- Stage rows: 1
- Benchmark-ready rows: 0
- Blocked rows: 1
- Report sections: reference_preparation

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.index_reference | Reference Preparation | Reference Index Assets | not_benchmark_ready | corpus | observer_specialized_benchmark | runnable | comparable | planner_only | assigned |

## bwa

- Domains: bam
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: alignment_intake

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| bam | bam.align | Alignment Intake | Alignment Baseline | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-mini | not_required |

## centrifuge

- Domains: fastq
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: contamination_screening

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.screen_taxonomy | Contamination Screening | Screening and Contamination | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-02-edna-mini | assigned |

## clumpify

- Domains: fastq
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: read_cleanup

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.remove_duplicates | Read Cleanup | Cleanup and Retention | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |

## contammix

- Domains: bam
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: ancient_signal

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| bam | bam.contamination | Ancient Signal | Damage and Authenticity | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-adna-bam-mini | assigned |

## cutadapt

- Domains: fastq
- Stage rows: 3
- Benchmark-ready rows: 3
- Blocked rows: 0
- Report sections: amplicon_interpretation, read_cleanup

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.normalize_primers | Amplicon Interpretation | Amplicon Feature Tables | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-03-amplicon-mini | not_required |
| fastq | fastq.trim_reads | Read Cleanup | Cleanup and Retention | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| fastq | fastq.trim_terminal_damage | Read Cleanup | Cleanup and Retention | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |

## dada2

- Domains: fastq
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: amplicon_interpretation

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.infer_asvs | Amplicon Interpretation | Amplicon Feature Tables | benchmark_ready | none | governed_execution | runnable | parse_normalized | fixture:corpus-03-amplicon-mini | not_required |

## damageprofiler

- Domains: bam
- Stage rows: 2
- Benchmark-ready rows: 2
- Blocked rows: 0
- Report sections: ancient_signal

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| bam | bam.authenticity | Ancient Signal | Damage and Authenticity | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-adna-damage-mini | not_required |
| bam | bam.damage | Ancient Signal | Damage and Authenticity | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-adna-damage-mini | not_required |

## dustmasker

- Domains: fastq
- Stage rows: 1
- Benchmark-ready rows: 0
- Blocked rows: 1
- Report sections: read_cleanup

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.filter_low_complexity | Read Cleanup | Cleanup and Retention | not_benchmark_ready | support | planned_contract | declared_only | not_normalized | fixture:corpus-01-mini | not_required |

## fastp

- Domains: fastq
- Stage rows: 5
- Benchmark-ready rows: 4
- Blocked rows: 1
- Report sections: quality_profiling, read_cleanup

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.filter_low_complexity | Read Cleanup | Cleanup and Retention | not_benchmark_ready | support | planned_contract | declared_only | not_normalized | fixture:corpus-01-mini | not_required |
| fastq | fastq.filter_reads | Read Cleanup | Cleanup and Retention | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| fastq | fastq.profile_read_lengths | Quality Profiling | QC Signal Profiles | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| fastq | fastq.trim_polyg_tails | Read Cleanup | Cleanup and Retention | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| fastq | fastq.trim_reads | Read Cleanup | Cleanup and Retention | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |

## fastq_scan

- Domains: fastq
- Stage rows: 2
- Benchmark-ready rows: 2
- Blocked rows: 0
- Report sections: input_readiness, quality_profiling

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.profile_overrepresented_sequences | Quality Profiling | QC Signal Profiles | benchmark_ready | none | observer_specialized_benchmark | runnable | comparable | fixture:corpus-01-mini | not_required |
| fastq | fastq.validate_reads | Input Readiness | Validation and Intake | benchmark_ready | none | observer_specialized_benchmark | runnable | comparable | fixture:corpus-01-mini | not_required |

## fastqc

- Domains: fastq
- Stage rows: 3
- Benchmark-ready rows: 3
- Blocked rows: 0
- Report sections: input_readiness, quality_profiling

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.detect_adapters | Quality Profiling | QC Signal Profiles | benchmark_ready | none | observer_specialized_benchmark | runnable | comparable | fixture:corpus-01-mini | not_required |
| fastq | fastq.profile_overrepresented_sequences | Quality Profiling | QC Signal Profiles | benchmark_ready | none | observer_specialized_benchmark | runnable | comparable | fixture:corpus-01-mini | not_required |
| fastq | fastq.validate_reads | Input Readiness | Validation and Intake | benchmark_ready | none | observer_specialized_benchmark | runnable | comparable | fixture:corpus-01-mini | not_required |

## fastqvalidator

- Domains: fastq
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: input_readiness

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.validate_reads | Input Readiness | Validation and Intake | benchmark_ready | none | observer_specialized_benchmark | runnable | comparable | fixture:corpus-01-mini | not_required |

## fastuniq

- Domains: fastq
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: read_cleanup

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.remove_duplicates | Read Cleanup | Cleanup and Retention | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |

## fastx_clipper

- Domains: fastq
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: read_cleanup

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.trim_reads | Read Cleanup | Cleanup and Retention | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |

## flash2

- Domains: fastq
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: read_cleanup

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.merge_pairs | Read Cleanup | Cleanup and Retention | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |

## fqtools

- Domains: fastq
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: input_readiness

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.validate_reads | Input Readiness | Validation and Intake | benchmark_ready | none | observer_specialized_benchmark | runnable | comparable | fixture:corpus-01-mini | not_required |

## gatk

- Domains: bam
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: downstream_readiness

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| bam | bam.recalibration | Downstream Readiness | Variant and Bias Readiness | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | assigned |

## kaiju

- Domains: fastq
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: contamination_screening

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.screen_taxonomy | Contamination Screening | Screening and Contamination | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-02-edna-mini | assigned |

## king

- Domains: bam
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: sample_identity

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| bam | bam.kinship | Sample Identity | Identity and Relatedness | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-kinship-mini | assigned |

## kraken2

- Domains: fastq
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: contamination_screening

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.screen_taxonomy | Contamination Screening | Screening and Contamination | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-02-edna-mini | assigned |

## krakenuniq

- Domains: fastq
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: contamination_screening

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.screen_taxonomy | Contamination Screening | Screening and Contamination | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-02-edna-mini | assigned |

## leehom

- Domains: fastq
- Stage rows: 2
- Benchmark-ready rows: 2
- Blocked rows: 0
- Report sections: read_cleanup

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.merge_pairs | Read Cleanup | Cleanup and Retention | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| fastq | fastq.trim_reads | Read Cleanup | Cleanup and Retention | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |

## lighter

- Domains: fastq
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: read_cleanup

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.correct_errors | Read Cleanup | Cleanup and Retention | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |

## mapdamage2

- Domains: bam
- Stage rows: 2
- Benchmark-ready rows: 2
- Blocked rows: 0
- Report sections: ancient_signal, downstream_readiness

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| bam | bam.bias_mitigation | Downstream Readiness | Variant and Bias Readiness | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |
| bam | bam.damage | Ancient Signal | Damage and Authenticity | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-adna-damage-mini | not_required |

## mosdepth

- Domains: bam
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: coverage_quality

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| bam | bam.coverage | Coverage and Quality | Coverage, Bias, and QC | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |

## multiqc

- Domains: bam, fastq
- Stage rows: 2
- Benchmark-ready rows: 1
- Blocked rows: 1
- Report sections: alignment_intake, quality_profiling

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| bam | bam.qc_pre | Alignment Intake | Alignment Baseline | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |
| fastq | fastq.report_qc | Quality Profiling | QC Signal Profiles | not_benchmark_ready | corpus | observer_specialized_benchmark | runnable | comparable | planner_only | not_required |

## musket

- Domains: fastq
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: read_cleanup

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.correct_errors | Read Cleanup | Cleanup and Retention | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |

## ngsbriggs

- Domains: bam
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: ancient_signal

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| bam | bam.damage | Ancient Signal | Damage and Authenticity | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-adna-damage-mini | not_required |

## pear

- Domains: fastq
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: read_cleanup

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.merge_pairs | Read Cleanup | Cleanup and Retention | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |

## picard

- Domains: bam
- Stage rows: 6
- Benchmark-ready rows: 6
- Blocked rows: 0
- Report sections: alignment_intake, alignment_refinement, coverage_quality, library_complexity

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| bam | bam.duplication_metrics | Library Complexity | Duplicate and Complexity | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |
| bam | bam.gc_bias | Coverage and Quality | Coverage, Bias, and QC | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |
| bam | bam.insert_size | Coverage and Quality | Coverage, Bias, and QC | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |
| bam | bam.length_filter | Alignment Refinement | Filter and Retention | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |
| bam | bam.mapping_summary | Alignment Intake | Alignment Baseline | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |
| bam | bam.markdup | Library Complexity | Duplicate and Complexity | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |

## pmdtools

- Domains: bam
- Stage rows: 2
- Benchmark-ready rows: 2
- Blocked rows: 0
- Report sections: ancient_signal

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| bam | bam.authenticity | Ancient Signal | Damage and Authenticity | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-adna-damage-mini | not_required |
| bam | bam.damage | Ancient Signal | Damage and Authenticity | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-adna-damage-mini | not_required |

## preseq

- Domains: bam
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: library_complexity

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| bam | bam.complexity | Library Complexity | Duplicate and Complexity | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |

## prinseq

- Domains: fastq
- Stage rows: 4
- Benchmark-ready rows: 4
- Blocked rows: 0
- Report sections: quality_profiling, read_cleanup

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.filter_low_complexity | Read Cleanup | Cleanup and Retention | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| fastq | fastq.filter_reads | Read Cleanup | Cleanup and Retention | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| fastq | fastq.profile_read_lengths | Quality Profiling | QC Signal Profiles | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| fastq | fastq.trim_reads | Read Cleanup | Cleanup and Retention | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |

## pydamage

- Domains: bam
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: ancient_signal

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| bam | bam.damage | Ancient Signal | Damage and Authenticity | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-adna-damage-mini | not_required |

## rcorrector

- Domains: fastq
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: read_cleanup

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.correct_errors | Read Cleanup | Cleanup and Retention | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |

## rxy

- Domains: bam
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: sample_identity

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| bam | bam.sex | Sample Identity | Identity and Relatedness | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-adna-bam-mini | assigned |

## samtools

- Domains: bam
- Stage rows: 10
- Benchmark-ready rows: 10
- Blocked rows: 0
- Report sections: alignment_intake, alignment_refinement, coverage_quality, library_complexity

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| bam | bam.coverage | Coverage and Quality | Coverage, Bias, and QC | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |
| bam | bam.duplication_metrics | Library Complexity | Duplicate and Complexity | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |
| bam | bam.endogenous_content | Coverage and Quality | Coverage, Bias, and QC | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |
| bam | bam.filter | Alignment Refinement | Filter and Retention | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |
| bam | bam.length_filter | Alignment Refinement | Filter and Retention | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |
| bam | bam.mapping_summary | Alignment Intake | Alignment Baseline | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |
| bam | bam.mapq_filter | Alignment Refinement | Filter and Retention | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |
| bam | bam.markdup | Library Complexity | Duplicate and Complexity | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |
| bam | bam.qc_pre | Alignment Intake | Alignment Baseline | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |
| bam | bam.validate | Alignment Intake | Alignment Baseline | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-bam-mini | not_required |

## schmutzi

- Domains: bam
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: ancient_signal

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| bam | bam.contamination | Ancient Signal | Damage and Authenticity | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-adna-bam-mini | assigned |

## seqfu

- Domains: fastq
- Stage rows: 3
- Benchmark-ready rows: 2
- Blocked rows: 1
- Report sections: amplicon_interpretation, quality_profiling

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.normalize_abundance | Amplicon Interpretation | Amplicon Feature Tables | not_benchmark_ready | support | planned_contract | declared_only | not_normalized | fixture:corpus-03-amplicon-mini | not_required |
| fastq | fastq.profile_read_lengths | Quality Profiling | QC Signal Profiles | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| fastq | fastq.profile_reads | Quality Profiling | QC Signal Profiles | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |

## seqkit

- Domains: fastq
- Stage rows: 6
- Benchmark-ready rows: 6
- Blocked rows: 0
- Report sections: amplicon_interpretation, quality_profiling, read_cleanup

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.filter_reads | Read Cleanup | Cleanup and Retention | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| fastq | fastq.normalize_abundance | Amplicon Interpretation | Amplicon Feature Tables | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-03-amplicon-mini | not_required |
| fastq | fastq.profile_overrepresented_sequences | Quality Profiling | QC Signal Profiles | benchmark_ready | none | observer_specialized_benchmark | runnable | comparable | fixture:corpus-01-mini | not_required |
| fastq | fastq.profile_reads | Quality Profiling | QC Signal Profiles | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| fastq | fastq.trim_reads | Read Cleanup | Cleanup and Retention | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| fastq | fastq.trim_terminal_damage | Read Cleanup | Cleanup and Retention | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |

## seqkit_stats

- Domains: fastq
- Stage rows: 2
- Benchmark-ready rows: 2
- Blocked rows: 0
- Report sections: quality_profiling

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.profile_read_lengths | Quality Profiling | QC Signal Profiles | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| fastq | fastq.profile_reads | Quality Profiling | QC Signal Profiles | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |

## seqpurge

- Domains: fastq
- Stage rows: 1
- Benchmark-ready rows: 0
- Blocked rows: 1
- Report sections: read_cleanup

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.trim_reads | Read Cleanup | Cleanup and Retention | not_benchmark_ready | support | planned_contract | declared_only | not_normalized | fixture:corpus-01-mini | not_required |

## seqtk

- Domains: fastq
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: input_readiness

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.validate_reads | Input Readiness | Validation and Intake | benchmark_ready | none | observer_specialized_benchmark | runnable | comparable | fixture:corpus-01-mini | not_required |

## skewer

- Domains: fastq
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: read_cleanup

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.trim_reads | Read Cleanup | Cleanup and Retention | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |

## sortmerna

- Domains: fastq
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: contamination_screening

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.deplete_rrna | Contamination Screening | Screening and Contamination | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | assigned |

## star

- Domains: fastq
- Stage rows: 1
- Benchmark-ready rows: 0
- Blocked rows: 1
- Report sections: reference_preparation

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.index_reference | Reference Preparation | Reference Index Assets | not_benchmark_ready | corpus | observer_specialized_benchmark | runnable | comparable | planner_only | not_required |

## trim_galore

- Domains: fastq
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: read_cleanup

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.trim_reads | Read Cleanup | Cleanup and Retention | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |

## trimmomatic

- Domains: fastq
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: read_cleanup

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.trim_reads | Read Cleanup | Cleanup and Retention | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |

## umi_tools

- Domains: fastq
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: read_cleanup

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.extract_umis | Read Cleanup | Cleanup and Retention | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |

## verifybamid2

- Domains: bam
- Stage rows: 1
- Benchmark-ready rows: 1
- Blocked rows: 0
- Report sections: ancient_signal

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| bam | bam.contamination | Ancient Signal | Damage and Authenticity | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-adna-bam-mini | assigned |

## vsearch

- Domains: fastq
- Stage rows: 3
- Benchmark-ready rows: 3
- Blocked rows: 0
- Report sections: amplicon_interpretation, read_cleanup

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.cluster_otus | Amplicon Interpretation | Amplicon Feature Tables | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-03-amplicon-mini | not_required |
| fastq | fastq.merge_pairs | Read Cleanup | Cleanup and Retention | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-01-mini | not_required |
| fastq | fastq.remove_chimeras | Amplicon Interpretation | Amplicon Feature Tables | benchmark_ready | none | governed_benchmark_cohort | runnable | benchmark_normalized | fixture:corpus-03-amplicon-mini | not_required |

## yleaf

- Domains: bam
- Stage rows: 2
- Benchmark-ready rows: 2
- Blocked rows: 0
- Report sections: sample_identity

| Domain | Stage | Report section | Summary table | Benchmark status | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| bam | bam.haplogroups | Sample Identity | Identity and Relatedness | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-adna-bam-mini | assigned |
| bam | bam.sex | Sample Identity | Identity and Relatedness | benchmark_ready | none | supported | runnable | parser_fixture_validated | fixture:corpus-01-adna-bam-mini | assigned |
