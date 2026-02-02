//! Canonical BAM stage graph and branching rules.

use bijux_core::domain::{PipelineDomain, PipelineSpec};

use crate::bam_stage_registry::BamStage;
use crate::metrics::CoverageMetricsV1;
use crate::params::RecalibrationSkipCriteria;

pub const BAM_CANONICAL_STAGE_ORDER: [BamStage; 15] = [
    BamStage::Validate,
    BamStage::QcPre,
    BamStage::Filter,
    BamStage::Markdup,
    BamStage::Complexity,
    BamStage::Coverage,
    BamStage::Damage,
    BamStage::Authenticity,
    BamStage::Contamination,
    BamStage::Sex,
    BamStage::BiasMitigation,
    BamStage::Recalibration,
    BamStage::Haplogroups,
    BamStage::Genotyping,
    BamStage::Kinship,
];

pub struct BamDomain;

impl PipelineDomain for BamDomain {
    fn domain_id() -> &'static str {
        "bam"
    }

    fn canonical_pipeline() -> PipelineSpec {
        PipelineSpec {
            stages: BAM_CANONICAL_STAGE_ORDER
                .iter()
                .map(|stage| stage.as_str().to_string())
                .collect(),
        }
    }
}

#[must_use]
pub fn should_skip_recalibration(
    coverage: &CoverageMetricsV1,
    criteria: &RecalibrationSkipCriteria,
) -> Option<String> {
    if coverage.mean < criteria.min_mean_coverage {
        return Some(format!(
            "mean coverage {:.2}x < {:.2}x",
            coverage.mean, criteria.min_mean_coverage
        ));
    }
    if coverage.breadth_1x < criteria.min_breadth_1x {
        return Some(format!(
            "breadth@1x {:.3} < {:.3}",
            coverage.breadth_1x, criteria.min_breadth_1x
        ));
    }
    None
}
