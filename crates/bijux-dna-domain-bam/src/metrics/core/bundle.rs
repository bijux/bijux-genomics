use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::complexity::ComplexityMetricsV1;
use super::coverage::{CoverageMetricsV1, CoverageUniformityV1, EffectiveCoverageV1};
use super::damage::{DamageComparisonV1, DamageMetricsV1};
use crate::metrics::downstream::{
    AuthenticityScoreV1, BamStageVerdictV1, ContaminationMetricsV1, ContaminationReconciliationV1,
    ContaminationSufficiencyV1, CoverageSufficiencyV1, GenotypingMetricsV1,
    HaplogroupSufficiencyV1, KinshipSufficiencyV1, SexInferenceV1, SexSufficiencyV1,
};
use crate::metrics::pre::{
    AlignmentCountsV1, FragmentLengthSummaryV1, IdxstatsSummaryV1, MapqSummaryV1,
};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BamMetricsBundleV1 {
    pub schema_version: String,
    pub alignment: AlignmentCountsV1,
    pub fragment_length: FragmentLengthSummaryV1,
    pub mapq: MapqSummaryV1,
    #[serde(default)]
    pub idxstats: IdxstatsSummaryV1,
    pub coverage: CoverageMetricsV1,
    #[serde(default)]
    pub coverage_uniformity: CoverageUniformityV1,
    #[serde(default)]
    pub effective_coverage: EffectiveCoverageV1,
    pub damage: DamageMetricsV1,
    #[serde(default)]
    pub damage_comparison: Option<DamageComparisonV1>,
    pub contamination: ContaminationMetricsV1,
    #[serde(default)]
    pub contamination_reconciliation: ContaminationReconciliationV1,
    pub sex: SexInferenceV1,
    pub complexity: ComplexityMetricsV1,
    #[serde(default)]
    pub authenticity: AuthenticityScoreV1,
    #[serde(default)]
    pub coverage_sufficiency: CoverageSufficiencyV1,
    #[serde(default)]
    pub sex_sufficiency: SexSufficiencyV1,
    #[serde(default)]
    pub contamination_sufficiency: ContaminationSufficiencyV1,
    #[serde(default)]
    pub haplogroup_sufficiency: HaplogroupSufficiencyV1,
    #[serde(default)]
    pub kinship_sufficiency: KinshipSufficiencyV1,
    #[serde(default)]
    pub stage_verdict: Option<BamStageVerdictV1>,
    pub genotyping: GenotypingMetricsV1,
}

impl BamMetricsBundleV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            schema_version: "bijux.bam_metrics.v1".to_string(),
            alignment: AlignmentCountsV1::empty(),
            fragment_length: FragmentLengthSummaryV1::empty(),
            mapq: MapqSummaryV1::empty(),
            idxstats: IdxstatsSummaryV1::empty(),
            coverage: CoverageMetricsV1::empty(),
            coverage_uniformity: CoverageUniformityV1::empty(),
            effective_coverage: EffectiveCoverageV1::empty(),
            damage: DamageMetricsV1::empty(),
            damage_comparison: None,
            contamination: ContaminationMetricsV1::empty(),
            contamination_reconciliation: ContaminationReconciliationV1::empty(),
            sex: SexInferenceV1::empty(),
            complexity: ComplexityMetricsV1::empty(),
            authenticity: AuthenticityScoreV1::empty(),
            coverage_sufficiency: CoverageSufficiencyV1::empty(),
            sex_sufficiency: SexSufficiencyV1::empty(),
            contamination_sufficiency: ContaminationSufficiencyV1::empty(),
            haplogroup_sufficiency: HaplogroupSufficiencyV1::empty(),
            kinship_sufficiency: KinshipSufficiencyV1::empty(),
            stage_verdict: None,
            genotyping: GenotypingMetricsV1::empty(),
        }
    }
}

pub type BamMetricsV1 = BamMetricsBundleV1;
