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
    "fastq.contaminant_screen",
    "fastq.damage_aware_pretrim",
    "fastq.validate_reads",
    "fastq.detect_adapters",
    "fastq.trim_reads",
    "fastq.filter_reads",
    "fastq.host_depletion",
    "fastq.profile_read_lengths",
    "fastq.low_complexity",
    "fastq.merge",
    "fastq.otu_clustering",
    "fastq.profile_overrepresented_sequences",
    "fastq.polyg_tailing",
    "fastq.prepare_reference",
    "fastq.primer_normalization",
    "fastq.report_qc",
    "fastq.rrna",
    "fastq.profile_reads",
    "fastq.screen_taxonomy",
    "fastq.umi",
    "fastq.correct",
    "fastq.deduplicate",
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
    "bijux.fastq.merge.v1",
    "bijux.fastq.report_qc.v1",
    "bijux.fastq.profile_reads.v1",
];
