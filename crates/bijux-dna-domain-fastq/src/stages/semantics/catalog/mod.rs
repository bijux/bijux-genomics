#![allow(dead_code)]

mod amplicon;
mod boundaries;
mod cleanup;
mod screening;
mod transforms;

use crate::metrics::spec::MetricClass;
use crate::pipeline_contract::StageCriticality;
use bijux_dna_core::ids::StageId;

pub use boundaries::STAGE_BOUNDARY_INVARIANTS;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FastqStageKind {
    Core,
    Optional,
    Meta,
    Amplicon,
}

#[derive(Debug, Clone, Copy)]
pub struct StageSemantics {
    pub mutates_fastq: bool,
    pub consumes_pairs: bool,
    pub produces_reports_only: bool,
    pub affects_metrics: &'static [MetricClass],
}

#[derive(Debug, Clone)]
pub struct StageDefinition {
    pub stage_id: StageId,
    pub kind: FastqStageKind,
    pub criticality: StageCriticality,
    pub semantics: StageSemantics,
}

#[derive(Debug, Clone)]
pub struct BoundaryInvariant {
    pub from: StageId,
    pub to: StageId,
    pub rule: &'static str,
}

const fn stage_definition(
    stage_id: &'static str,
    kind: FastqStageKind,
    criticality: StageCriticality,
    mutates_fastq: bool,
    consumes_pairs: bool,
    produces_reports_only: bool,
    affects_metrics: &'static [MetricClass],
) -> StageDefinition {
    StageDefinition {
        stage_id: StageId::from_static(stage_id),
        kind,
        criticality,
        semantics: StageSemantics {
            mutates_fastq,
            consumes_pairs,
            produces_reports_only,
            affects_metrics,
        },
    }
}

pub const STAGES: [StageDefinition; 27] = [
    cleanup::VALIDATE_READS,
    cleanup::PROFILE_READ_LENGTHS,
    cleanup::DETECT_ADAPTERS,
    cleanup::DETECT_DUPLICATES_PREMERGE,
    cleanup::ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN,
    cleanup::TRIM_TERMINAL_DAMAGE,
    amplicon::NORMALIZE_PRIMERS,
    cleanup::TRIM_POLYG_TAILS,
    cleanup::TRIM_READS,
    cleanup::FILTER_READS,
    cleanup::PROFILE_READS,
    screening::DEPLETE_RRNA,
    transforms::MERGE_PAIRS,
    transforms::REMOVE_DUPLICATES,
    transforms::FILTER_LOW_COMPLEXITY,
    screening::DEPLETE_HOST,
    screening::DEPLETE_REFERENCE_CONTAMINANTS,
    transforms::CORRECT_ERRORS,
    transforms::EXTRACT_UMIS,
    cleanup::PROFILE_OVERREPRESENTED_SEQUENCES,
    amplicon::REMOVE_CHIMERAS,
    amplicon::INFER_ASVS,
    amplicon::CLUSTER_OTUS,
    amplicon::NORMALIZE_ABUNDANCE,
    screening::SCREEN_TAXONOMY,
    cleanup::REPORT_QC,
    screening::INDEX_REFERENCE,
];
