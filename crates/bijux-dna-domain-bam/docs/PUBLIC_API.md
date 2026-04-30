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
- `evaluate_bam_merge_compatibility`
- `compare_bam_duplicate_methods`
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
- `BAM_PARAMS_CATALOG`
- `BAM_METRICS_CATALOG`
