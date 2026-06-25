//! BAM stage wiring (internal).

#[allow(dead_code)]
pub(crate) const STAGE_MODULES: &[&str] = &[
    "authenticity",
    "bias_mitigation",
    "complexity",
    "contamination",
    "coverage",
    "damage",
    "duplication_metrics",
    "endogenous_content",
    "filter",
    "gc_bias",
    "haplogroups",
    "insert_size",
    "kinship",
    "length_filter",
    "mapping_summary",
    "mapq_filter",
    "markdup",
    "overlap_correction",
    "qc_pre",
    "recalibration",
    "sex",
    "validate",
];

pub(crate) mod authenticity;
pub(crate) mod bias_mitigation;
pub(crate) mod complexity;
pub(crate) mod contamination;
pub(crate) mod coverage;
pub(crate) mod damage;
pub(crate) mod duplication_metrics;
pub(crate) mod endogenous_content;
pub(crate) mod filter;
pub(crate) mod gc_bias;
pub(crate) mod haplogroups;
pub(crate) mod insert_size;
pub(crate) mod kinship;
pub(crate) mod length_filter;
pub(crate) mod mapping_summary;
pub(crate) mod mapq_filter;
pub(crate) mod markdup;
pub(crate) mod overlap_correction;
pub(crate) mod qc_pre;
pub(crate) mod recalibration;
pub(crate) mod sex;
pub(crate) mod validate;
