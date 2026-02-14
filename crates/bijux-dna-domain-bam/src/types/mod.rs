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

pub const BAM_PARAMS_CATALOG: &[&str] = &[
    "bijux.bam.params.align.v1",
    "bijux.bam.params.filter.v1",
    "bijux.bam.params.markdup.v1",
    "bijux.bam.params.coverage.v1",
    "bijux.bam.params.damage.v1",
];

pub const BAM_METRICS_CATALOG: &[&str] = &[
    "bijux.bam.validate.v1",
    "bijux.bam.mapping_summary.v1",
    "bijux.bam.coverage.v1",
    "bijux.bam.damage.v1",
    "bijux.bam.duplication.v1",
];
