use crate::stages::ids::{
    STAGE_DEPLETE_HOST, STAGE_DEPLETE_REFERENCE_CONTAMINANTS, STAGE_DEPLETE_RRNA,
    STAGE_DETECT_ADAPTERS, STAGE_ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN, STAGE_FILTER_LOW_COMPLEXITY,
    STAGE_FILTER_READS, STAGE_PROFILE_OVERREPRESENTED_SEQUENCES, STAGE_PROFILE_READS,
    STAGE_PROFILE_READ_LENGTHS, STAGE_REPORT_QC, STAGE_SCREEN_TAXONOMY, STAGE_TRIM_POLYG_TAILS,
    STAGE_TRIM_READS, STAGE_TRIM_TERMINAL_DAMAGE, STAGE_VALIDATE_READS,
};
use bijux_dna_core::ids::StageId;

use super::StageParamDescriptor;
use crate::params::quality;

pub(super) const STAGE_PARAM_DESCRIPTORS: &[(&StageId, StageParamDescriptor)] = &[
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
        &STAGE_DETECT_ADAPTERS,
        StageParamDescriptor {
            param_type_id: "fastq.detect_adapters",
            schema_version: quality::detect_adapters::DETECT_ADAPTERS_SCHEMA_VERSION,
        },
    ),
    (
        &STAGE_ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN,
        StageParamDescriptor {
            param_type_id: "fastq.estimate_library_complexity_prealign",
            schema_version: quality::stats::ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN_SCHEMA_VERSION,
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
            schema_version: quality::filter_low_complexity::FILTER_LOW_COMPLEXITY_SCHEMA_VERSION,
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
        &STAGE_REPORT_QC,
        StageParamDescriptor {
            param_type_id: "fastq.report_qc",
            schema_version: quality::qc_post::REPORT_QC_SCHEMA_VERSION,
        },
    ),
];
