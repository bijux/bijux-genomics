pub mod authenticity;
pub mod bias_mitigation;
pub mod complexity;
pub mod contamination;
pub mod coverage;
pub mod damage;
pub mod filter;
pub mod genotyping;
pub mod haplogroups;
pub mod kinship;
pub mod markdup;
pub mod qc_pre;
pub mod recalibration;
pub mod sex;
pub mod support;
pub mod validate;

#[must_use]
pub fn available_stages() -> &'static [&'static str] {
    &[
        "validate",
        "qc_pre",
        "filter",
        "markdup",
        "complexity",
        "coverage",
        "damage",
        "authenticity",
        "contamination",
        "sex",
        "bias_mitigation",
        "recalibration",
        "haplogroups",
        "genotyping",
        "kinship",
    ]
}
