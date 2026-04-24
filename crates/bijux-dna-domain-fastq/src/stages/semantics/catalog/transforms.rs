use crate::metrics::spec::MetricClass;
use crate::pipeline_contract::StageCriticality;

use super::{stage_definition, FastqStageKind, StageDefinition};

pub const MERGE_PAIRS: StageDefinition = stage_definition(
    "fastq.merge_pairs",
    FastqStageKind::Core,
    StageCriticality::Optional,
    true,
    true,
    false,
    &[MetricClass::Integrity, MetricClass::Retention],
);

pub const REMOVE_DUPLICATES: StageDefinition = stage_definition(
    "fastq.remove_duplicates",
    FastqStageKind::Optional,
    StageCriticality::Optional,
    true,
    true,
    false,
    &[MetricClass::Integrity, MetricClass::Retention, MetricClass::QualityShift],
);

pub const FILTER_LOW_COMPLEXITY: StageDefinition = stage_definition(
    "fastq.filter_low_complexity",
    FastqStageKind::Optional,
    StageCriticality::Optional,
    true,
    true,
    false,
    &[MetricClass::Integrity, MetricClass::Retention, MetricClass::QualityShift],
);

pub const CORRECT_ERRORS: StageDefinition = stage_definition(
    "fastq.correct_errors",
    FastqStageKind::Core,
    StageCriticality::Optional,
    true,
    true,
    false,
    &[MetricClass::Integrity, MetricClass::QualityShift],
);

pub const EXTRACT_UMIS: StageDefinition = stage_definition(
    "fastq.extract_umis",
    FastqStageKind::Optional,
    StageCriticality::Optional,
    true,
    true,
    false,
    &[MetricClass::Integrity, MetricClass::Retention],
);

pub const STAGES: [StageDefinition; 5] =
    [MERGE_PAIRS, REMOVE_DUPLICATES, FILTER_LOW_COMPLEXITY, CORRECT_ERRORS, EXTRACT_UMIS];
