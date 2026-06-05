use super::tool_metrics::{
    parse_adapterremoval_metrics, parse_fastp_metrics, parse_fastqc_summary_metrics,
    parse_multiqc_general_stats_metrics, parse_samtools_flagstat_metrics,
    parse_seqkit_tool_metrics,
};
use super::{
    parse_bbduk_reads_removed, parse_correct_errors_report, parse_deduplicate_report,
    parse_deplete_host_report, parse_deplete_reference_contaminants_report,
    parse_deplete_rrna_report, parse_detect_adapters_report,
    parse_detect_duplicates_premerge_report, parse_duplicate_classes_tsv,
    parse_estimate_library_complexity_prealign_report, parse_extract_umis_report,
    parse_fastqvalidator_count, parse_filter_low_complexity_report, parse_filter_reads_report,
    parse_index_reference_report, parse_infer_asvs_report, parse_length_histogram,
    parse_low_complexity_report, parse_profile_overrepresented_report,
    parse_profile_read_lengths_report, parse_profile_reads_report, parse_remove_chimeras_report,
    parse_remove_duplicates_provenance, parse_remove_duplicates_report, parse_report_qc_report,
    parse_screen_summary_tsv, parse_screen_taxonomy_report, parse_seqkit_stats,
    parse_terminal_damage_report, parse_trim_reads_report, parse_validated_reads_manifest,
    parse_validation_report,
};
use crate::params::trim::TerminalDamageExecutionPolicy;
use crate::params::DamageMode;
use crate::{
    PairedMode, ValidateFailureClass, DEPLETE_RRNA_REPORT_SCHEMA_VERSION,
    REPORT_QC_REPORT_SCHEMA_VERSION, SCREEN_TAXONOMY_REPORT_SCHEMA_VERSION,
    TERMINAL_DAMAGE_REPORT_SCHEMA_VERSION, VALIDATED_READS_MANIFEST_SCHEMA_VERSION,
    VALIDATION_REPORT_SCHEMA_VERSION,
};
use anyhow::Result;
use bijux_dna_core::id_catalog;

mod backend_metric_fixtures;
mod processing_reports;
mod profile_reports;
mod raw_fixture_bank;
mod read_cleanup_reports;
mod screening_reports;
mod validation_reports;

fn assert_f64_eq(left: f64, right: f64) {
    assert!((left - right).abs() < f64::EPSILON);
}
