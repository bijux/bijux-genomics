# BAM 2xx Common Contracts

This directory defines shared interpretation assumptions for BAM 2xx examples.

## Interpretation Contracts
- MAPQ assumptions must be stated per stage and tool.
- Duplicate handling assumptions must be explicit (markdup vs remove).
- Endogenous content interpretation must reference denominator definition.
- BAM validity checks require BAM + BAI consistency.

## Report Contract Sections
BAM examples should include minimal snapshots aligned with analyze report contract sections:
- contract
- completeness
- stages
- provenance
- run_provenance
- decision_config
- analysis_selection_contract
- retention_definition
- retention_context
- assets_provenance
- metric_semantics
- telemetry
- qc_improvement
- final_qc_summary
- filter_interpretation
- adapter_inference
- claims_registry
