//! Canonical BAM stage graph and branching rules.

use bijux_core::domain::{PipelineDomain, PipelineSpec};

pub const BAM_CANONICAL_STAGE_ORDER: [&str; 14] = [
    "bam.validate",
    "bam.qc_pre",
    "bam.filter",
    "bam.markdup",
    "bam.complexity",
    "bam.coverage",
    "bam.damage",
    "bam.contamination",
    "bam.sex",
    "bam.bias_mitigation",
    "bam.recalibration",
    "bam.haplogroups",
    "bam.genotyping",
    "bam.kinship",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BamBranchRule {
    pub condition: &'static str,
    pub action: &'static str,
}

pub const BAM_BRANCHING_RULES: [BamBranchRule; 2] = [
    BamBranchRule {
        condition: "low coverage detected (coverage.mean < 1x or breadth@1x < 0.1)",
        action: "skip bam.recalibration by default",
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
                .map(|stage| (*stage).to_string())
                .collect(),
        }
    }
}
