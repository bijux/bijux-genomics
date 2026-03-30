use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_domain_fastq::{
    is_observer_specialized_stage_tool as domain_is_observer_specialized_stage_tool,
    observer_specialized_stage_tool_bindings as domain_observer_specialized_stage_tool_bindings,
};

pub mod artifacts;
pub mod commands;

pub use artifacts::*;
pub use bijux_dna_domain_fastq::observer::{
    parse_bbduk_reads_removed, parse_cluster_otus_report, parse_correct_errors_report,
    parse_deduplicate_report, parse_deplete_host_report,
    parse_deplete_reference_contaminants_report, parse_deplete_rrna_report,
    parse_detect_adapters_report, parse_duplicate_classes_tsv, parse_extract_umis_report,
    parse_fastp_metrics, parse_fastqvalidator_count, parse_filter_low_complexity_report,
    parse_filter_reads_report, parse_index_reference_report, parse_infer_asvs_report,
    parse_length_histogram, parse_low_complexity_report, parse_merge_pairs_report,
    parse_multiqc_general_stats_metrics, parse_normalize_abundance_report,
    parse_normalize_primers_report, parse_profile_overrepresented_report,
    parse_profile_read_lengths_report, parse_profile_reads_report, parse_remove_chimeras_report,
    parse_remove_duplicates_provenance, parse_remove_duplicates_report, parse_report_qc_report,
    parse_screen_summary_tsv, parse_screen_taxonomy_report, parse_seqkit_stats,
    parse_terminal_damage_report, parse_trim_polyg_report, parse_trim_reads_report,
    parse_validated_reads_manifest, parse_validation_report,
};
pub use commands::*;

#[must_use]
pub fn observer_specialized_stage_tool_bindings() -> Vec<(StageId, ToolId)> {
    domain_observer_specialized_stage_tool_bindings()
}

#[must_use]
pub fn is_observer_specialized_stage_tool(stage_id: &StageId, tool_id: &ToolId) -> bool {
    domain_is_observer_specialized_stage_tool(stage_id, tool_id)
}
