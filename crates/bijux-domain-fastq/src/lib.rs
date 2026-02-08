//! FASTQ domain definitions and contracts.
//!
//! Owns: FASTQ stage semantics, invariants, and contracts.
//! Must NOT depend on: bijux-engine or runtime/container execution logic.
// Reading order:
// 1. domain.rs
// 2. core types
// 3. stage semantics
// 4. metrics spec
// 5. domain adapter
// Structural layout of this crate is frozen as of FASTQ v1.
pub mod banks;
pub mod bench_repository;
mod domain_adapter;
pub mod invariants;
pub mod metrics;
pub mod params;
pub mod pipeline_contract;
pub mod prelude;
pub mod run;
pub mod stage_contract;
pub mod stage_semantics;
pub mod stage_specs;
pub mod stages;
pub mod types;

pub use banks::{
    adapter_bank_path, adapter_categories, adapter_presets_path, adapters_by_category,
    load_adapter_bank, load_adapter_presets, resolve_adapter_preset, AdapterBankV1, AdapterEntryV1,
    AdapterPresetV1, AdapterPresetsV1, EffectiveAdapterSet, ReadScope,
};
pub use banks::{
    contaminant_motifs_path, contaminant_presets_path, contaminant_references_dir,
    load_contaminant_motifs, load_contaminant_presets, resolve_contaminant_preset,
    ContaminantMotifBankV1, ContaminantMotifEntryV1, ContaminantPresetV1, ContaminantPresetsV1,
    ContaminantReferenceSpecV1, EffectiveContaminantSet,
};
pub use banks::{
    load_polyx_bank, load_polyx_presets, polyx_bank_path, polyx_presets_path, resolve_polyx_preset,
    EffectivePolyxSet, PolyxBankV1, PolyxEntryV1, PolyxPresetV1, PolyxPresetsV1,
};
pub use bench_repository::BenchResultsRepository;
pub use invariants::{
    evaluate_invariants, fastq_invariant_specs, thresholds_from_env, InvariantEvaluation,
    InvariantThresholds,
};
pub use params::{parse_effective_params, EffectiveParams, PairedMode};
pub use pipeline_contract::{
    canonical_stage_order, forbidden_transitions, optional_branches, StageCriticality,
};
pub use run::{assess_input_dir, discover_fastq_files};
pub use run::{bench_corpus, BenchCorpus, BenchCorpusId, BenchDataset};
pub use stages::{
    assess_merge_suitability, contract_for_stage, ensure_umi_headers, inspect_headers,
    log_header_warnings, normalize_outputs, preflight_stage, stage_contract_hash,
    stage_contract_json, HeaderInspection, MergeSuitability, NormalizedOutputs,
};
pub use stages::{
    bench_dir_name, STAGES, STAGE_CORRECT, STAGE_DETECT_ADAPTERS, STAGE_FILTER, STAGE_MERGE,
    STAGE_PREFIX, STAGE_PREPROCESS, STAGE_QC_POST, STAGE_RRNA, STAGE_SCREEN, STAGE_STATS_NEUTRAL,
    STAGE_TRIM, STAGE_UMI, STAGE_VALIDATE_PRE,
};
pub use stages::{
    canonical_contract_for_stage, infer_input_kind, qc_class_for_stage, FastqStage,
    FastqStageContract, QcClass, StageContract, StageIO,
};
pub use stages::{
    fastq_stage_is_stable, stage_criticality, stage_kind, stage_metric_classes,
    stage_metric_invariants, stage_semantics, BoundaryInvariant, FastqStageKind, StageDefinition,
    StageSemantics, STAGE_BOUNDARY_INVARIANTS,
};
pub use types::{
    AdapterContributionV1, AdapterTrimmingReportV1, FastqArtifact, FastqArtifactKind, FastqLayout,
    FastqPE, FastqPairedEnd, FastqSE, FastqSampleId, FastqSingleEnd, FastqStats, RetentionReportV1,
    ToolReferenceV1,
};
