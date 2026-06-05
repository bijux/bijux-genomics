//! Owner: bijux-dna-domain-fastq
//! Observer parsers for tool stdout and report artifacts.
//! Supported formats must be deterministic and stable across versions.

use crate::metrics::{
    AdapterRemovalToolMetricsV1, FastpToolMetricsV1, FastqcToolMetricsV1, MultiqcToolMetricsV1,
    SamtoolsFlagstatMetricsV1, SeqkitToolMetricsV1,
};
use crate::params::{
    detect_adapters::{AdapterEvidenceFormat, AdapterEvidenceScope, AdapterInspectionMode},
    PairedMode,
};
use crate::DuplicateClassEntryV1;
use crate::{
    ClusterOtusReportV1, CorrectErrorsReportV1, DepleteHostReportV1,
    DepleteReferenceContaminantsReportV1, DepleteRrnaReportV1, DetectAdaptersReportV1,
    DetectDuplicatesPremergeReportV1, ExtractUmisReportV1, FilterLowComplexityReportV1,
    FilterReadsReportV1, IndexReferenceReportV1, InferAsvsReportV1, MergePairsReportV1,
    NormalizeAbundanceReportV1, NormalizePrimersReportV1, OverrepresentedSequenceRowV1,
    ProfileOverrepresentedReportV1, ProfileReadLengthBinV1, ProfileReadLengthsReportV1,
    ProfileReadsHistogramBinV1, ProfileReadsMateSummaryV1, ProfileReadsReportV1,
    RemoveChimerasReportV1, RemoveDuplicatesProvenanceV1, RemoveDuplicatesReportV1,
    ReportQcReportV1, ScreenTaxonomyReportV1, TaxonomyScreenSummaryEntryV1, TerminalDamageReportV1,
    TrimPolygReportV1, TrimReadsReportV1, ValidatedReadsManifestV1, ValidationReportV1,
};
use bijux_dna_core::prelude::measure::SeqkitMetrics;

mod adapter_taxonomy;
mod correct_errors;
mod depletion;
mod duplicates;
mod filtering;
#[cfg(test)]
mod parser_contracts;
mod profiles;
mod reports;
mod raw_parser_contract;
mod sequence;
pub(super) mod tool_metrics;

pub use self::sequence::{parse_fastqvalidator_count, parse_length_histogram, parse_seqkit_stats};

pub use self::reports::{
    parse_cluster_otus_report, parse_detect_duplicates_premerge_report,
    parse_estimate_library_complexity_prealign_report, parse_filter_reads_report,
    parse_index_reference_report, parse_infer_asvs_report, parse_merge_pairs_report,
    parse_normalize_abundance_report, parse_normalize_primers_report, parse_report_qc_report,
    parse_screen_taxonomy_report, parse_terminal_damage_report, parse_trim_polyg_report,
    parse_trim_reads_report, parse_validated_reads_manifest, parse_validation_report,
};
pub use self::raw_parser_contract::{
    evaluate_fastq_raw_parser_failure_contracts, FastqRawParserFailureClass,
    FastqRawParserFailureContractRow,
};

pub use self::correct_errors::parse_correct_errors_report;

pub use self::profiles::{
    parse_profile_overrepresented_report, parse_profile_read_lengths_report,
    parse_profile_reads_report,
};

pub use self::depletion::{
    parse_deplete_host_report, parse_deplete_reference_contaminants_report,
    parse_deplete_rrna_report,
};

pub use self::adapter_taxonomy::{parse_detect_adapters_report, parse_screen_summary_tsv};

pub use self::duplicates::{
    parse_deduplicate_report, parse_duplicate_classes_tsv, parse_remove_chimeras_report,
    parse_remove_duplicates_provenance, parse_remove_duplicates_report,
};

pub use self::filtering::{
    parse_bbduk_reads_removed, parse_extract_umis_report, parse_filter_low_complexity_report,
    parse_low_complexity_report,
};

use self::tool_metrics::{parse_report_u64_field, u64_to_f64};
