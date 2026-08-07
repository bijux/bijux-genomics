<!-- GENERATED FILE - DO NOT EDIT -->
<!-- source: bijux-dna-bench::workflow::stage_universe -->
# Canonical stage universe

Every domain stage has an explicit benchmark disposition. The domain YAML stage contract is the authority; this index binds benchmark-required stages to permanent run IDs.

- Domain stages: **89**
- Benchmark required: **71**
- Qualification only: **9**
- Orchestration only: **9**
- Test only: **0**
- Retired: **0**
- Excluded: **0**

| Stage | Status | Disposition | Run | Rationale |
|---|---|---|---|---|
| `bam.align` | `supported` | `benchmark_required` | `run-2005` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `bam.authenticity` | `supported` | `benchmark_required` | `run-2006` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `bam.bias_mitigation` | `supported` | `benchmark_required` | `run-2007` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `bam.complexity` | `supported` | `benchmark_required` | `run-2008` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `bam.contamination` | `supported` | `benchmark_required` | `run-2009` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `bam.coverage` | `supported` | `benchmark_required` | `run-2002` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `bam.damage` | `supported` | `benchmark_required` | `run-2010` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `bam.duplication_metrics` | `supported` | `benchmark_required` | `run-2011` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `bam.endogenous_content` | `supported` | `benchmark_required` | `run-2003` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `bam.filter` | `supported` | `benchmark_required` | `run-2012` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `bam.gc_bias` | `planned` | `benchmark_required` | `run-2013` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `bam.genotyping` | `supported` | `benchmark_required` | `run-2001` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `bam.haplogroups` | `supported` | `benchmark_required` | `run-2014` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `bam.insert_size` | `planned` | `benchmark_required` | `run-2015` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `bam.kinship` | `supported` | `benchmark_required` | `run-2016` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `bam.length_filter` | `supported` | `benchmark_required` | `run-2017` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `bam.mapping_summary` | `supported` | `benchmark_required` | `run-2004` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `bam.mapq_filter` | `supported` | `benchmark_required` | `run-2018` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `bam.markdup` | `planned` | `benchmark_required` | `run-2019` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `bam.overlap_correction` | `supported` | `benchmark_required` | `run-2020` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `bam.qc_pre` | `supported` | `benchmark_required` | `run-2021` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `bam.recalibration` | `supported` | `benchmark_required` | `run-2022` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `bam.sex` | `supported` | `benchmark_required` | `run-2023` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `bam.validate` | `supported` | `benchmark_required` | `run-2024` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `fastq.build_contaminant_db` | `planned` | `orchestration_only` | - | This preparation or provenance operation supports benchmark orchestration and is not an independently measured scientific stage. |
| `fastq.build_rrna_db` | `planned` | `orchestration_only` | - | This preparation or provenance operation supports benchmark orchestration and is not an independently measured scientific stage. |
| `fastq.build_taxonomy_db` | `planned` | `orchestration_only` | - | This preparation or provenance operation supports benchmark orchestration and is not an independently measured scientific stage. |
| `fastq.capture_provenance_snapshot` | `planned` | `orchestration_only` | - | This preparation or provenance operation supports benchmark orchestration and is not an independently measured scientific stage. |
| `fastq.classify_layout` | `planned` | `qualification_only` | - | This utility transformation requires contract qualification but does not support a standalone scientific benchmark claim. |
| `fastq.cluster_otus` | `supported` | `benchmark_required` | `run-2025` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `fastq.concatenate_lanes` | `planned` | `qualification_only` | - | This utility transformation requires contract qualification but does not support a standalone scientific benchmark claim. |
| `fastq.correct_errors` | `supported` | `benchmark_required` | `run-2026` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `fastq.deinterleave_reads` | `planned` | `qualification_only` | - | This utility transformation requires contract qualification but does not support a standalone scientific benchmark claim. |
| `fastq.demultiplex_reads` | `planned` | `qualification_only` | - | This utility transformation requires contract qualification but does not support a standalone scientific benchmark claim. |
| `fastq.deplete_host` | `supported` | `benchmark_required` | `run-2027` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `fastq.deplete_reference_contaminants` | `supported` | `benchmark_required` | `run-2028` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `fastq.deplete_rrna` | `supported` | `benchmark_required` | `run-2029` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `fastq.detect_adapters` | `supported` | `benchmark_required` | `run-2030` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `fastq.detect_duplicates_premerge` | `supported` | `benchmark_required` | `run-2031` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `fastq.detect_instrument_artifacts` | `planned` | `qualification_only` | - | This utility transformation requires contract qualification but does not support a standalone scientific benchmark claim. |
| `fastq.estimate_library_complexity_prealign` | `supported` | `benchmark_required` | `run-2032` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `fastq.extract_umis` | `supported` | `benchmark_required` | `run-2033` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `fastq.filter_low_complexity` | `supported` | `benchmark_required` | `run-2034` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `fastq.filter_reads` | `supported` | `benchmark_required` | `run-2035` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `fastq.index_reference` | `supported` | `benchmark_required` | `run-2036` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `fastq.infer_asvs` | `supported` | `benchmark_required` | `run-2037` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `fastq.interleave_reads` | `planned` | `qualification_only` | - | This utility transformation requires contract qualification but does not support a standalone scientific benchmark claim. |
| `fastq.materialize_qc_manifest` | `planned` | `orchestration_only` | - | This preparation or provenance operation supports benchmark orchestration and is not an independently measured scientific stage. |
| `fastq.merge_pairs` | `supported` | `benchmark_required` | `run-2038` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `fastq.normalize_abundance` | `supported` | `benchmark_required` | `run-2039` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `fastq.normalize_primers` | `supported` | `benchmark_required` | `run-2040` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `fastq.normalize_read_names` | `planned` | `qualification_only` | - | This utility transformation requires contract qualification but does not support a standalone scientific benchmark claim. |
| `fastq.prepare_adapter_bank` | `planned` | `orchestration_only` | - | This preparation or provenance operation supports benchmark orchestration and is not an independently measured scientific stage. |
| `fastq.prepare_host_reference_bundle` | `planned` | `orchestration_only` | - | This preparation or provenance operation supports benchmark orchestration and is not an independently measured scientific stage. |
| `fastq.prepare_primer_bank` | `planned` | `orchestration_only` | - | This preparation or provenance operation supports benchmark orchestration and is not an independently measured scientific stage. |
| `fastq.profile_overrepresented_sequences` | `supported` | `benchmark_required` | `run-2041` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `fastq.profile_read_lengths` | `supported` | `benchmark_required` | `run-2042` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `fastq.profile_reads` | `supported` | `benchmark_required` | `run-2043` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `fastq.remove_chimeras` | `supported` | `benchmark_required` | `run-2044` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `fastq.remove_duplicates` | `supported` | `benchmark_required` | `run-2045` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `fastq.repair_pairs` | `planned` | `qualification_only` | - | This utility transformation requires contract qualification but does not support a standalone scientific benchmark claim. |
| `fastq.report_qc` | `supported` | `benchmark_required` | `run-2046` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `fastq.screen_taxonomy` | `supported` | `benchmark_required` | `run-2047` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `fastq.subsample_reads` | `planned` | `qualification_only` | - | This utility transformation requires contract qualification but does not support a standalone scientific benchmark claim. |
| `fastq.trim_polyg_tails` | `supported` | `benchmark_required` | `run-2048` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `fastq.trim_reads` | `supported` | `benchmark_required` | `run-2049` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `fastq.trim_terminal_damage` | `supported` | `benchmark_required` | `run-2050` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `fastq.validate_reads` | `supported` | `benchmark_required` | `run-2051` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `fastq.verify_assets` | `planned` | `orchestration_only` | - | This preparation or provenance operation supports benchmark orchestration and is not an independently measured scientific stage. |
| `vcf.admixture` | `supported` | `benchmark_required` | `run-2052` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `vcf.call` | `supported` | `benchmark_required` | `run-2053` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `vcf.call_diploid` | `supported` | `benchmark_required` | `run-2054` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `vcf.call_gl` | `supported` | `benchmark_required` | `run-2055` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `vcf.call_pseudohaploid` | `supported` | `benchmark_required` | `run-2056` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `vcf.damage_filter` | `supported` | `benchmark_required` | `run-2057` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `vcf.demography` | `supported` | `benchmark_required` | `run-2058` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `vcf.filter` | `supported` | `benchmark_required` | `run-2059` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `vcf.gl_propagation` | `supported` | `benchmark_required` | `run-2060` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `vcf.ibd` | `supported` | `benchmark_required` | `run-2061` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `vcf.imputation_metrics` | `supported` | `benchmark_required` | `run-2062` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `vcf.impute` | `supported` | `benchmark_required` | `run-2063` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `vcf.pca` | `supported` | `benchmark_required` | `run-2064` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `vcf.phasing` | `supported` | `benchmark_required` | `run-2065` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `vcf.population_structure` | `supported` | `benchmark_required` | `run-2066` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `vcf.postprocess` | `supported` | `benchmark_required` | `run-2067` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `vcf.prepare_reference_panel` | `supported` | `benchmark_required` | `run-2068` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `vcf.qc` | `supported` | `benchmark_required` | `run-2069` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `vcf.roh` | `supported` | `benchmark_required` | `run-2070` | The stage has a permanent run reservation in the governed benchmark portfolio. |
| `vcf.stats` | `supported` | `benchmark_required` | `run-2071` | The stage has a permanent run reservation in the governed benchmark portfolio. |
