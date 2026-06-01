mod contig;
mod paths;
mod sample_meta;

pub use contig::ContigId;
pub use paths::{BaiPath, BamPath, BedRegions, DictPath, FaiPath, ReferenceFasta};
pub use sample_meta::{
    CaptureType, ExpectedSex, LibraryType, Platform, ReadGroupPolicy, SampleMeta, Strandedness,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BamInvariantsPreset {
    Default,
    Adna,
}

impl BamInvariantsPreset {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Adna => "adna",
        }
    }
}

pub const BAM_STAGE_ID_CATALOG: &[&str] = &[
    "bam.align",
    "bam.authenticity",
    "bam.bias_mitigation",
    "bam.complexity",
    "bam.contamination",
    "bam.coverage",
    "bam.damage",
    "bam.duplication_metrics",
    "bam.endogenous_content",
    "bam.filter",
    "bam.gc_bias",
    "bam.genotyping",
    "bam.haplogroups",
    "bam.insert_size",
    "bam.kinship",
    "bam.length_filter",
    "bam.mapping_summary",
    "bam.mapq_filter",
    "bam.markdup",
    "bam.overlap_correction",
    "bam.qc_pre",
    "bam.recalibration",
    "bam.sex",
    "bam.validate",
];

pub const BAM_LOCAL_BENCH_STAGE_ID_CATALOG: &[&str] = BAM_STAGE_ID_CATALOG;

pub const BAM_PARAMS_CATALOG: &[&str] = &[
    "bijux.bam.params.align.v1",
    "bijux.bam.params.validate.v1",
    "bijux.bam.params.qc_pre.v1",
    "bijux.bam.params.mapping_summary.v1",
    "bijux.bam.params.filter.v1",
    "bijux.bam.params.mapq_filter.v1",
    "bijux.bam.params.length_filter.v1",
    "bijux.bam.params.markdup.v1",
    "bijux.bam.params.duplication_metrics.v1",
    "bijux.bam.params.complexity.v1",
    "bijux.bam.params.coverage.v1",
    "bijux.bam.params.insert_size.v1",
    "bijux.bam.params.gc_bias.v1",
    "bijux.bam.params.endogenous_content.v1",
    "bijux.bam.params.overlap_correction.v1",
    "bijux.bam.params.damage.v1",
    "bijux.bam.params.authenticity.v1",
    "bijux.bam.params.contamination.v1",
    "bijux.bam.params.sex.v1",
    "bijux.bam.params.bias_mitigation.v1",
    "bijux.bam.params.recalibration.v1",
    "bijux.bam.params.haplogroups.v1",
    "bijux.bam.params.genotyping.v1",
    "bijux.bam.params.kinship.v1",
];

pub const BAM_METRICS_CATALOG: &[&str] = &[
    "bijux.bam.align.v1",
    "bijux.bam.validate.v1",
    "bijux.bam.qc_pre.v1",
    "bijux.bam.mapping_summary.v1",
    "bijux.bam.filter.v1",
    "bijux.bam.mapq_filter.v1",
    "bijux.bam.length_filter.v1",
    "bijux.bam.markdup.v1",
    "bijux.bam.duplication_metrics.v1",
    "bijux.bam.complexity.v1",
    "bijux.bam.coverage.v1",
    "bijux.bam.insert_size.v1",
    "bijux.bam.gc_bias.v1",
    "bijux.bam.endogenous_content.v1",
    "bijux.bam.overlap_correction.v1",
    "bijux.bam.damage.v1",
    "bijux.bam.authenticity.v1",
    "bijux.bam.contamination.v1",
    "bijux.bam.sex.v1",
    "bijux.bam.bias_mitigation.v1",
    "bijux.bam.recalibration.v1",
    "bijux.bam.haplogroups.v1",
    "bijux.bam.genotyping.v1",
    "bijux.bam.kinship.v1",
];
