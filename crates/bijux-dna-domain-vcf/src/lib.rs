//! VCF domain primitives: stage IDs, typed params, metrics, and registry materialization.

pub mod params;

use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

pub const STAGE_PREFIX: &str = "vcf.";
pub const STAGE_CALL: &str = "vcf.call";
pub const STAGE_FILTER: &str = "vcf.filter";
pub const STAGE_STATS: &str = "vcf.stats";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VcfInvariantsPreset {
    Minimal,
}

impl VcfInvariantsPreset {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Minimal => "vcf_minimal",
        }
    }
}

pub const VCF_STAGE_ID_CATALOG: &[&str] = &[STAGE_CALL, STAGE_FILTER, STAGE_STATS];
pub const VCF_PARAMS_CATALOG: &[&str] = &[
    "bijux.vcf.call.params",
    "bijux.vcf.filter.params",
    "bijux.vcf.stats.params",
];
pub const VCF_METRICS_CATALOG: &[&str] = &["bijux.vcf.stats.v1"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VcfStage {
    Call,
    Filter,
    Stats,
}

impl VcfStage {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Call => STAGE_CALL,
            Self::Filter => STAGE_FILTER,
            Self::Stats => STAGE_STATS,
        }
    }

    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[Self::Call, Self::Filter, Self::Stats]
    }
}

impl TryFrom<&str> for VcfStage {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            STAGE_CALL => Ok(Self::Call),
            STAGE_FILTER => Ok(Self::Filter),
            STAGE_STATS => Ok(Self::Stats),
            _ => Err(anyhow!("unknown VCF stage: {value}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VcfStatsMetricsV1 {
    pub schema_version: String,
    pub variants_total: u64,
    pub snps: u64,
    pub indels: u64,
    pub ti_tv: Option<f64>,
    pub filter_breakdown: BTreeMap<String, u64>,
}

impl VcfStatsMetricsV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            schema_version: "bijux.vcf.stats.v1".to_string(),
            variants_total: 0,
            snps: 0,
            indels: 0,
            ti_tv: None,
            filter_breakdown: BTreeMap::new(),
        }
    }
}

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
