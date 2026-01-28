use crate::metrics::spec::{metric_spec_for_stage, MetricClass};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FastqStageKind {
    Core,
    Optional,
    Meta,
}

#[derive(Debug, Clone, Copy)]
pub struct StageSemantics {
    pub mutates_fastq: bool,
    pub consumes_pairs: bool,
    pub produces_reports_only: bool,
    pub affects_metrics: &'static [MetricClass],
}

#[derive(Debug, Clone, Copy)]
pub struct StageDefinition {
    pub stage_id: &'static str,
    pub kind: FastqStageKind,
    pub semantics: StageSemantics,
}

#[derive(Debug, Clone, Copy)]
pub struct BoundaryInvariant {
    pub from: &'static str,
    pub to: &'static str,
    pub rule: &'static str,
}

pub const CANONICAL_STAGE_ORDER: [&str; 4] = [
    "fastq.validate",
    "fastq.trim",
    "fastq.filter",
    "fastq.stats",
];

pub const OPTIONAL_BRANCHES: [(&str, &[&str]); 5] = [
    ("fastq.umi", &["fastq.trim"]),
    ("fastq.screen", &["fastq.validate"]),
    ("fastq.qc_post", &["fastq.validate"]),
    ("fastq.merge", &["fastq.trim", "fastq.filter"]),
    ("fastq.correct", &["fastq.trim"]),
];

pub const STAGE_BOUNDARY_INVARIANTS: [BoundaryInvariant; 4] = [
    BoundaryInvariant {
        from: "fastq.validate",
        to: "fastq.trim",
        rule: "validation does not modify reads; trim consumes validated reads",
    },
    BoundaryInvariant {
        from: "fastq.trim",
        to: "fastq.filter",
        rule: "trim output must remain FASTQ and preserve pairing",
    },
    BoundaryInvariant {
        from: "fastq.filter",
        to: "fastq.stats",
        rule: "filter output remains FASTQ; stats is report-only",
    },
    BoundaryInvariant {
        from: "fastq.merge",
        to: "fastq.stats",
        rule: "merge produces merged reads; stats accepts merged FASTQ",
    },
];

pub const STAGES: [StageDefinition; 10] = [
    StageDefinition {
        stage_id: "fastq.validate",
        kind: FastqStageKind::Core,
        semantics: StageSemantics {
            mutates_fastq: false,
            consumes_pairs: false,
            produces_reports_only: true,
            affects_metrics: &[MetricClass::Integrity],
        },
    },
    StageDefinition {
        stage_id: "fastq.trim",
        kind: FastqStageKind::Core,
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
        stage_id: "fastq.filter",
        kind: FastqStageKind::Core,
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
        stage_id: "fastq.stats",
        kind: FastqStageKind::Core,
        semantics: StageSemantics {
            mutates_fastq: false,
            consumes_pairs: false,
            produces_reports_only: true,
            affects_metrics: &[MetricClass::Integrity, MetricClass::Composition],
        },
    },
    StageDefinition {
        stage_id: "fastq.merge",
        kind: FastqStageKind::Core,
        semantics: StageSemantics {
            mutates_fastq: true,
            consumes_pairs: true,
            produces_reports_only: false,
            affects_metrics: &[MetricClass::Integrity, MetricClass::Retention],
        },
    },
    StageDefinition {
        stage_id: "fastq.correct",
        kind: FastqStageKind::Core,
        semantics: StageSemantics {
            mutates_fastq: true,
            consumes_pairs: true,
            produces_reports_only: false,
            affects_metrics: &[MetricClass::Integrity, MetricClass::QualityShift],
        },
    },
    StageDefinition {
        stage_id: "fastq.umi",
        kind: FastqStageKind::Optional,
        semantics: StageSemantics {
            mutates_fastq: true,
            consumes_pairs: true,
            produces_reports_only: false,
            affects_metrics: &[MetricClass::Integrity, MetricClass::Retention],
        },
    },
    StageDefinition {
        stage_id: "fastq.screen",
        kind: FastqStageKind::Optional,
        semantics: StageSemantics {
            mutates_fastq: false,
            consumes_pairs: false,
            produces_reports_only: true,
            affects_metrics: &[MetricClass::Contamination],
        },
    },
    StageDefinition {
        stage_id: "fastq.qc_post",
        kind: FastqStageKind::Optional,
        semantics: StageSemantics {
            mutates_fastq: false,
            consumes_pairs: false,
            produces_reports_only: true,
            affects_metrics: &[MetricClass::QualityShift, MetricClass::Contamination],
        },
    },
    StageDefinition {
        stage_id: "fastq.preprocess",
        kind: FastqStageKind::Meta,
        semantics: StageSemantics {
            mutates_fastq: false,
            consumes_pairs: true,
            produces_reports_only: false,
            affects_metrics: &[MetricClass::Integrity, MetricClass::Retention],
        },
    },
];

#[must_use]
pub fn stage_semantics(stage_id: &str) -> Option<StageSemantics> {
    STAGES
        .iter()
        .find(|stage| stage.stage_id == stage_id)
        .map(|stage| stage.semantics)
}

#[must_use]
pub fn stage_kind(stage_id: &str) -> Option<FastqStageKind> {
    STAGES
        .iter()
        .find(|stage| stage.stage_id == stage_id)
        .map(|stage| stage.kind)
}

#[must_use]
pub fn stage_metric_classes(stage_id: &str) -> Option<&'static [MetricClass]> {
    stage_semantics(stage_id).map(|semantics| semantics.affects_metrics)
}

#[must_use]
pub fn stage_metric_invariants(stage_id: &str) -> Option<&'static [&'static str]> {
    metric_spec_for_stage(stage_id).map(|spec| spec.invariants)
}
