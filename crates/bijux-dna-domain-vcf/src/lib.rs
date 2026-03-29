//! VCF domain primitives: stage IDs, typed params, metrics, and registry materialization.

pub mod contracts;
pub mod coverage;
pub mod metrics;
pub mod params;
pub mod stage_baseline;
pub mod taxonomy;

pub use metrics::{
    VcfCallSummaryMetricsV1, VcfFilterBreakdownMetricsV1, VcfStatsMetricsV1,
};
pub use stage_baseline::{
    VcfInvariantsPreset, VcfStage, STAGE_CALL, STAGE_FILTER_READS, STAGE_PREFIX, STAGE_STATS,
};
pub use taxonomy::{
    validate_downstream_transition, CoverageRegime, DomainSupportStatus, VcfDomainStage,
    VcfStageKind, VCF_FORBIDDEN_TRANSITIONS, VCF_STAGE_ORDER_DOWNSTREAM, VCF_STAGE_TAXONOMY,
};

pub const VCF_STAGE_ID_CATALOG: &[&str] = &[
    "vcf.admixture",
    "vcf.call",
    "vcf.call_diploid",
    "vcf.call_gl",
    "vcf.call_pseudohaploid",
    "vcf.damage_filter",
    "vcf.demography",
    "vcf.filter",
    "vcf.gl_propagation",
    "vcf.ibd",
    "vcf.imputation",
    "vcf.impute",
    "vcf.pca",
    "vcf.phasing",
    "vcf.population_structure",
    "vcf.postprocess",
    "vcf.prepare_reference_panel",
    "vcf.qc",
    "vcf.roh",
    "vcf.stats",
];
pub const VCF_PARAMS_CATALOG: &[&str] = &[
    "bijux.vcf.call.params",
    "bijux.vcf.filter.params",
    "bijux.vcf.stats.params",
];
pub const VCF_METRICS_CATALOG: &[&str] = &["bijux.vcf.stats.v1"];
pub const VCF_PRODUCTION_TOOLS: &[&str] = &["bcftools"];

#[must_use]
pub fn param_registry_toml() -> String {
    let mut out = String::new();
    out.push_str("# GENERATED - DO NOT EDIT - source: crates/bijux-dna-domain-vcf\n\n");
    out.push_str("[[params]]\nstage_id = \"vcf.call\"\nparam_type_id = \"bijux.vcf.call.params\"\nschema_version = \"bijux.vcf.params.v1\"\n\n");
    out.push_str("[[params]]\nstage_id = \"vcf.filter\"\nparam_type_id = \"bijux.vcf.filter.params\"\nschema_version = \"bijux.vcf.params.v1\"\n\n");
    out.push_str("[[params]]\nstage_id = \"vcf.stats\"\nparam_type_id = \"bijux.vcf.stats.params\"\nschema_version = \"bijux.vcf.params.v1\"\n");
    out
}

#[must_use]
pub fn required_tools_toml() -> String {
    let mut out = String::new();
    out.push_str("# GENERATED - DO NOT EDIT - source: crates/bijux-dna-domain-vcf\n");
    out.push_str("# source_commit: 53b050a6d117e40e0122777655e9d8cc428be9ad\n");
    out.push_str("# domain_schema_version: bijux.domain.v1\n\n");
    out.push_str("schema_version = \"bijux.required_tools.v1\"\n");
    out.push_str("required_tools = [\"bcftools\"]\n");
    out
}
