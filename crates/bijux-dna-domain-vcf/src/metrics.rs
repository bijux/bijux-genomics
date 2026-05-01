use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VcfCallSummaryMetricsV1 {
    pub schema_version: String,
    pub sample_name: String,
    pub variants_called: u64,
    pub snps: u64,
    pub indels: u64,
}

impl VcfCallSummaryMetricsV1 {
    #[must_use]
    pub fn empty(sample_name: impl Into<String>) -> Self {
        Self {
            schema_version: "bijux.vcf.call_summary.v1".to_string(),
            sample_name: sample_name.into(),
            variants_called: 0,
            snps: 0,
            indels: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VcfFilterBreakdownMetricsV1 {
    pub schema_version: String,
    pub sample_name: String,
    pub variants_in: u64,
    pub variants_pass: u64,
    pub variants_filtered: u64,
    pub filter_breakdown: BTreeMap<String, u64>,
}

impl VcfFilterBreakdownMetricsV1 {
    #[must_use]
    pub fn empty(sample_name: impl Into<String>) -> Self {
        Self {
            schema_version: "bijux.vcf.filter_breakdown.v1".to_string(),
            sample_name: sample_name.into(),
            variants_in: 0,
            variants_pass: 0,
            variants_filtered: 0,
            filter_breakdown: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VcfStatsMetricsV1 {
    pub schema_version: String,
    pub sample_name: String,
    pub call_summary: VcfCallSummaryMetricsV1,
    pub filter_summary: VcfFilterBreakdownMetricsV1,
    pub variants_total: u64,
    #[serde(default)]
    pub sample_count: u64,
    pub snps: u64,
    pub indels: u64,
    pub ti_tv: Option<f64>,
    #[serde(default)]
    pub missingness_post: Option<f64>,
    #[serde(default)]
    pub heterozygosity_ratio: Option<f64>,
    #[serde(default)]
    pub annotation_coverage: Option<f64>,
    pub filter_breakdown: BTreeMap<String, u64>,
    pub depth_distribution: BTreeMap<String, u64>,
}

impl VcfStatsMetricsV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self::empty_for_sample("sample")
    }

    #[must_use]
    pub fn empty_for_sample(sample_name: impl Into<String>) -> Self {
        let sample_name = sample_name.into();
        Self {
            schema_version: "bijux.vcf.stats.v1".to_string(),
            sample_name: sample_name.clone(),
            call_summary: VcfCallSummaryMetricsV1::empty(sample_name.clone()),
            filter_summary: VcfFilterBreakdownMetricsV1::empty(sample_name),
            variants_total: 0,
            sample_count: 0,
            snps: 0,
            indels: 0,
            ti_tv: None,
            missingness_post: None,
            heterozygosity_ratio: None,
            annotation_coverage: None,
            filter_breakdown: BTreeMap::new(),
            depth_distribution: BTreeMap::new(),
        }
    }
}
