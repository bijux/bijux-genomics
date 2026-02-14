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
    "fastq.validate_pre",
    "fastq.detect_adapters",
    "fastq.trim",
    "fastq.filter",
    "fastq.host_depletion",
    "fastq.length_distribution_pre",
    "fastq.low_complexity",
    "fastq.merge",
    "fastq.otu_clustering",
    "fastq.overrepresented_sequences",
    "fastq.polyg_tailing",
    "fastq.prepare_reference",
    "fastq.primer_normalization",
    "fastq.qc_post",
    "fastq.rrna",
    "fastq.stats_neutral",
    "fastq.screen",
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
    "bijux.fastq.trim.v1",
    "bijux.fastq.filter.v1",
    "bijux.fastq.merge.v1",
    "bijux.fastq.qc_post.v1",
    "bijux.fastq.stats_neutral.v1",
];
