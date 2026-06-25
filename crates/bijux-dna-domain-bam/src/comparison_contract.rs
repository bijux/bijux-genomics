use std::sync::OnceLock;

use bijux_dna_core::ids::{StageId, ToolId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BamComparableStageContract {
    pub stage_id: StageId,
    pub compatible_tool_ids: Vec<ToolId>,
    pub shared_metrics: Vec<BamComparableMetricContract>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BamComparableMetricContract {
    pub name: String,
    pub meaning: String,
    pub scientific_threshold: Option<BamScientificThresholdContract>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BamScientificPassDirection {
    Minimum,
    Maximum,
    Range,
    ExactMatch,
    StructuredMatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BamScientificToleranceKind {
    RelativeFraction,
    AbsoluteDelta,
    ExactMatch,
    NormalizedSetOverlap,
    NormalizedRecordOverlap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BamScientificInsufficiencyPolicy {
    RefuseStageComparison,
    DropMetricFromStage,
    WarnAndExcludeStage,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BamScientificThresholdContract {
    pub pass_direction: BamScientificPassDirection,
    pub tolerance_kind: BamScientificToleranceKind,
    pub tolerance_value: f64,
    pub insufficiency_policy: BamScientificInsufficiencyPolicy,
}

#[derive(Debug, Deserialize)]
struct StageComparisonManifest {
    stage_id: String,
    #[serde(default)]
    compatible_tools: Vec<String>,
    #[serde(default)]
    metrics: Vec<MetricManifest>,
}

#[derive(Debug, Deserialize)]
struct MetricManifest {
    name: String,
    meaning: String,
    #[serde(default)]
    scientific_threshold: Option<BamScientificThresholdContract>,
}

fn comparable_stage_contracts_manifest() -> &'static [BamComparableStageContract] {
    static CONTRACTS: OnceLock<Vec<BamComparableStageContract>> = OnceLock::new();
    CONTRACTS.get_or_init(|| {
        [
            include_str!("../../../domain/bam/stages/align.yaml"),
            include_str!("../../../domain/bam/stages/authenticity.yaml"),
            include_str!("../../../domain/bam/stages/contamination.yaml"),
            include_str!("../../../domain/bam/stages/coverage.yaml"),
            include_str!("../../../domain/bam/stages/damage.yaml"),
            include_str!("../../../domain/bam/stages/duplication_metrics.yaml"),
            include_str!("../../../domain/bam/stages/filter.yaml"),
            include_str!("../../../domain/bam/stages/kinship.yaml"),
            include_str!("../../../domain/bam/stages/length_filter.yaml"),
            include_str!("../../../domain/bam/stages/mapping_summary.yaml"),
            include_str!("../../../domain/bam/stages/mapq_filter.yaml"),
            include_str!("../../../domain/bam/stages/markdup.yaml"),
            include_str!("../../../domain/bam/stages/qc_pre.yaml"),
            include_str!("../../../domain/bam/stages/sex.yaml"),
            include_str!("../../../domain/bam/stages/validate.yaml"),
        ]
        .into_iter()
        .map(|raw| {
            let manifest: StageComparisonManifest = bijux_dna_infra::formats::parse_yaml(raw)
                .unwrap_or_else(|err| panic!("parse BAM stage comparison manifest: {err}"));
            let mut compatible_tool_ids =
                manifest.compatible_tools.into_iter().map(ToolId::new).collect::<Vec<_>>();
            compatible_tool_ids.sort();
            compatible_tool_ids.dedup();

            BamComparableStageContract {
                stage_id: StageId::new(manifest.stage_id),
                compatible_tool_ids,
                shared_metrics: manifest
                    .metrics
                    .into_iter()
                    .map(|metric| BamComparableMetricContract {
                        name: metric.name,
                        meaning: metric.meaning,
                        scientific_threshold: metric.scientific_threshold,
                    })
                    .collect(),
            }
        })
        .filter(|contract| contract.compatible_tool_ids.len() >= 2)
        .collect()
    })
}

#[must_use]
pub fn comparable_benchmark_stage_contracts() -> Vec<BamComparableStageContract> {
    comparable_stage_contracts_manifest().to_vec()
}

#[must_use]
pub fn comparable_benchmark_stage_ids() -> Vec<StageId> {
    comparable_stage_contracts_manifest().iter().map(|contract| contract.stage_id.clone()).collect()
}

#[must_use]
pub fn comparable_benchmark_stage_contract_for_stage(
    stage_id: &StageId,
) -> Option<BamComparableStageContract> {
    comparable_stage_contracts_manifest()
        .iter()
        .find(|contract| contract.stage_id == *stage_id)
        .cloned()
}

#[must_use]
pub fn comparable_tool_ids_for_stage(stage_id: &StageId) -> Vec<ToolId> {
    comparable_benchmark_stage_contract_for_stage(stage_id)
        .map(|contract| contract.compatible_tool_ids)
        .unwrap_or_default()
}

#[must_use]
pub fn stage_comparable_metric_fields_for_stage(stage_id: &StageId) -> Vec<String> {
    stage_comparable_metric_contracts_for_stage(stage_id)
        .into_iter()
        .map(|metric| metric.name)
        .collect()
}

#[must_use]
pub fn stage_comparable_metric_contracts_for_stage(
    stage_id: &StageId,
) -> Vec<BamComparableMetricContract> {
    comparable_benchmark_stage_contract_for_stage(stage_id)
        .map(|contract| contract.shared_metrics)
        .unwrap_or_default()
}
