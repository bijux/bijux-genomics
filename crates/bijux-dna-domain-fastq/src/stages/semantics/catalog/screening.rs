use crate::metrics::spec::MetricClass;
use crate::pipeline_contract::StageCriticality;

use super::{stage_definition, FastqStageKind, StageDefinition};

pub const DEPLETE_RRNA: StageDefinition = stage_definition(
    "fastq.deplete_rrna",
    FastqStageKind::Optional,
    StageCriticality::Optional,
    true,
    true,
    false,
    &[MetricClass::Contamination, MetricClass::Retention],
);

pub const DEPLETE_HOST: StageDefinition = stage_definition(
    "fastq.deplete_host",
    FastqStageKind::Optional,
    StageCriticality::Optional,
    true,
    true,
    false,
    &[MetricClass::Contamination, MetricClass::Retention],
);

pub const DEPLETE_REFERENCE_CONTAMINANTS: StageDefinition = stage_definition(
    "fastq.deplete_reference_contaminants",
    FastqStageKind::Optional,
    StageCriticality::Optional,
    true,
    true,
    false,
    &[MetricClass::Contamination, MetricClass::Retention],
);

pub const SCREEN_TAXONOMY: StageDefinition = stage_definition(
    "fastq.screen_taxonomy",
    FastqStageKind::Optional,
    StageCriticality::Optional,
    false,
    false,
    true,
    &[MetricClass::Contamination],
);

pub const INDEX_REFERENCE: StageDefinition = stage_definition(
    "fastq.index_reference",
    FastqStageKind::Meta,
    StageCriticality::Optional,
    false,
    false,
    false,
    &[],
);

pub const STAGES: [StageDefinition; 5] =
    [DEPLETE_RRNA, DEPLETE_HOST, DEPLETE_REFERENCE_CONTAMINANTS, SCREEN_TAXONOMY, INDEX_REFERENCE];
