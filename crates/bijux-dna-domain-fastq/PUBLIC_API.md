# bijux-dna-domain-fastq Public API

Public modules exported from src/lib.rs:
- banks
- bench_repository
- execution_support
- id_catalog
- invariants
- metrics
- observer
- params
- pipeline_contract
- prelude
- run
- stage_contract
- stage_semantics
- stage_specs
- stages
- types

Important re-export groups:
- Stage and pipeline contracts: `canonical_stage_order`, `canonical_amplicon_stage_order`,
  `default_shotgun_preprocess_stage_order`, `preprocess_pipeline_graph_for_stage_order`,
  `contract_for_stage`, `stage_contract_json`, and `stage_contract_hash`.
- Bank contracts: adapter, contaminant, and polyX bank loaders, preset resolvers, path helpers,
  effective selection types, and provenance context builders.
- Parameter contracts: `stage_param_descriptor`, `parse_effective_params`, trim, UMI, stats,
  correction, and profile parameter types.
- Metrics and invariants: FASTQ QC summaries, tool metrics, classification metrics,
  `fastq_invariant_specs`, and `evaluate_invariants`.
- Execution support and governance: execution support catalogs, stage-tool governance profiles,
  benchmark readiness, and input-layout filtering.
- Runtime-facing FASTQ discovery: `assess_input_dir`, `discover_fastq_files`, and bench corpus
  descriptors.
