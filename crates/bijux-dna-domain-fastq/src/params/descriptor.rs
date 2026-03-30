use crate::stages::ids::{
    STAGE_CLUSTER_OTUS, STAGE_CORRECT_ERRORS, STAGE_DEPLETE_HOST,
    STAGE_DEPLETE_REFERENCE_CONTAMINANTS, STAGE_DEPLETE_RRNA, STAGE_DETECT_ADAPTERS,
    STAGE_EXTRACT_UMIS, STAGE_FILTER_LOW_COMPLEXITY, STAGE_FILTER_READS, STAGE_INDEX_REFERENCE,
    STAGE_INFER_ASVS, STAGE_MERGE_PAIRS, STAGE_NORMALIZE_ABUNDANCE, STAGE_NORMALIZE_PRIMERS,
    STAGE_PROFILE_OVERREPRESENTED_SEQUENCES, STAGE_PROFILE_READS, STAGE_PROFILE_READ_LENGTHS,
    STAGE_REMOVE_CHIMERAS, STAGE_REMOVE_DUPLICATES, STAGE_REPORT_QC, STAGE_SCREEN_TAXONOMY,
    STAGE_TRIM_POLYG_TAILS, STAGE_TRIM_READS, STAGE_TRIM_TERMINAL_DAMAGE, STAGE_VALIDATE_READS,
};
use bijux_dna_core::ids::StageId;

use super::{edna, processing, quality};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StageParamDescriptor {
    pub param_type_id: &'static str,
    pub schema_version: &'static str,
}

const STAGE_PARAM_DESCRIPTORS: &[(&StageId, StageParamDescriptor)] = &[
    (
        &STAGE_PROFILE_READS,
        StageParamDescriptor {
            param_type_id: "fastq.profile_reads",
            schema_version: quality::stats::STATS_SCHEMA_VERSION,
        },
    ),
    (
        &STAGE_PROFILE_READ_LENGTHS,
        StageParamDescriptor {
            param_type_id: "fastq.profile_read_lengths",
            schema_version: quality::stats::READ_LENGTH_PROFILE_SCHEMA_VERSION,
        },
    ),
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
        &STAGE_DETECT_ADAPTERS,
        StageParamDescriptor {
            param_type_id: "fastq.detect_adapters",
            schema_version: quality::detect_adapters::DETECT_ADAPTERS_SCHEMA_VERSION,
        },
    ),
    (
        &STAGE_TRIM_READS,
        StageParamDescriptor {
            param_type_id: "fastq.trim_reads",
            schema_version: "bijux.fastq.params.trim_reads.v1",
        },
    ),
    (
        &STAGE_TRIM_TERMINAL_DAMAGE,
        StageParamDescriptor {
            param_type_id: "fastq.trim_terminal_damage",
            schema_version: quality::trim::TRIM_TERMINAL_DAMAGE_SCHEMA_VERSION,
        },
    ),
    (
        &STAGE_TRIM_POLYG_TAILS,
        StageParamDescriptor {
            param_type_id: "fastq.trim_polyg_tails",
            schema_version: quality::trim::TRIM_POLYG_TAILS_SCHEMA_VERSION,
        },
    ),
    (
        &STAGE_FILTER_READS,
        StageParamDescriptor {
            param_type_id: "fastq.filter_reads",
            schema_version: "bijux.fastq.params.filter_reads.v1",
        },
    ),
    (
        &STAGE_FILTER_LOW_COMPLEXITY,
        StageParamDescriptor {
            param_type_id: "fastq.filter_low_complexity",
            schema_version: "bijux.fastq.params.filter_low_complexity.v1",
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
        &STAGE_DEPLETE_HOST,
        StageParamDescriptor {
            param_type_id: "fastq.deplete_host",
            schema_version: quality::screen::HOST_DEPLETION_SCHEMA_VERSION,
        },
    ),
    (
        &STAGE_DEPLETE_REFERENCE_CONTAMINANTS,
        StageParamDescriptor {
            param_type_id: "fastq.deplete_reference_contaminants",
            schema_version: quality::screen::REFERENCE_DEPLETION_SCHEMA_VERSION,
        },
    ),
    (
        &STAGE_DEPLETE_RRNA,
        StageParamDescriptor {
            param_type_id: "fastq.deplete_rrna",
            schema_version: "bijux.fastq.params.deplete_rrna.v1",
        },
    ),
    (
        &STAGE_SCREEN_TAXONOMY,
        StageParamDescriptor {
            param_type_id: "fastq.screen_taxonomy",
            schema_version: "bijux.fastq.params.screen_taxonomy.v1",
        },
    ),
    (
        &STAGE_PROFILE_OVERREPRESENTED_SEQUENCES,
        StageParamDescriptor {
            param_type_id: "fastq.profile_overrepresented_sequences",
            schema_version: quality::stats::OVERREPRESENTED_PROFILE_SCHEMA_VERSION,
        },
    ),
    (
        &STAGE_VALIDATE_READS,
        StageParamDescriptor {
            param_type_id: "fastq.validate_reads",
            schema_version: quality::validate::VALIDATE_SCHEMA_VERSION,
        },
    ),
    (
        &STAGE_REMOVE_DUPLICATES,
        StageParamDescriptor {
            param_type_id: "fastq.remove_duplicates",
            schema_version: processing::remove_duplicates::REMOVE_DUPLICATES_SCHEMA_VERSION,
        },
    ),
    (
        &STAGE_REPORT_QC,
        StageParamDescriptor {
            param_type_id: "fastq.report_qc",
            schema_version: quality::qc_post::REPORT_QC_SCHEMA_VERSION,
        },
    ),
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

#[must_use]
pub fn stage_param_descriptor(stage_id: &StageId) -> Option<StageParamDescriptor> {
    STAGE_PARAM_DESCRIPTORS
        .iter()
        .find_map(|(candidate_stage_id, descriptor)| {
            if stage_id == *candidate_stage_id {
                Some(*descriptor)
            } else {
                None
            }
        })
}
