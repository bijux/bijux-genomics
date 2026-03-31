use crate::stages::ids::{
    STAGE_CORRECT_ERRORS, STAGE_EXTRACT_UMIS, STAGE_INDEX_REFERENCE, STAGE_MERGE_PAIRS,
    STAGE_REMOVE_DUPLICATES,
};
use bijux_dna_core::ids::StageId;

use super::StageParamDescriptor;
use crate::params::processing;

pub(super) const STAGE_PARAM_DESCRIPTORS: &[(&StageId, StageParamDescriptor)] = &[
    (
        &STAGE_CORRECT_ERRORS,
        StageParamDescriptor {
            param_type_id: "fastq.correct_errors",
            schema_version: processing::correct::CORRECT_SCHEMA_VERSION,
        },
    ),
    (
        &STAGE_EXTRACT_UMIS,
        StageParamDescriptor {
            param_type_id: "fastq.extract_umis",
            schema_version: processing::umi::UMI_SCHEMA_VERSION,
        },
    ),
    (
        &STAGE_MERGE_PAIRS,
        StageParamDescriptor {
            param_type_id: "fastq.merge_pairs",
            schema_version: processing::merge::MERGE_SCHEMA_VERSION,
        },
    ),
    (
        &STAGE_INDEX_REFERENCE,
        StageParamDescriptor {
            param_type_id: "fastq.index_reference",
            schema_version: processing::reference_index::INDEX_REFERENCE_SCHEMA_VERSION,
        },
    ),
    (
        &STAGE_REMOVE_DUPLICATES,
        StageParamDescriptor {
            param_type_id: "fastq.remove_duplicates",
            schema_version: processing::remove_duplicates::REMOVE_DUPLICATES_SCHEMA_VERSION,
        },
    ),
];
