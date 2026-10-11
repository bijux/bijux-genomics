use super::{FamilyProbeSpec, ProbeKind};
use crate::reports::ParseKind;

pub(super) const FASTQ: &[FamilyProbeSpec] = &[
    FamilyProbeSpec {
        family_id: "reference_indexing",
        representative_stage_id: "fastq.index_reference",
        probe_kind: ProbeKind::GeneratedIndexReference,
        parse_kind: ParseKind::FastqIndexReference,
        snapshot_keys: &["tool_id", "threads", "index_format", "index_file_count", "index_bytes"],
    },
    FamilyProbeSpec {
        family_id: "adapter_and_quality_trimming",
        representative_stage_id: "fastq.trim_reads",
        probe_kind: ProbeKind::LocalStage {
            stage_id: "fastq.trim_reads",
            proof_pointer: Some("/cases/0/report_json"),
        },
        parse_kind: ParseKind::FastqTrimReads,
        snapshot_keys: &["tool_id", "reads_in", "reads_out", "bases_removed"],
    },
    FamilyProbeSpec {
        family_id: "read_pair_merging",
        representative_stage_id: "fastq.merge_pairs",
        probe_kind: ProbeKind::LocalStage {
            stage_id: "fastq.merge_pairs",
            proof_pointer: Some("/case_report_json"),
        },
        parse_kind: ParseKind::FastqMergePairs,
        snapshot_keys: &["tool_id", "reads_merged", "reads_unmerged", "merge_rate"],
    },
    FamilyProbeSpec {
        family_id: "read_filtering_and_low_complexity",
        representative_stage_id: "fastq.filter_reads",
        probe_kind: ProbeKind::LocalStage {
            stage_id: "fastq.filter_reads",
            proof_pointer: Some("/report_json"),
        },
        parse_kind: ParseKind::FastqFilterReads,
        snapshot_keys: &["tool_id", "reads_in", "reads_out", "reads_removed_by_kmer"],
    },
    FamilyProbeSpec {
        family_id: "read_validation_and_profiling",
        representative_stage_id: "fastq.profile_reads",
        probe_kind: ProbeKind::LocalStage {
            stage_id: "fastq.profile_reads",
            proof_pointer: Some("/cases/0/report_json"),
        },
        parse_kind: ParseKind::FastqProfileReads,
        snapshot_keys: &["tool_id", "reads_total", "bases_total", "gc_percent"],
    },
    FamilyProbeSpec {
        family_id: "report_aggregation",
        representative_stage_id: "fastq.report_qc",
        probe_kind: ProbeKind::LocalStage { stage_id: "fastq.report_qc", proof_pointer: None },
        parse_kind: ParseKind::FastqReportQc,
        snapshot_keys: &[
            "tool_id",
            "governed_qc_input_count",
            "governed_qc_lineage_hash",
            "aggregation_scope",
        ],
    },
    FamilyProbeSpec {
        family_id: "error_correction",
        representative_stage_id: "fastq.correct_errors",
        probe_kind: ProbeKind::GeneratedCorrectErrors,
        parse_kind: ParseKind::FastqCorrectErrors,
        snapshot_keys: &["tool_id", "threads", "reads_in", "reads_out", "changed_reads"],
    },
    FamilyProbeSpec {
        family_id: "duplicate_handling",
        representative_stage_id: "fastq.remove_duplicates",
        probe_kind: ProbeKind::LocalStage {
            stage_id: "fastq.remove_duplicates",
            proof_pointer: Some("/case_report_json"),
        },
        parse_kind: ParseKind::FastqRemoveDuplicates,
        snapshot_keys: &["tool_id", "reads_in", "reads_out", "duplicate_reads"],
    },
    FamilyProbeSpec {
        family_id: "umi_processing",
        representative_stage_id: "fastq.extract_umis",
        probe_kind: ProbeKind::LocalStage {
            stage_id: "fastq.extract_umis",
            proof_pointer: Some("/case_report_json"),
        },
        parse_kind: ParseKind::FastqExtractUmis,
        snapshot_keys: &["tool_id", "reads_in", "reads_out", "extracted_umis"],
    },
    FamilyProbeSpec {
        family_id: "taxonomy_classification",
        representative_stage_id: "fastq.screen_taxonomy",
        probe_kind: ProbeKind::GeneratedScreenTaxonomy,
        parse_kind: ParseKind::FastqScreenTaxonomy,
        snapshot_keys: &["tool_id", "reads_in", "classified_fraction", "contamination_rate"],
    },
    FamilyProbeSpec {
        family_id: "rrna_depletion",
        representative_stage_id: "fastq.deplete_rrna",
        probe_kind: ProbeKind::GeneratedDepleteRrna,
        parse_kind: ParseKind::FastqDepleteRrna,
        snapshot_keys: &["tool_id", "reads_in", "reads_removed", "rrna_fraction_removed"],
    },
    FamilyProbeSpec {
        family_id: "amplicon_processing",
        representative_stage_id: "fastq.infer_asvs",
        probe_kind: ProbeKind::LocalStage {
            stage_id: "fastq.infer_asvs",
            proof_pointer: Some("/case_report_json"),
        },
        parse_kind: ParseKind::FastqInferAsvs,
        snapshot_keys: &["tool_id", "feature_count", "sample_count", "reads_in"],
    },
    FamilyProbeSpec {
        family_id: "abundance_normalization",
        representative_stage_id: "fastq.normalize_abundance",
        probe_kind: ProbeKind::LocalStage {
            stage_id: "fastq.normalize_abundance",
            proof_pointer: Some("/case_report_json"),
        },
        parse_kind: ParseKind::FastqNormalizeAbundance,
        snapshot_keys: &["tool_id", "row_count", "sample_count", "normalization_method"],
    },
    FamilyProbeSpec {
        family_id: "internal_workflow_runtime",
        representative_stage_id: "fastq.detect_duplicates_premerge",
        probe_kind: ProbeKind::LocalStage {
            stage_id: "fastq.detect_duplicates_premerge",
            proof_pointer: Some("/cases/0/report_json"),
        },
        parse_kind: ParseKind::FastqDetectDuplicatesPremerge,
        snapshot_keys: &["tool_id", "reads_in", "duplicate_reads", "duplicate_fraction"],
    },
];
