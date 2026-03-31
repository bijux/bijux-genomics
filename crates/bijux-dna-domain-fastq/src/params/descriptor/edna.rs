use crate::stages::ids::{
    STAGE_CLUSTER_OTUS, STAGE_INFER_ASVS, STAGE_NORMALIZE_ABUNDANCE, STAGE_NORMALIZE_PRIMERS,
    STAGE_REMOVE_CHIMERAS,
};
use bijux_dna_core::ids::StageId;

use super::StageParamDescriptor;
use crate::params::edna;

pub(super) const STAGE_PARAM_DESCRIPTORS: &[(&StageId, StageParamDescriptor)] = &[
    (
        &STAGE_NORMALIZE_PRIMERS,
        StageParamDescriptor {
            param_type_id: "fastq.normalize_primers",
            schema_version: edna::EDNA_SCHEMA_VERSION,
        },
    ),
    (
        &STAGE_REMOVE_CHIMERAS,
        StageParamDescriptor {
            param_type_id: "fastq.remove_chimeras",
            schema_version: edna::EDNA_SCHEMA_VERSION,
        },
    ),
    (
        &STAGE_INFER_ASVS,
        StageParamDescriptor {
            param_type_id: "fastq.infer_asvs",
            schema_version: edna::EDNA_SCHEMA_VERSION,
        },
    ),
    (
        &STAGE_CLUSTER_OTUS,
        StageParamDescriptor {
            param_type_id: "fastq.cluster_otus",
            schema_version: edna::EDNA_SCHEMA_VERSION,
        },
    ),
    (
        &STAGE_NORMALIZE_ABUNDANCE,
        StageParamDescriptor {
            param_type_id: "fastq.normalize_abundance",
            schema_version: edna::EDNA_SCHEMA_VERSION,
        },
    ),
];
