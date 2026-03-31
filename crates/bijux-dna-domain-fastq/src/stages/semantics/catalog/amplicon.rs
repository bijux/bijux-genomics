use super::{stage_definition, FastqStageKind, StageDefinition};
use crate::metrics::spec::MetricClass;
use crate::pipeline_contract::StageCriticality;

pub const NORMALIZE_PRIMERS: StageDefinition = stage_definition(
    "fastq.normalize_primers",
    FastqStageKind::Amplicon,
    StageCriticality::Essential,
    true,
    true,
    false,
    &[MetricClass::Integrity, MetricClass::Retention],
);

pub const REMOVE_CHIMERAS: StageDefinition = stage_definition(
    "fastq.remove_chimeras",
    FastqStageKind::Amplicon,
    StageCriticality::Essential,
    true,
    false,
    false,
    &[MetricClass::Integrity, MetricClass::Retention],
);

pub const INFER_ASVS: StageDefinition = stage_definition(
    "fastq.infer_asvs",
    FastqStageKind::Amplicon,
    StageCriticality::Experimental,
    false,
    false,
    false,
    &[MetricClass::Composition],
);

pub const CLUSTER_OTUS: StageDefinition = stage_definition(
    "fastq.cluster_otus",
    FastqStageKind::Amplicon,
    StageCriticality::Essential,
    false,
    false,
    false,
    &[MetricClass::Composition],
);

pub const NORMALIZE_ABUNDANCE: StageDefinition = stage_definition(
    "fastq.normalize_abundance",
    FastqStageKind::Amplicon,
    StageCriticality::Essential,
    false,
    false,
    false,
    &[MetricClass::Composition],
);

pub const STAGES: [StageDefinition; 5] = [
    NORMALIZE_PRIMERS,
    REMOVE_CHIMERAS,
    INFER_ASVS,
    CLUSTER_OTUS,
    NORMALIZE_ABUNDANCE,
];
