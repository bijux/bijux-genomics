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
    "bam.validate",
    "bam.mapping_summary",
    "bam.qc_pre",
    "bam.filter",
    "bam.markdup",
    "bam.complexity",
    "bam.coverage",
    "bam.insert_size",
    "bam.gc_bias",
    "bam.damage",
    "bam.authenticity",
    "bam.contamination",
    "bam.sex",
    "bam.bias_mitigation",
    "bam.recalibration",
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
