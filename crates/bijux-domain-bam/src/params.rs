//! Owner: bijux-domain-bam
//! Canonical effective parameters for BAM stages.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::artifacts::BedRegions;
use crate::sample_meta::ExpectedSex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OpticalDuplicatePolicy {
    None,
    MarkOnly,
    Remove,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UmiPolicy {
    Ignore,
    UseTag,
    Collapse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DuplicateAction {
    Mark,
    Remove,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UdgModel {
    NonUdg,
    HalfUdg,
    Udg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ContaminationScope {
    Mito,
    Nuclear,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BqsrMode {
    Standard,
    Skip,
    EmitOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ValidateEffectiveParams {
    pub strict: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QcPreEffectiveParams {
    #[serde(default)]
    pub regions: Option<BedRegions>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FilterEffectiveParams {
    pub mapq_threshold: u8,
    pub include_flags: Vec<u16>,
    pub exclude_flags: Vec<u16>,
    pub min_length: u32,
    pub remove_duplicates: bool,
    pub base_quality_threshold: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MarkDupEffectiveParams {
    pub optical_duplicates: OpticalDuplicatePolicy,
    pub umi_policy: UmiPolicy,
    pub duplicate_action: DuplicateAction,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ComplexityEffectiveParams {
    pub min_reads: u64,
    pub projection_points: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CoverageEffectiveParams {
    #[serde(default)]
    pub regions: Option<BedRegions>,
    pub depth_thresholds: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DamageEffectiveParams {
    pub udg_model: UdgModel,
    pub pmd_threshold_5p: f64,
    pub pmd_threshold_3p: f64,
    pub trim_5p: u8,
    pub trim_3p: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContaminationEffectiveParams {
    pub reference_panels: Vec<String>,
    pub scope: ContaminationScope,
    #[serde(default)]
    pub prior: Option<f64>,
    pub sex_specific: bool,
    #[serde(default)]
    pub assumptions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SexEffectiveParams {
    #[serde(default)]
    pub expected_sex: Option<ExpectedSex>,
    pub method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BiasMitigationEffectiveParams {
    pub gc_bias_correction: bool,
    pub map_bias_correction: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RecalibrationSkipCriteria {
    pub min_mean_coverage: f64,
    pub min_breadth_1x: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BqsrEffectiveParams {
    pub known_sites: Vec<String>,
    pub mode: BqsrMode,
    pub skip_criteria: RecalibrationSkipCriteria,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HaplogroupEffectiveParams {
    pub reference_panel: String,
    #[serde(default)]
    pub min_coverage: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GenotypingEffectiveParams {
    pub caller: String,
    #[serde(default)]
    pub min_posterior: Option<f64>,
    #[serde(default)]
    pub min_call_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct KinshipEffectiveParams {
    pub reference_panel: String,
    pub min_overlap_snps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "stage", rename_all = "snake_case")]
pub enum BamEffectiveParams {
    Validate(ValidateEffectiveParams),
    QcPre(QcPreEffectiveParams),
    Filter(FilterEffectiveParams),
    Markdup(MarkDupEffectiveParams),
    Complexity(ComplexityEffectiveParams),
    Coverage(CoverageEffectiveParams),
    Damage(DamageEffectiveParams),
    Contamination(ContaminationEffectiveParams),
    Sex(SexEffectiveParams),
    BiasMitigation(BiasMitigationEffectiveParams),
    Recalibration(BqsrEffectiveParams),
    Haplogroups(HaplogroupEffectiveParams),
    Genotyping(GenotypingEffectiveParams),
    Kinship(KinshipEffectiveParams),
}

#[must_use]
#[allow(dead_code)]
pub fn parse_effective_params(
    stage_id: &str,
    value: &serde_json::Value,
) -> Option<BamEffectiveParams> {
    match stage_id {
        "bam.validate" => serde_json::from_value::<ValidateEffectiveParams>(value.clone())
            .ok()
            .map(BamEffectiveParams::Validate),
        "bam.qc_pre" => serde_json::from_value::<QcPreEffectiveParams>(value.clone())
            .ok()
            .map(BamEffectiveParams::QcPre),
        "bam.filter" => serde_json::from_value::<FilterEffectiveParams>(value.clone())
            .ok()
            .map(BamEffectiveParams::Filter),
        "bam.markdup" => serde_json::from_value::<MarkDupEffectiveParams>(value.clone())
            .ok()
            .map(BamEffectiveParams::Markdup),
        "bam.complexity" => serde_json::from_value::<ComplexityEffectiveParams>(value.clone())
            .ok()
            .map(BamEffectiveParams::Complexity),
        "bam.coverage" => serde_json::from_value::<CoverageEffectiveParams>(value.clone())
            .ok()
            .map(BamEffectiveParams::Coverage),
        "bam.damage" => serde_json::from_value::<DamageEffectiveParams>(value.clone())
            .ok()
            .map(BamEffectiveParams::Damage),
        "bam.contamination" => {
            serde_json::from_value::<ContaminationEffectiveParams>(value.clone())
                .ok()
                .map(BamEffectiveParams::Contamination)
        }
        "bam.sex" => serde_json::from_value::<SexEffectiveParams>(value.clone())
            .ok()
            .map(BamEffectiveParams::Sex),
        "bam.bias_mitigation" => {
            serde_json::from_value::<BiasMitigationEffectiveParams>(value.clone())
                .ok()
                .map(BamEffectiveParams::BiasMitigation)
        }
        "bam.recalibration" => serde_json::from_value::<BqsrEffectiveParams>(value.clone())
            .ok()
            .map(BamEffectiveParams::Recalibration),
        "bam.haplogroups" => serde_json::from_value::<HaplogroupEffectiveParams>(value.clone())
            .ok()
            .map(BamEffectiveParams::Haplogroups),
        "bam.genotyping" => serde_json::from_value::<GenotypingEffectiveParams>(value.clone())
            .ok()
            .map(BamEffectiveParams::Genotyping),
        "bam.kinship" => serde_json::from_value::<KinshipEffectiveParams>(value.clone())
            .ok()
            .map(BamEffectiveParams::Kinship),
        _ => None,
    }
}
