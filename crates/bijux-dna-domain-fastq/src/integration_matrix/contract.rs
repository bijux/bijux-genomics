use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::Deserialize;

use super::model::ToolIntegrationLevel;

#[derive(Debug, Deserialize)]
pub(super) struct DomainIndexContract {
    pub stage_tool_integration: BTreeMap<String, BTreeMap<String, ToolIntegrationLevel>>,
    #[serde(default)]
    pub reference_index_compatibility: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub stage_sanity_metrics: BTreeMap<String, Vec<String>>,
    pub benchmark_scenarios: BTreeMap<String, BenchmarkScenarioRecord>,
}

#[derive(Debug, Deserialize)]
pub(super) struct BenchmarkScenarioRecord {
    pub stage_id: String,
    pub description: String,
    pub fairness_rules: Vec<String>,
    pub cohort_artifact_id: String,
    pub comparison_artifact_id: String,
    pub normalization_artifact_id: String,
}

pub(super) fn domain_index_contract() -> &'static DomainIndexContract {
    static CONTRACT: OnceLock<DomainIndexContract> = OnceLock::new();
    CONTRACT.get_or_init(|| {
        bijux_dna_infra::formats::parse_yaml(include_str!("../../../../domain/fastq/index.yaml"))
            .unwrap_or_else(|err| {
                panic!("parse domain/fastq/index.yaml integration contract: {err}")
            })
    })
}
