pub use crate::stages::{
    assess_merge_suitability, contract_for_stage, ensure_umi_headers, inspect_headers,
    log_header_warnings, normalize_outputs, preflight_stage, stage_contract_hash,
    stage_contract_json, HeaderInspection, MergeSuitability, NormalizedOutputs,
};
pub use crate::stages::{
    bench_dir_name, STAGES, STAGE_CLUSTER_OTUS, STAGE_CORRECT_ERRORS, STAGE_DEPLETE_RRNA,
    STAGE_DETECT_ADAPTERS, STAGE_EXTRACT_UMIS, STAGE_FILTER_LOW_COMPLEXITY, STAGE_FILTER_READS,
    STAGE_INFER_ASVS, STAGE_MERGE_PAIRS, STAGE_NORMALIZE_ABUNDANCE, STAGE_NORMALIZE_PRIMERS,
    STAGE_PREFIX, STAGE_PROFILE_READS, STAGE_REMOVE_CHIMERAS, STAGE_REMOVE_DUPLICATES,
    STAGE_REPORT_QC, STAGE_SCREEN_TAXONOMY, STAGE_TRIM_READS, STAGE_TRIM_TERMINAL_DAMAGE,
    STAGE_VALIDATE_READS,
};
pub use crate::stages::{
    canonical_contract_for_stage, infer_input_kind, qc_class_for_stage, FastqStage,
    FastqStageContract, QcClass, StageContract, StageIO,
};
pub use crate::stages::{
    fastq_stage_is_stable, stage_compatible_tool_ids, stage_criticality, stage_input_ids,
    stage_kind, stage_metric_classes, stage_metric_invariants, stage_output_ids,
    stage_parameter_ids, stage_semantics, BoundaryInvariant, FastqStageKind, StageDefinition,
    StageSemantics, STAGE_BOUNDARY_INVARIANTS,
};
