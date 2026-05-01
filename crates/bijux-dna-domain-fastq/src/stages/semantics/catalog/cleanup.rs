use crate::metrics::spec::MetricClass;
use crate::pipeline_contract::StageCriticality;

use super::{stage_definition, FastqStageKind, StageDefinition};

pub const VALIDATE_READS: StageDefinition = stage_definition(
    "fastq.validate_reads",
    FastqStageKind::Core,
    StageCriticality::Essential,
    false,
    false,
    true,
    &[MetricClass::Integrity],
);

pub const PROFILE_READ_LENGTHS: StageDefinition = stage_definition(
    "fastq.profile_read_lengths",
    FastqStageKind::Optional,
    StageCriticality::Essential,
    false,
    false,
    true,
    &[MetricClass::Integrity, MetricClass::Composition],
);

pub const DETECT_ADAPTERS: StageDefinition = stage_definition(
    "fastq.detect_adapters",
    FastqStageKind::Core,
    StageCriticality::Essential,
    false,
    false,
    true,
    &[MetricClass::Composition],
);

pub const DETECT_DUPLICATES_PREMERGE: StageDefinition = stage_definition(
    "fastq.detect_duplicates_premerge",
    FastqStageKind::Optional,
    StageCriticality::Optional,
    false,
    false,
    true,
    &[MetricClass::Integrity],
);

pub const ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN: StageDefinition = stage_definition(
    "fastq.estimate_library_complexity_prealign",
    FastqStageKind::Optional,
    StageCriticality::Optional,
    false,
    false,
    true,
    &[MetricClass::Integrity],
);

pub const TRIM_TERMINAL_DAMAGE: StageDefinition = stage_definition(
    "fastq.trim_terminal_damage",
    FastqStageKind::Core,
    StageCriticality::Essential,
    true,
    true,
    false,
    &[MetricClass::Integrity, MetricClass::Retention],
);

pub const TRIM_POLYG_TAILS: StageDefinition = stage_definition(
    "fastq.trim_polyg_tails",
    FastqStageKind::Optional,
    StageCriticality::Essential,
    true,
    true,
    false,
    &[MetricClass::Integrity, MetricClass::Retention],
);

pub const TRIM_READS: StageDefinition = stage_definition(
    "fastq.trim_reads",
    FastqStageKind::Core,
    StageCriticality::Essential,
    true,
    true,
    false,
    &[MetricClass::Integrity, MetricClass::Retention, MetricClass::QualityShift],
);

pub const FILTER_READS: StageDefinition = stage_definition(
    "fastq.filter_reads",
    FastqStageKind::Core,
    StageCriticality::Essential,
    true,
    true,
    false,
    &[MetricClass::Integrity, MetricClass::Retention, MetricClass::QualityShift],
);

pub const PROFILE_READS: StageDefinition = stage_definition(
    "fastq.profile_reads",
    FastqStageKind::Core,
    StageCriticality::Essential,
    false,
    false,
    true,
    &[MetricClass::Integrity, MetricClass::Composition],
);

pub const PROFILE_OVERREPRESENTED_SEQUENCES: StageDefinition = stage_definition(
    "fastq.profile_overrepresented_sequences",
    FastqStageKind::Optional,
    StageCriticality::Essential,
    false,
    false,
    true,
    &[MetricClass::Composition],
);

pub const REPORT_QC: StageDefinition = stage_definition(
    "fastq.report_qc",
    FastqStageKind::Optional,
    StageCriticality::Essential,
    false,
    false,
    true,
    &[MetricClass::QualityShift, MetricClass::Contamination],
);

pub const STAGES: [StageDefinition; 12] = [
    VALIDATE_READS,
    PROFILE_READ_LENGTHS,
    DETECT_ADAPTERS,
    DETECT_DUPLICATES_PREMERGE,
    ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN,
    TRIM_TERMINAL_DAMAGE,
    TRIM_POLYG_TAILS,
    TRIM_READS,
    FILTER_READS,
    PROFILE_READS,
    PROFILE_OVERREPRESENTED_SEQUENCES,
    REPORT_QC,
];
