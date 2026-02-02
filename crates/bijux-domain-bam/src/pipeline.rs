//! Canonical BAM stage graph and branching rules.

use bijux_core::domain::{PipelineDomain, PipelineSpec};

use crate::metrics::CoverageMetricsV1;
use crate::params::{ContaminationScope, RecalibrationSkipCriteria, UdgModel};
use crate::sample_meta::LibraryType;
use crate::BamStage;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BamBranchRule {
    pub condition: &'static str,
    pub action: &'static str,
}

pub const BAM_BRANCHING_RULES: [BamBranchRule; 2] = [
    BamBranchRule {
        condition: "low coverage detected (coverage.mean < 1x or breadth@1x < 0.1)",
        action: "skip recalibration by default",
    },
    BamBranchRule {
        condition: "aDNA library type (non-UDG/half-UDG/UDG) set in SampleMeta",
        action: "adjust filter, damage, and contamination models accordingly",
    },
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BamLibraryPolicy {
    pub udg_model: UdgModel,
    pub min_length: u32,
    pub contamination_scope: ContaminationScope,
}

#[must_use]
pub fn policy_for_library_type(library_type: LibraryType) -> BamLibraryPolicy {
    match library_type {
        LibraryType::NonUdg => BamLibraryPolicy {
            udg_model: UdgModel::NonUdg,
            min_length: 30,
            contamination_scope: ContaminationScope::Both,
        },
        LibraryType::HalfUdg => BamLibraryPolicy {
            udg_model: UdgModel::HalfUdg,
            min_length: 28,
            contamination_scope: ContaminationScope::Both,
        },
        LibraryType::Udg => BamLibraryPolicy {
            udg_model: UdgModel::Udg,
            min_length: 25,
            contamination_scope: ContaminationScope::Nuclear,
        },
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
