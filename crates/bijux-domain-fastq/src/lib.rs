//! FASTQ domain definitions and contracts.
//!
//! Owns: FASTQ stage semantics, invariants, and contracts.
//! Must NOT depend on: bijux-engine or runtime/container execution logic.
// Reading order:
// 1. domain.rs
// 2. core types
// 3. stage semantics
// 4. metrics spec
// 5. execution adapters
// Structural layout of this crate is frozen as of FASTQ v1.
mod adapter;
mod adapter_bank;
mod contaminant_bank;
mod metrics;
pub mod params;
mod polyx_bank;
pub mod stage_registry;
pub mod invariants;
pub mod pipeline_contract;
pub mod run;
pub mod types;
mod stages;

pub use adapter_bank::{
    adapter_bank_path, adapter_categories, adapter_presets_path, adapters_by_category,
    load_adapter_bank, load_adapter_presets, resolve_adapter_preset, AdapterBankV1, AdapterEntryV1,
    AdapterPresetV1, AdapterPresetsV1, EffectiveAdapterSet, ReadScope,
};
pub use contaminant_bank::{
    contaminant_motifs_path, contaminant_presets_path, contaminant_references_dir,
    load_contaminant_motifs, load_contaminant_presets, resolve_contaminant_preset,
    ContaminantMotifBankV1, ContaminantMotifEntryV1, ContaminantPresetV1, ContaminantPresetsV1,
    ContaminantReferenceSpecV1, EffectiveContaminantSet,
};
pub use stage_registry::{
    canonical_contract_for_stage, contract_for_stage, infer_input_kind, qc_class_for_stage,
    fastq_stage_is_stable, stage_criticality, stage_kind, stage_metric_classes,
    stage_metric_invariants, stage_semantics,
    assess_merge_suitability, ensure_umi_headers, inspect_headers, log_header_warnings,
    normalize_outputs, preflight_stage, BoundaryInvariant, FastqStage, FastqStageContract,
    FastqStageKind, HeaderInspection, MergeSuitability, NormalizedOutputs, QcClass,
    StageContract, StageDefinition, StageIO, StageSemantics, STAGE_BOUNDARY_INVARIANTS, STAGES,
};
pub use pipeline_contract::{
    canonical_stage_order, forbidden_transitions, optional_branches, StageCriticality,
};
pub use types::{
    AdapterContributionV1, AdapterTrimmingReportV1, FastqArtifact, FastqArtifactKind, FastqLayout,
    FastqPE, FastqPairedEnd, FastqSE, FastqSampleId, FastqSingleEnd, FastqStats, RetentionReportV1,
    ToolReferenceV1,
};
pub use bijux_core::RawFailure;
pub use invariants::{
    evaluate_invariants, thresholds_from_env, InvariantEvaluation, InvariantThresholds,
};
pub use metrics::deltas::{compute_delta, ratio_u64};
pub use params::{parse_effective_params, EffectiveParams, PairedMode};
pub use run::query::get_results;
pub use run::{
    append_event, bench_corpus, benchmark_runs, create_run_layout, now_string, update_run_index,
    write_benchmark_exports, write_environment, write_input_assessment, write_manifest,
    write_run_metadata, BenchCorpus, BenchCorpusId, BenchDataset, InputAssessmentV1,
    RunArtifactEntry, RunEnvironment, RunIndexEntry, RunLayout, RunManifest, RunStageEntry,
    ToolImageDigest,
};
pub use run::{assess_input_dir, discover_fastq_files};
pub use polyx_bank::{
    load_polyx_bank, load_polyx_presets, polyx_bank_path, polyx_presets_path, resolve_polyx_preset,
    EffectivePolyxSet, PolyxBankV1, PolyxEntryV1, PolyxPresetV1, PolyxPresetsV1,
};
pub use stages::args;
