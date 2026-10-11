use super::models::FastqReportPlacement;
use bijux_dna_core::id_catalog::{
    FASTQ_CLUSTER_OTUS, FASTQ_CORRECT, FASTQ_DEDUPLICATE, FASTQ_DEPLETE_HOST,
    FASTQ_DEPLETE_REFERENCE_CONTAMINANTS, FASTQ_DEPLETE_RRNA, FASTQ_DETECT_ADAPTERS,
    FASTQ_DETECT_DUPLICATES_PREMERGE, FASTQ_ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN, FASTQ_FILTER,
    FASTQ_INDEX_REFERENCE, FASTQ_INFER_ASVS, FASTQ_LOW_COMPLEXITY, FASTQ_MERGE,
    FASTQ_NORMALIZE_ABUNDANCE, FASTQ_NORMALIZE_PRIMERS, FASTQ_PROFILE_OVERREPRESENTED_SEQUENCES,
    FASTQ_PROFILE_READ_LENGTHS, FASTQ_QC_POST, FASTQ_REMOVE_CHIMERAS, FASTQ_SCREEN,
    FASTQ_STATS_NEUTRAL, FASTQ_TRIM, FASTQ_TRIM_POLYG_TAILS, FASTQ_TRIM_TERMINAL_DAMAGE, FASTQ_UMI,
    FASTQ_VALIDATE_READS,
};
pub(super) fn placement_for_stage(stage_id: &str) -> Option<FastqReportPlacement> {
    match stage_id {
        FASTQ_INDEX_REFERENCE => Some(FastqReportPlacement {
            section_id: "reference_preparation",
            section_title: "Reference Preparation",
            summary_table_id: "reference_index_assets",
            summary_table_title: "Reference Index Assets",
            report_focus: "reference provenance and benchmark index availability",
        }),
        FASTQ_VALIDATE_READS => Some(FastqReportPlacement {
            section_id: "input_readiness",
            section_title: "Input Readiness",
            summary_table_id: "validation_intake",
            summary_table_title: "Validation and Intake",
            report_focus: "input validation, manifest integrity, and governed read admission",
        }),
        FASTQ_PROFILE_READ_LENGTHS
        | FASTQ_DETECT_ADAPTERS
        | FASTQ_STATS_NEUTRAL
        | FASTQ_PROFILE_OVERREPRESENTED_SEQUENCES
        | FASTQ_QC_POST => Some(FastqReportPlacement {
            section_id: "quality_profiling",
            section_title: "Quality Profiling",
            summary_table_id: "qc_signal_profiles",
            summary_table_title: "QC Signal Profiles",
            report_focus: "baseline read quality, composition, and aggregated QC evidence",
        }),
        FASTQ_DETECT_DUPLICATES_PREMERGE | FASTQ_ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN => {
            Some(FastqReportPlacement {
                section_id: "quality_profiling",
                section_title: "Quality Profiling",
                summary_table_id: "premerge_complexity",
                summary_table_title: "Pre-merge Complexity",
                report_focus: "duplicate burden and pre-alignment library complexity evidence",
            })
        }
        FASTQ_TRIM_TERMINAL_DAMAGE
        | FASTQ_TRIM_POLYG_TAILS
        | FASTQ_TRIM
        | FASTQ_FILTER
        | FASTQ_MERGE
        | FASTQ_DEDUPLICATE
        | FASTQ_LOW_COMPLEXITY
        | FASTQ_CORRECT
        | FASTQ_UMI => Some(FastqReportPlacement {
            section_id: "read_cleanup",
            section_title: "Read Cleanup",
            summary_table_id: "cleanup_retention",
            summary_table_title: "Cleanup and Retention",
            report_focus: "sequence-retention deltas, cleanup transforms, and mutation provenance",
        }),
        FASTQ_DEPLETE_RRNA
        | FASTQ_DEPLETE_HOST
        | FASTQ_DEPLETE_REFERENCE_CONTAMINANTS
        | FASTQ_SCREEN => Some(FastqReportPlacement {
            section_id: "contamination_screening",
            section_title: "Contamination Screening",
            summary_table_id: "screening_contamination",
            summary_table_title: "Screening and Contamination",
            report_focus:
                "reference-guided depletion, contamination screening, and taxonomy evidence",
        }),
        FASTQ_NORMALIZE_PRIMERS
        | FASTQ_REMOVE_CHIMERAS
        | FASTQ_INFER_ASVS
        | FASTQ_CLUSTER_OTUS
        | FASTQ_NORMALIZE_ABUNDANCE => Some(FastqReportPlacement {
            section_id: "amplicon_interpretation",
            section_title: "Amplicon Interpretation",
            summary_table_id: "amplicon_features",
            summary_table_title: "Amplicon Feature Tables",
            report_focus:
                "amplicon cleanup, feature inference, and abundance normalization outputs",
        }),
        _ => None,
    }
}
