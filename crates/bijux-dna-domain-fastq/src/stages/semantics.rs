#![allow(dead_code)]

use crate::metrics::spec::{metric_spec_for_stage, MetricClass};
use crate::pipeline_contract::{self, StageCriticality};
use bijux_dna_core::ids::StageId;

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

pub const STAGE_BOUNDARY_INVARIANTS: [BoundaryInvariant; 6] = [
    BoundaryInvariant {
        from: StageId::from_static("fastq.validate_reads"),
        to: StageId::from_static("fastq.detect_adapters"),
        rule: "validation does not modify reads; adapter detection consumes validated reads",
    },
    BoundaryInvariant {
        from: StageId::from_static("fastq.detect_adapters"),
        to: StageId::from_static("fastq.damage_aware_pretrim"),
        rule: "damage-aware pretrim consumes unchanged reads from report-only adapter detection",
    },
    BoundaryInvariant {
        from: StageId::from_static("fastq.damage_aware_pretrim"),
        to: StageId::from_static("fastq.trim_reads"),
        rule: "damage-aware pretrim output remains FASTQ and preserves pairing semantics",
    },
    BoundaryInvariant {
        from: StageId::from_static("fastq.trim_reads"),
        to: StageId::from_static("fastq.filter_reads"),
        rule: "trim output must remain FASTQ and preserve pairing",
    },
    BoundaryInvariant {
        from: StageId::from_static("fastq.filter_reads"),
        to: StageId::from_static("fastq.profile_reads"),
        rule: "filter output remains FASTQ; stats is report-only",
    },
    BoundaryInvariant {
        from: StageId::from_static("fastq.merge"),
        to: StageId::from_static("fastq.profile_reads"),
        rule: "merge produces merged reads; stats accepts merged FASTQ",
    },
];

pub const STAGES: [StageDefinition; 25] = [
    StageDefinition {
        stage_id: StageId::from_static("fastq.validate_reads"),
        kind: FastqStageKind::Core,
        criticality: StageCriticality::Essential,
        semantics: StageSemantics {
            mutates_fastq: false,
            consumes_pairs: false,
            produces_reports_only: true,
            affects_metrics: &[MetricClass::Integrity],
        },
    },
    StageDefinition {
        stage_id: StageId::from_static("fastq.profile_read_lengths"),
        kind: FastqStageKind::Optional,
        criticality: StageCriticality::Optional,
        semantics: StageSemantics {
            mutates_fastq: false,
            consumes_pairs: false,
            produces_reports_only: true,
            affects_metrics: &[MetricClass::Integrity, MetricClass::Composition],
        },
    },
    StageDefinition {
        stage_id: StageId::from_static("fastq.detect_adapters"),
        kind: FastqStageKind::Core,
        criticality: StageCriticality::Essential,
        semantics: StageSemantics {
            mutates_fastq: false,
            consumes_pairs: false,
            produces_reports_only: true,
            affects_metrics: &[MetricClass::Composition],
        },
    },
    StageDefinition {
        stage_id: StageId::from_static("fastq.damage_aware_pretrim"),
        kind: FastqStageKind::Core,
        criticality: StageCriticality::Essential,
        semantics: StageSemantics {
            mutates_fastq: true,
            consumes_pairs: true,
            produces_reports_only: false,
            affects_metrics: &[MetricClass::Integrity, MetricClass::Retention],
        },
    },
    StageDefinition {
        stage_id: StageId::from_static("fastq.primer_normalization"),
        kind: FastqStageKind::Amplicon,
        criticality: StageCriticality::Essential,
        semantics: StageSemantics {
            mutates_fastq: true,
            consumes_pairs: true,
            produces_reports_only: false,
            affects_metrics: &[MetricClass::Integrity, MetricClass::Retention],
        },
    },
    StageDefinition {
        stage_id: StageId::from_static("fastq.polyg_tailing"),
        kind: FastqStageKind::Optional,
        criticality: StageCriticality::Optional,
        semantics: StageSemantics {
            mutates_fastq: true,
            consumes_pairs: true,
            produces_reports_only: false,
            affects_metrics: &[MetricClass::Integrity, MetricClass::Retention],
        },
    },
    StageDefinition {
        stage_id: StageId::from_static("fastq.trim_reads"),
        kind: FastqStageKind::Core,
        criticality: StageCriticality::Essential,
        semantics: StageSemantics {
            mutates_fastq: true,
            consumes_pairs: true,
            produces_reports_only: false,
            affects_metrics: &[
                MetricClass::Integrity,
                MetricClass::Retention,
                MetricClass::QualityShift,
            ],
        },
    },
    StageDefinition {
        stage_id: StageId::from_static("fastq.filter_reads"),
        kind: FastqStageKind::Core,
        criticality: StageCriticality::Essential,
        semantics: StageSemantics {
            mutates_fastq: true,
            consumes_pairs: true,
            produces_reports_only: false,
            affects_metrics: &[
                MetricClass::Integrity,
                MetricClass::Retention,
                MetricClass::QualityShift,
            ],
        },
    },
    StageDefinition {
        stage_id: StageId::from_static("fastq.profile_reads"),
        kind: FastqStageKind::Core,
        criticality: StageCriticality::Essential,
        semantics: StageSemantics {
            mutates_fastq: false,
            consumes_pairs: false,
            produces_reports_only: true,
            affects_metrics: &[MetricClass::Integrity, MetricClass::Composition],
        },
    },
    StageDefinition {
        stage_id: StageId::from_static("fastq.rrna"),
        kind: FastqStageKind::Optional,
        criticality: StageCriticality::Optional,
        semantics: StageSemantics {
            mutates_fastq: true,
            consumes_pairs: true,
            produces_reports_only: false,
            affects_metrics: &[MetricClass::Contamination, MetricClass::Retention],
        },
    },
    StageDefinition {
        stage_id: StageId::from_static("fastq.merge"),
        kind: FastqStageKind::Core,
        criticality: StageCriticality::Essential,
        semantics: StageSemantics {
            mutates_fastq: true,
            consumes_pairs: true,
            produces_reports_only: false,
            affects_metrics: &[MetricClass::Integrity, MetricClass::Retention],
        },
    },
    StageDefinition {
        stage_id: StageId::from_static("fastq.deduplicate"),
        kind: FastqStageKind::Optional,
        criticality: StageCriticality::Optional,
        semantics: StageSemantics {
            mutates_fastq: true,
            consumes_pairs: true,
            produces_reports_only: false,
            affects_metrics: &[
                MetricClass::Integrity,
                MetricClass::Retention,
                MetricClass::QualityShift,
            ],
        },
    },
    StageDefinition {
        stage_id: StageId::from_static("fastq.low_complexity"),
        kind: FastqStageKind::Optional,
        criticality: StageCriticality::Optional,
        semantics: StageSemantics {
            mutates_fastq: true,
            consumes_pairs: true,
            produces_reports_only: false,
            affects_metrics: &[
                MetricClass::Integrity,
                MetricClass::Retention,
                MetricClass::QualityShift,
            ],
        },
    },
    StageDefinition {
        stage_id: StageId::from_static("fastq.host_depletion"),
        kind: FastqStageKind::Optional,
        criticality: StageCriticality::Optional,
        semantics: StageSemantics {
            mutates_fastq: true,
            consumes_pairs: true,
            produces_reports_only: false,
            affects_metrics: &[MetricClass::Contamination, MetricClass::Retention],
        },
    },
    StageDefinition {
        stage_id: StageId::from_static("fastq.contaminant_screen"),
        kind: FastqStageKind::Optional,
        criticality: StageCriticality::Optional,
        semantics: StageSemantics {
            mutates_fastq: true,
            consumes_pairs: true,
            produces_reports_only: false,
            affects_metrics: &[MetricClass::Contamination, MetricClass::Retention],
        },
    },
    StageDefinition {
        stage_id: StageId::from_static("fastq.correct"),
        kind: FastqStageKind::Core,
        criticality: StageCriticality::Essential,
        semantics: StageSemantics {
            mutates_fastq: true,
            consumes_pairs: true,
            produces_reports_only: false,
            affects_metrics: &[MetricClass::Integrity, MetricClass::QualityShift],
        },
    },
    StageDefinition {
        stage_id: StageId::from_static("fastq.umi"),
        kind: FastqStageKind::Optional,
        criticality: StageCriticality::Optional,
        semantics: StageSemantics {
            mutates_fastq: true,
            consumes_pairs: true,
            produces_reports_only: false,
            affects_metrics: &[MetricClass::Integrity, MetricClass::Retention],
        },
    },
    StageDefinition {
        stage_id: StageId::from_static("fastq.profile_overrepresented_sequences"),
        kind: FastqStageKind::Optional,
        criticality: StageCriticality::Optional,
        semantics: StageSemantics {
            mutates_fastq: false,
            consumes_pairs: false,
            produces_reports_only: true,
            affects_metrics: &[MetricClass::Composition],
        },
    },
    StageDefinition {
        stage_id: StageId::from_static("fastq.chimera_detection"),
        kind: FastqStageKind::Amplicon,
        criticality: StageCriticality::Essential,
        semantics: StageSemantics {
            mutates_fastq: true,
            consumes_pairs: false,
            produces_reports_only: false,
            affects_metrics: &[MetricClass::Integrity, MetricClass::Retention],
        },
    },
    StageDefinition {
        stage_id: StageId::from_static("fastq.asv_inference"),
        kind: FastqStageKind::Amplicon,
        criticality: StageCriticality::Optional,
        semantics: StageSemantics {
            mutates_fastq: false,
            consumes_pairs: false,
            produces_reports_only: false,
            affects_metrics: &[MetricClass::Composition],
        },
    },
    StageDefinition {
        stage_id: StageId::from_static("fastq.otu_clustering"),
        kind: FastqStageKind::Amplicon,
        criticality: StageCriticality::Optional,
        semantics: StageSemantics {
            mutates_fastq: false,
            consumes_pairs: false,
            produces_reports_only: false,
            affects_metrics: &[MetricClass::Composition],
        },
    },
    StageDefinition {
        stage_id: StageId::from_static("fastq.abundance_normalization"),
        kind: FastqStageKind::Amplicon,
        criticality: StageCriticality::Essential,
        semantics: StageSemantics {
            mutates_fastq: false,
            consumes_pairs: false,
            produces_reports_only: false,
            affects_metrics: &[MetricClass::Composition],
        },
    },
    StageDefinition {
        stage_id: StageId::from_static("fastq.screen_taxonomy"),
        kind: FastqStageKind::Optional,
        criticality: StageCriticality::Experimental,
        semantics: StageSemantics {
            mutates_fastq: false,
            consumes_pairs: false,
            produces_reports_only: true,
            affects_metrics: &[MetricClass::Contamination],
        },
    },
    StageDefinition {
        stage_id: StageId::from_static("fastq.report_qc"),
        kind: FastqStageKind::Optional,
        criticality: StageCriticality::Optional,
        semantics: StageSemantics {
            mutates_fastq: false,
            consumes_pairs: false,
            produces_reports_only: true,
            affects_metrics: &[MetricClass::QualityShift, MetricClass::Contamination],
        },
    },
    StageDefinition {
        stage_id: StageId::from_static("fastq.prepare_reference"),
        kind: FastqStageKind::Meta,
        criticality: StageCriticality::Optional,
        semantics: StageSemantics {
            mutates_fastq: false,
            consumes_pairs: false,
            produces_reports_only: false,
            affects_metrics: &[],
        },
    },
];

#[must_use]
pub fn stage_semantics(stage_id: &StageId) -> Option<StageSemantics> {
    STAGES
        .iter()
        .find(|stage| stage.stage_id.as_str() == stage_id.as_str())
        .map(|stage| stage.semantics)
}

#[must_use]
pub fn stage_kind(stage_id: &StageId) -> Option<FastqStageKind> {
    STAGES
        .iter()
        .find(|stage| stage.stage_id.as_str() == stage_id.as_str())
        .map(|stage| stage.kind)
}

#[must_use]
pub fn stage_criticality(stage_id: &StageId) -> Option<StageCriticality> {
    STAGES
        .iter()
        .find(|stage| stage.stage_id.as_str() == stage_id.as_str())
        .map(|stage| stage.criticality)
}

#[must_use]
pub fn fastq_stage_is_stable(stage_id: &StageId) -> bool {
    !matches!(
        stage_criticality(stage_id),
        Some(StageCriticality::Experimental)
    )
}

#[must_use]
pub fn stage_metric_classes(stage_id: &StageId) -> Option<&'static [MetricClass]> {
    stage_semantics(stage_id).map(|semantics| semantics.affects_metrics)
}

#[must_use]
pub fn stage_metric_invariants(stage_id: &StageId) -> Option<&'static [&'static str]> {
    metric_spec_for_stage(stage_id.as_str()).map(|spec| spec.invariants)
}

#[must_use]
pub fn canonical_stage_order() -> Vec<StageId> {
    pipeline_contract::canonical_stage_order()
}

#[must_use]
pub fn optional_branches() -> Vec<(StageId, Vec<StageId>)> {
    pipeline_contract::optional_branches()
}
