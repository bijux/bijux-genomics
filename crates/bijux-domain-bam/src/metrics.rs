//! Canonical BAM metrics schema v1.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::sample_meta::LibraryType;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AlignmentCountsV1 {
    pub total: u64,
    pub primary: u64,
    pub mapped: u64,
    pub proper_pair: u64,
    pub duplicates: u64,
}

impl AlignmentCountsV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            total: 0,
            primary: 0,
            mapped: 0,
            proper_pair: 0,
            duplicates: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FragmentLengthSummaryV1 {
    pub mean: f64,
    pub median: f64,
    pub p10: f64,
    pub p90: f64,
    pub short_fraction: f64,
}

impl FragmentLengthSummaryV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            mean: 0.0,
            median: 0.0,
            p10: 0.0,
            p90: 0.0,
            short_fraction: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MapqSummaryV1 {
    pub mean: f64,
    pub median: f64,
    pub p10: f64,
    pub p90: f64,
    pub histogram: Vec<(u8, u64)>,
}

impl MapqSummaryV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            mean: 0.0,
            median: 0.0,
            p10: 0.0,
            p90: 0.0,
            histogram: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CoverageMetricsV1 {
    pub mean: f64,
    pub median: f64,
    pub breadth_1x: f64,
    pub breadth_3x: f64,
    pub breadth_5x: f64,
}

impl CoverageMetricsV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            mean: 0.0,
            median: 0.0,
            breadth_1x: 0.0,
            breadth_3x: 0.0,
            breadth_5x: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CoverageUniformityV1 {
    pub coefficient_of_variation: f64,
    pub dropout_fraction: f64,
}

impl CoverageUniformityV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            coefficient_of_variation: 0.0,
            dropout_fraction: 0.0,
        }
    }
}

impl Default for CoverageUniformityV1 {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EffectiveCoverageV1 {
    pub raw: f64,
    pub dedup: f64,
    pub pmd_filtered: f64,
}

impl EffectiveCoverageV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            raw: 0.0,
            dedup: 0.0,
            pmd_filtered: 0.0,
        }
    }
}

impl Default for EffectiveCoverageV1 {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DamageMetricsV1 {
    pub c_to_t_5p: f64,
    pub g_to_a_3p: f64,
    pub pmd_score_histogram: Vec<(u8, u64)>,
}

impl DamageMetricsV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            c_to_t_5p: 0.0,
            g_to_a_3p: 0.0,
            pmd_score_histogram: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContaminationMetricsV1 {
    pub method: String,
    pub estimate: f64,
    pub ci_low: f64,
    pub ci_high: f64,
    pub assumptions: Vec<String>,
}

impl ContaminationMetricsV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            method: "unknown".to_string(),
            estimate: 0.0,
            ci_low: 0.0,
            ci_high: 0.0,
            assumptions: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SexConfidenceClass {
    Male,
    Female,
    Ambiguous,
    Insufficient,
}

impl Default for SexConfidenceClass {
    fn default() -> Self {
        Self::Insufficient
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SexInferenceV1 {
    pub x_to_y_ratio: f64,
    pub confidence: f64,
    pub method: String,
    #[serde(default)]
    pub classification: SexConfidenceClass,
    #[serde(default)]
    pub sufficient_data: bool,
}

impl SexInferenceV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            x_to_y_ratio: 0.0,
            confidence: 0.0,
            method: "unknown".to_string(),
            classification: SexConfidenceClass::Insufficient,
            sufficient_data: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ComplexityMetricsV1 {
    pub observed_reads: u64,
    pub projected_reads: Vec<(u64, u64)>,
}

impl ComplexityMetricsV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            observed_reads: 0,
            projected_reads: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuthenticityEvidenceV1 {
    pub damage_high: bool,
    pub fragments_short: bool,
    pub mapq_low_with_damage: bool,
}

impl AuthenticityEvidenceV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            damage_high: false,
            fragments_short: false,
            mapq_low_with_damage: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LibraryTypeInferenceV1 {
    pub inferred: LibraryType,
    pub confidence: f64,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TrimSuggestionV1 {
    pub trim_5p: u8,
    pub trim_3p: u8,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuthenticityScoreV1 {
    pub score: f64,
    pub confidence: f64,
    pub evidence: AuthenticityEvidenceV1,
    #[serde(default)]
    pub library_type_inference: Option<LibraryTypeInferenceV1>,
    #[serde(default)]
    pub trim_suggestion: Option<TrimSuggestionV1>,
    #[serde(default)]
    pub rationale: Vec<String>,
}

impl AuthenticityScoreV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            score: 0.0,
            confidence: 0.0,
            evidence: AuthenticityEvidenceV1::empty(),
            library_type_inference: None,
            trim_suggestion: None,
            rationale: Vec::new(),
        }
    }
}

impl Default for AuthenticityScoreV1 {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContaminationReconciliationV1 {
    pub mt_fraction: Option<f64>,
    pub nuclear_fraction: Option<f64>,
    pub assessment: String,
}

impl ContaminationReconciliationV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            mt_fraction: None,
            nuclear_fraction: None,
            assessment: "unknown".to_string(),
        }
    }
}

impl Default for ContaminationReconciliationV1 {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HaplogroupSufficiencyV1 {
    pub sufficient: bool,
    pub min_coverage: f64,
    pub reason: String,
}

impl HaplogroupSufficiencyV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            sufficient: false,
            min_coverage: 0.0,
            reason: "unknown".to_string(),
        }
    }
}

impl Default for HaplogroupSufficiencyV1 {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct KinshipSufficiencyV1 {
    pub sufficient: bool,
    pub overlap_snps: u32,
    pub reason: String,
}

impl KinshipSufficiencyV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            sufficient: false,
            overlap_snps: 0,
            reason: "unknown".to_string(),
        }
    }
}

impl Default for KinshipSufficiencyV1 {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GenotypingMetricsV1 {
    pub call_rate: f64,
    pub mean_posterior: f64,
    pub posterior_histogram: Vec<(u8, u64)>,
}

impl GenotypingMetricsV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            call_rate: 0.0,
            mean_posterior: 0.0,
            posterior_histogram: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BamMetricsV1 {
    pub schema_version: String,
    pub alignment: AlignmentCountsV1,
    pub fragment_length: FragmentLengthSummaryV1,
    pub mapq: MapqSummaryV1,
    pub coverage: CoverageMetricsV1,
    #[serde(default)]
    pub coverage_uniformity: CoverageUniformityV1,
    #[serde(default)]
    pub effective_coverage: EffectiveCoverageV1,
    pub damage: DamageMetricsV1,
    pub contamination: ContaminationMetricsV1,
    #[serde(default)]
    pub contamination_reconciliation: ContaminationReconciliationV1,
    pub sex: SexInferenceV1,
    pub complexity: ComplexityMetricsV1,
    #[serde(default)]
    pub authenticity: AuthenticityScoreV1,
    #[serde(default)]
    pub haplogroup_sufficiency: HaplogroupSufficiencyV1,
    #[serde(default)]
    pub kinship_sufficiency: KinshipSufficiencyV1,
    pub genotyping: GenotypingMetricsV1,
}

impl BamMetricsV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            schema_version: "bijux.bam_metrics.v1".to_string(),
            alignment: AlignmentCountsV1::empty(),
            fragment_length: FragmentLengthSummaryV1::empty(),
            mapq: MapqSummaryV1::empty(),
            coverage: CoverageMetricsV1::empty(),
            coverage_uniformity: CoverageUniformityV1::empty(),
            effective_coverage: EffectiveCoverageV1::empty(),
            damage: DamageMetricsV1::empty(),
            contamination: ContaminationMetricsV1::empty(),
            contamination_reconciliation: ContaminationReconciliationV1::empty(),
            sex: SexInferenceV1::empty(),
            complexity: ComplexityMetricsV1::empty(),
            authenticity: AuthenticityScoreV1::empty(),
            haplogroup_sufficiency: HaplogroupSufficiencyV1::empty(),
            kinship_sufficiency: KinshipSufficiencyV1::empty(),
            genotyping: GenotypingMetricsV1::empty(),
        }
    }
}
