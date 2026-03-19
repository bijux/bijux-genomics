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
    "fastq.abundance_normalization",
    "fastq.asv_inference",
    "fastq.chimera_detection",
    "fastq.deplete_reference_contaminants",
    "fastq.trim_terminal_damage",
    "fastq.validate_reads",
    "fastq.detect_adapters",
    "fastq.trim_reads",
    "fastq.filter_reads",
    "fastq.deplete_host",
    "fastq.profile_read_lengths",
    "fastq.filter_low_complexity",
    "fastq.merge_pairs",
    "fastq.otu_clustering",
    "fastq.profile_overrepresented_sequences",
    "fastq.trim_polyg_tails",
    "fastq.prepare_reference",
    "fastq.primer_normalization",
    "fastq.report_qc",
    "fastq.deplete_rrna",
    "fastq.profile_reads",
    "fastq.screen_taxonomy",
    "fastq.extract_umis",
    "fastq.correct_errors",
    "fastq.remove_duplicates",
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
