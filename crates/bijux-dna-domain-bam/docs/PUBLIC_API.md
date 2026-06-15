# bijux-dna-domain-bam Public API

Public modules exported from `src/lib.rs`:
- alignment
- defaults
- invariants
- metrics
- params
- pipeline_contract
- prelude
- stage_specs
- types

Primary stage-spec helpers:
- `benchmark_corpus_assignment_for_stage_tool`
- `governed_benchmark_stage_tool_bindings`
- `BenchmarkCorpusFamily`
- `contract_for_stage`
- `required_audit_artifacts`
- `stage_contract_hash`
- `stage_contract_json`
- `stage_spec_opt`
- `stage_spec`
- `stage_specs`

Primary artifact and workflow helpers:
- `bam_alignment_strategies`
- `bam_alignment_strategy_for_tool`
- `bam_post_alignment_chain`
- `execute_bam_validation`
- `align_fastq_to_bam_bwa_style`
- `align_fastq_to_bam_bowtie2_style`
- `sort_and_index_tiny_bam`
- `propagate_bam_sample_identity`
- `evaluate_bam_merge_compatibility`
- `merge_tiny_bam_with_conflict_refusal`
- `apply_duplicate_policy_tiny_bam`
- `filter_tiny_bam_by_mapq`
- `summarize_tiny_bam_mapping`
- `compare_bam_duplicate_methods`
- `summarize_tiny_bam_coverage`
- `classify_bam_coverage_regime`
- `bam_adna_workflow_contract`
- `bam_contamination_workflow_contract`
- `bam_scientific_report_contracts`
- `bam_scientific_report_contract_for_stage`
- `estimate_bam_stage_resources`
- `bam_bench_corpus_manifest`
- `required_bam_bench_corpus_scenarios`

Primary catalogs:
- `BAM_STAGE_ID_CATALOG`
- `BAM_LOCAL_BENCH_STAGE_ID_CATALOG`
- `BAM_PARAMS_CATALOG`
- `BAM_METRICS_CATALOG`
