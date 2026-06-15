//! FASTQ domain catalogs and profile invariant presets.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FastqInvariantsPreset {
    Default,
    Minimal,
    Adna,
    ReferenceAdna,
}

impl FastqInvariantsPreset {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Minimal => "minimal",
            Self::Adna => "adna",
            Self::ReferenceAdna => "reference_adna",
        }
    }
}

pub const FASTQ_STAGE_ID_CATALOG: &[&str] = &[
    "fastq.build_contaminant_db",
    "fastq.build_rrna_db",
    "fastq.build_taxonomy_db",
    "fastq.capture_provenance_snapshot",
    "fastq.classify_layout",
    "fastq.normalize_abundance",
    "fastq.infer_asvs",
    "fastq.remove_chimeras",
    "fastq.concatenate_lanes",
    "fastq.deplete_reference_contaminants",
    "fastq.deinterleave_reads",
    "fastq.demultiplex_reads",
    "fastq.trim_terminal_damage",
    "fastq.validate_reads",
    "fastq.detect_adapters",
    "fastq.detect_duplicates_premerge",
    "fastq.detect_instrument_artifacts",
    "fastq.trim_reads",
    "fastq.filter_reads",
    "fastq.deplete_host",
    "fastq.estimate_library_complexity_prealign",
    "fastq.profile_read_lengths",
    "fastq.filter_low_complexity",
    "fastq.interleave_reads",
    "fastq.materialize_qc_manifest",
    "fastq.merge_pairs",
    "fastq.cluster_otus",
    "fastq.normalize_read_names",
    "fastq.prepare_adapter_bank",
    "fastq.prepare_host_reference_bundle",
    "fastq.prepare_primer_bank",
    "fastq.profile_overrepresented_sequences",
    "fastq.trim_polyg_tails",
    "fastq.index_reference",
    "fastq.normalize_primers",
    "fastq.repair_pairs",
    "fastq.report_qc",
    "fastq.deplete_rrna",
    "fastq.profile_reads",
    "fastq.screen_taxonomy",
    "fastq.subsample_reads",
    "fastq.extract_umis",
    "fastq.correct_errors",
    "fastq.remove_duplicates",
    "fastq.verify_assets",
];

pub const FASTQ_LOCAL_BENCH_STAGE_ID_CATALOG: &[&str] = &[
    "fastq.index_reference",
    "fastq.validate_reads",
    "fastq.profile_read_lengths",
    "fastq.detect_adapters",
    "fastq.detect_duplicates_premerge",
    "fastq.estimate_library_complexity_prealign",
    "fastq.trim_terminal_damage",
    "fastq.normalize_primers",
    "fastq.trim_polyg_tails",
    "fastq.trim_reads",
    "fastq.filter_reads",
    "fastq.profile_reads",
    "fastq.deplete_rrna",
    "fastq.merge_pairs",
    "fastq.remove_duplicates",
    "fastq.filter_low_complexity",
    "fastq.deplete_host",
    "fastq.deplete_reference_contaminants",
    "fastq.correct_errors",
    "fastq.extract_umis",
    "fastq.profile_overrepresented_sequences",
    "fastq.report_qc",
    "fastq.remove_chimeras",
    "fastq.infer_asvs",
    "fastq.cluster_otus",
    "fastq.normalize_abundance",
    "fastq.screen_taxonomy",
];

pub const FASTQ_PARAMS_CATALOG: &[&str] = &[
    "bijux.fastq.params.validate.v1",
    "bijux.fastq.params.trim.v1",
    "bijux.fastq.params.filter.v1",
    "bijux.fastq.params.merge.v1",
    "bijux.fastq.params.stats.v1",
    "bijux.fastq.params.correct.v1",
    "bijux.fastq.params.umi.v1",
];

pub const FASTQ_METRICS_CATALOG: &[&str] = &[
    "bijux.fastq.validate.v1",
    "bijux.fastq.trim_reads.v1",
    "bijux.fastq.filter_reads.v1",
    "bijux.fastq.merge_pairs.v1",
    "bijux.fastq.report_qc.v1",
    "bijux.fastq.profile_reads.v1",
];
