#![allow(dead_code)]

use crate::metrics::spec::MetricClass;
use crate::pipeline_contract::StageCriticality;
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
        to: StageId::from_static("fastq.trim_terminal_damage"),
        rule: "damage-aware pretrim consumes unchanged reads from report-only adapter detection",
    },
    BoundaryInvariant {
        from: StageId::from_static("fastq.trim_terminal_damage"),
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
        from: StageId::from_static("fastq.merge_pairs"),
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
        criticality: StageCriticality::Essential,
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
        stage_id: StageId::from_static("fastq.trim_terminal_damage"),
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
        stage_id: StageId::from_static("fastq.normalize_primers"),
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
        stage_id: StageId::from_static("fastq.trim_polyg_tails"),
        kind: FastqStageKind::Optional,
        criticality: StageCriticality::Essential,
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
        stage_id: StageId::from_static("fastq.deplete_rrna"),
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
        stage_id: StageId::from_static("fastq.merge_pairs"),
        kind: FastqStageKind::Core,
        criticality: StageCriticality::Optional,
        semantics: StageSemantics {
            mutates_fastq: true,
            consumes_pairs: true,
            produces_reports_only: false,
            affects_metrics: &[MetricClass::Integrity, MetricClass::Retention],
        },
    },
    StageDefinition {
        stage_id: StageId::from_static("fastq.remove_duplicates"),
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
        stage_id: StageId::from_static("fastq.filter_low_complexity"),
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
        stage_id: StageId::from_static("fastq.deplete_host"),
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
        stage_id: StageId::from_static("fastq.deplete_reference_contaminants"),
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
        stage_id: StageId::from_static("fastq.correct_errors"),
        kind: FastqStageKind::Core,
        criticality: StageCriticality::Optional,
        semantics: StageSemantics {
            mutates_fastq: true,
            consumes_pairs: true,
            produces_reports_only: false,
            affects_metrics: &[MetricClass::Integrity, MetricClass::QualityShift],
        },
    },
    StageDefinition {
        stage_id: StageId::from_static("fastq.extract_umis"),
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
        criticality: StageCriticality::Essential,
        semantics: StageSemantics {
            mutates_fastq: false,
            consumes_pairs: false,
            produces_reports_only: true,
            affects_metrics: &[MetricClass::Composition],
        },
    },
    StageDefinition {
        stage_id: StageId::from_static("fastq.remove_chimeras"),
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
        stage_id: StageId::from_static("fastq.infer_asvs"),
        kind: FastqStageKind::Amplicon,
        criticality: StageCriticality::Experimental,
        semantics: StageSemantics {
            mutates_fastq: false,
            consumes_pairs: false,
            produces_reports_only: false,
            affects_metrics: &[MetricClass::Composition],
        },
    },
    StageDefinition {
        stage_id: StageId::from_static("fastq.cluster_otus"),
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
        stage_id: StageId::from_static("fastq.normalize_abundance"),
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
        criticality: StageCriticality::Optional,
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
        criticality: StageCriticality::Essential,
        semantics: StageSemantics {
            mutates_fastq: false,
            consumes_pairs: false,
            produces_reports_only: true,
            affects_metrics: &[MetricClass::QualityShift, MetricClass::Contamination],
        },
    },
    StageDefinition {
        stage_id: StageId::from_static("fastq.index_reference"),
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
