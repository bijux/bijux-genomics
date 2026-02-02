//! Single source of truth for BAM stage metadata and contracts.

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use bijux_core::StageId;

use crate::params::{
    AuthenticityEffectiveParams, BamEffectiveParams, BiasMitigationEffectiveParams,
    BqsrEffectiveParams, BqsrMode, ComplexityEffectiveParams, ContaminationEffectiveParams,
    ContaminationScope, CoverageEffectiveParams, DamageEffectiveParams, DuplicateAction,
    FilterEffectiveParams, GenotypingEffectiveParams, HaplogroupEffectiveParams,
    KinshipEffectiveParams, MarkDupEffectiveParams, OpticalDuplicatePolicy, QcPreEffectiveParams,
    RecalibrationSkipCriteria, SexEffectiveParams, UdgModel, UmiPolicy, ValidateEffectiveParams,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BamArtifactKind {
    Bam,
    BamIndex,
    ReferenceFasta,
    ReferenceIndex,
    ReferenceDict,
    BedRegions,
    Report,
}

#[derive(Debug, Clone, Copy)]
pub struct BamStageContract {
    pub input: BamArtifactKind,
    pub output: BamArtifactKind,
    pub emits_bam: bool,
    pub emits_report: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuditArtifact {
    pub name: &'static str,
    pub filename: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BamStage {
    Validate,
    QcPre,
    Filter,
    Markdup,
    Complexity,
    Coverage,
    Damage,
    Authenticity,
    Contamination,
    Sex,
    BiasMitigation,
    Recalibration,
    Haplogroups,
    Genotyping,
    Kinship,
}

impl BamStage {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            BamStage::Validate => "bam.validate",
            BamStage::QcPre => "bam.qc_pre",
            BamStage::Filter => "bam.filter",
            BamStage::Markdup => "bam.markdup",
            BamStage::Complexity => "bam.complexity",
            BamStage::Coverage => "bam.coverage",
            BamStage::Damage => "bam.damage",
            BamStage::Authenticity => "bam.authenticity",
            BamStage::Contamination => "bam.contamination",
            BamStage::Sex => "bam.sex",
            BamStage::BiasMitigation => "bam.bias_mitigation",
            BamStage::Recalibration => "bam.recalibration",
            BamStage::Haplogroups => "bam.haplogroups",
            BamStage::Genotyping => "bam.genotyping",
            BamStage::Kinship => "bam.kinship",
        }
    }

    #[must_use]
    pub fn id(self) -> StageId {
        StageId(self.as_str().to_string())
    }

    #[must_use]
    pub const fn all() -> &'static [BamStage] {
        &[
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
        ]
    }

    /// # Errors
    /// Returns an error if the provided JSON does not match the expected
    /// effective params schema for the stage.
    pub fn parse_effective_params(self, value: &serde_json::Value) -> Result<BamEffectiveParams> {
        match self {
            BamStage::Validate => serde_json::from_value::<ValidateEffectiveParams>(value.clone())
                .map(BamEffectiveParams::Validate),
            BamStage::QcPre => serde_json::from_value::<QcPreEffectiveParams>(value.clone())
                .map(BamEffectiveParams::QcPre),
            BamStage::Filter => serde_json::from_value::<FilterEffectiveParams>(value.clone())
                .map(BamEffectiveParams::Filter),
            BamStage::Markdup => serde_json::from_value::<MarkDupEffectiveParams>(value.clone())
                .map(BamEffectiveParams::Markdup),
            BamStage::Complexity => {
                serde_json::from_value::<ComplexityEffectiveParams>(value.clone())
                    .map(BamEffectiveParams::Complexity)
            }
            BamStage::Coverage => serde_json::from_value::<CoverageEffectiveParams>(value.clone())
                .map(BamEffectiveParams::Coverage),
            BamStage::Damage => serde_json::from_value::<DamageEffectiveParams>(value.clone())
                .map(BamEffectiveParams::Damage),
            BamStage::Authenticity => {
                serde_json::from_value::<AuthenticityEffectiveParams>(value.clone())
                    .map(BamEffectiveParams::Authenticity)
            }
            BamStage::Contamination => {
                serde_json::from_value::<ContaminationEffectiveParams>(value.clone())
                    .map(BamEffectiveParams::Contamination)
            }
            BamStage::Sex => serde_json::from_value::<SexEffectiveParams>(value.clone())
                .map(BamEffectiveParams::Sex),
            BamStage::BiasMitigation => {
                serde_json::from_value::<BiasMitigationEffectiveParams>(value.clone())
                    .map(BamEffectiveParams::BiasMitigation)
            }
            BamStage::Recalibration => serde_json::from_value::<BqsrEffectiveParams>(value.clone())
                .map(BamEffectiveParams::Recalibration),
            BamStage::Haplogroups => {
                serde_json::from_value::<HaplogroupEffectiveParams>(value.clone())
                    .map(BamEffectiveParams::Haplogroups)
            }
            BamStage::Genotyping => {
                serde_json::from_value::<GenotypingEffectiveParams>(value.clone())
                    .map(BamEffectiveParams::Genotyping)
            }
            BamStage::Kinship => serde_json::from_value::<KinshipEffectiveParams>(value.clone())
                .map(BamEffectiveParams::Kinship),
        }
        .map_err(|err| anyhow!("failed to parse params for {}: {err}", self.as_str()))
    }
}

impl TryFrom<&str> for BamStage {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        match value {
            "bam.validate" => Ok(BamStage::Validate),
            "bam.qc_pre" => Ok(BamStage::QcPre),
            "bam.filter" => Ok(BamStage::Filter),
            "bam.markdup" => Ok(BamStage::Markdup),
            "bam.complexity" => Ok(BamStage::Complexity),
            "bam.coverage" => Ok(BamStage::Coverage),
            "bam.damage" => Ok(BamStage::Damage),
            "bam.authenticity" => Ok(BamStage::Authenticity),
            "bam.contamination" => Ok(BamStage::Contamination),
            "bam.sex" => Ok(BamStage::Sex),
            "bam.bias_mitigation" => Ok(BamStage::BiasMitigation),
            "bam.recalibration" => Ok(BamStage::Recalibration),
            "bam.haplogroups" => Ok(BamStage::Haplogroups),
            "bam.genotyping" => Ok(BamStage::Genotyping),
            "bam.kinship" => Ok(BamStage::Kinship),
            _ => Err(anyhow!("unknown bam stage: {value}")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArtifactPolicy {
    pub required_outputs: &'static [&'static str],
    pub required_audit: &'static [AuditArtifact],
}

#[derive(Debug, Clone)]
pub struct BamStageSpec {
    pub stage: BamStage,
    pub required_inputs: &'static [&'static str],
    pub artifact_policy: ArtifactPolicy,
    pub allowed_tools: &'static [&'static str],
    pub default_tool: &'static str,
    pub default_params: BamEffectiveParams,
}

#[must_use]
#[allow(clippy::too_many_lines)]
pub fn required_audit_artifacts(stage: BamStage) -> &'static [AuditArtifact] {
    match stage {
        BamStage::Validate => &[
            AuditArtifact {
                name: "validation_report",
                filename: "validation.json",
            },
            AuditArtifact {
                name: "flagstat",
                filename: "flagstat.txt",
            },
            AuditArtifact {
                name: "stage_metrics",
                filename: "stage.metrics.json",
            },
        ],
        BamStage::QcPre => &[
            AuditArtifact {
                name: "qc_report",
                filename: "qc_pre.json",
            },
            AuditArtifact {
                name: "flagstat",
                filename: "flagstat.txt",
            },
            AuditArtifact {
                name: "stats",
                filename: "samtools_stats.txt",
            },
            AuditArtifact {
                name: "summary",
                filename: "qc_pre.summary.json",
            },
            AuditArtifact {
                name: "stage_metrics",
                filename: "stage.metrics.json",
            },
        ],
        BamStage::Filter => &[
            AuditArtifact {
                name: "filtered_bam",
                filename: "filtered.bam",
            },
            AuditArtifact {
                name: "filtered_bai",
                filename: "filtered.bam.bai",
            },
            AuditArtifact {
                name: "flagstat",
                filename: "flagstat.txt",
            },
            AuditArtifact {
                name: "idxstats",
                filename: "idxstats.txt",
            },
            AuditArtifact {
                name: "summary",
                filename: "filter.summary.json",
            },
            AuditArtifact {
                name: "stage_metrics",
                filename: "stage.metrics.json",
            },
        ],
        BamStage::Markdup => &[
            AuditArtifact {
                name: "markdup_bam",
                filename: "markdup.bam",
            },
            AuditArtifact {
                name: "markdup_bai",
                filename: "markdup.bam.bai",
            },
            AuditArtifact {
                name: "flagstat",
                filename: "flagstat.txt",
            },
            AuditArtifact {
                name: "idxstats",
                filename: "idxstats.txt",
            },
            AuditArtifact {
                name: "summary",
                filename: "markdup.summary.json",
            },
            AuditArtifact {
                name: "stage_metrics",
                filename: "stage.metrics.json",
            },
        ],
        BamStage::Complexity => &[
            AuditArtifact {
                name: "complexity_report",
                filename: "complexity.json",
            },
            AuditArtifact {
                name: "preseq",
                filename: "preseq.txt",
            },
            AuditArtifact {
                name: "summary",
                filename: "complexity.summary.json",
            },
            AuditArtifact {
                name: "stage_metrics",
                filename: "stage.metrics.json",
            },
        ],
        BamStage::Coverage => &[
            AuditArtifact {
                name: "coverage_report",
                filename: "coverage.json",
            },
            AuditArtifact {
                name: "coverage_summary",
                filename: "coverage.mosdepth.summary.txt",
            },
            AuditArtifact {
                name: "summary",
                filename: "coverage.summary.json",
            },
            AuditArtifact {
                name: "stage_metrics",
                filename: "stage.metrics.json",
            },
        ],
        BamStage::Damage => &[
            AuditArtifact {
                name: "damage_report",
                filename: "damage.json",
            },
            AuditArtifact {
                name: "damage_pydamage",
                filename: "damage.pydamage.json",
            },
            AuditArtifact {
                name: "damage_profiler",
                filename: "damage.profiler.json",
            },
            AuditArtifact {
                name: "summary",
                filename: "damage.summary.json",
            },
            AuditArtifact {
                name: "stage_metrics",
                filename: "stage.metrics.json",
            },
        ],
        BamStage::Authenticity => &[
            AuditArtifact {
                name: "authenticity_report",
                filename: "authenticity.json",
            },
            AuditArtifact {
                name: "summary",
                filename: "authenticity.summary.json",
            },
            AuditArtifact {
                name: "stage_metrics",
                filename: "stage.metrics.json",
            },
        ],
        BamStage::Contamination => &[
            AuditArtifact {
                name: "contamination_report",
                filename: "contamination.json",
            },
            AuditArtifact {
                name: "summary",
                filename: "contamination.summary.json",
            },
            AuditArtifact {
                name: "stage_metrics",
                filename: "stage.metrics.json",
            },
        ],
        BamStage::Sex => &[
            AuditArtifact {
                name: "sex_report",
                filename: "sex.json",
            },
            AuditArtifact {
                name: "summary",
                filename: "sex.summary.json",
            },
            AuditArtifact {
                name: "stage_metrics",
                filename: "stage.metrics.json",
            },
        ],
        BamStage::BiasMitigation => &[
            AuditArtifact {
                name: "bias_report",
                filename: "bias_mitigation.json",
            },
            AuditArtifact {
                name: "summary",
                filename: "bias_mitigation.summary.json",
            },
            AuditArtifact {
                name: "stage_metrics",
                filename: "stage.metrics.json",
            },
        ],
        BamStage::Recalibration => &[
            AuditArtifact {
                name: "recal_bam",
                filename: "recal.bam",
            },
            AuditArtifact {
                name: "recal_bai",
                filename: "recal.bam.bai",
            },
            AuditArtifact {
                name: "recal_report",
                filename: "recalibration.report.json",
            },
            AuditArtifact {
                name: "summary",
                filename: "recalibration.summary.json",
            },
            AuditArtifact {
                name: "stage_metrics",
                filename: "stage.metrics.json",
            },
        ],
        BamStage::Haplogroups => &[
            AuditArtifact {
                name: "haplogroups",
                filename: "haplogroups.json",
            },
            AuditArtifact {
                name: "summary",
                filename: "haplogroups.summary.json",
            },
            AuditArtifact {
                name: "stage_metrics",
                filename: "stage.metrics.json",
            },
        ],
        BamStage::Genotyping => &[
            AuditArtifact {
                name: "genotyping_report",
                filename: "genotyping.json",
            },
            AuditArtifact {
                name: "summary",
                filename: "genotyping.summary.json",
            },
            AuditArtifact {
                name: "stage_metrics",
                filename: "stage.metrics.json",
            },
        ],
        BamStage::Kinship => &[
            AuditArtifact {
                name: "kinship_report",
                filename: "kinship.json",
            },
            AuditArtifact {
                name: "summary",
                filename: "kinship.summary.json",
            },
            AuditArtifact {
                name: "stage_metrics",
                filename: "stage.metrics.json",
            },
        ],
    }
}

#[must_use]
#[allow(clippy::too_many_lines)]
pub fn stage_spec(stage: BamStage) -> BamStageSpec {
    match stage {
        BamStage::Validate => BamStageSpec {
            stage,
            required_inputs: &["bam"],
            artifact_policy: ArtifactPolicy {
                required_outputs: &["validation_report", "flagstat", "stage_metrics"],
                required_audit: required_audit_artifacts(stage),
            },
            allowed_tools: &["samtools"],
            default_tool: "samtools",
            default_params: BamEffectiveParams::Validate(ValidateEffectiveParams { strict: true }),
        },
        BamStage::QcPre => BamStageSpec {
            stage,
            required_inputs: &["bam"],
            artifact_policy: ArtifactPolicy {
                required_outputs: &["qc_report", "flagstat", "stats", "summary", "stage_metrics"],
                required_audit: required_audit_artifacts(stage),
            },
            allowed_tools: &["samtools"],
            default_tool: "samtools",
            default_params: BamEffectiveParams::QcPre(QcPreEffectiveParams { regions: None }),
        },
        BamStage::Filter => BamStageSpec {
            stage,
            required_inputs: &["bam"],
            artifact_policy: ArtifactPolicy {
                required_outputs: &[
                    "filtered_bam",
                    "filtered_bai",
                    "flagstat",
                    "idxstats",
                    "summary",
                    "stage_metrics",
                ],
                required_audit: required_audit_artifacts(stage),
            },
            allowed_tools: &["samtools"],
            default_tool: "samtools",
            default_params: BamEffectiveParams::Filter(FilterEffectiveParams {
                mapq_threshold: 30,
                include_flags: Vec::new(),
                exclude_flags: Vec::new(),
                min_length: 30,
                remove_duplicates: false,
                base_quality_threshold: 20,
            }),
        },
        BamStage::Markdup => BamStageSpec {
            stage,
            required_inputs: &["bam"],
            artifact_policy: ArtifactPolicy {
                required_outputs: &[
                    "markdup_bam",
                    "markdup_bai",
                    "flagstat",
                    "idxstats",
                    "summary",
                    "stage_metrics",
                ],
                required_audit: required_audit_artifacts(stage),
            },
            allowed_tools: &["gatk", "samtools"],
            default_tool: "gatk",
            default_params: BamEffectiveParams::Markdup(MarkDupEffectiveParams {
                optical_duplicates: OpticalDuplicatePolicy::MarkOnly,
                umi_policy: UmiPolicy::Ignore,
                duplicate_action: DuplicateAction::Mark,
            }),
        },
        BamStage::Complexity => BamStageSpec {
            stage,
            required_inputs: &["bam"],
            artifact_policy: ArtifactPolicy {
                required_outputs: &["complexity_report", "preseq", "summary", "stage_metrics"],
                required_audit: required_audit_artifacts(stage),
            },
            allowed_tools: &["preseq"],
            default_tool: "preseq",
            default_params: BamEffectiveParams::Complexity(ComplexityEffectiveParams {
                min_reads: 100_000,
                projection_points: vec![1_000_000, 2_000_000],
            }),
        },
        BamStage::Coverage => BamStageSpec {
            stage,
            required_inputs: &["bam"],
            artifact_policy: ArtifactPolicy {
                required_outputs: &[
                    "coverage_report",
                    "coverage_summary",
                    "summary",
                    "stage_metrics",
                ],
                required_audit: required_audit_artifacts(stage),
            },
            allowed_tools: &["mosdepth"],
            default_tool: "mosdepth",
            default_params: BamEffectiveParams::Coverage(CoverageEffectiveParams {
                regions: None,
                depth_thresholds: vec![1, 3, 5],
            }),
        },
        BamStage::Damage => BamStageSpec {
            stage,
            required_inputs: &["bam"],
            artifact_policy: ArtifactPolicy {
                required_outputs: &[
                    "damage_report",
                    "damage_pydamage",
                    "damage_profiler",
                    "summary",
                    "stage_metrics",
                ],
                required_audit: required_audit_artifacts(stage),
            },
            allowed_tools: &["pydamage"],
            default_tool: "pydamage",
            default_params: BamEffectiveParams::Damage(DamageEffectiveParams {
                udg_model: UdgModel::NonUdg,
                pmd_threshold_5p: 0.3,
                pmd_threshold_3p: 0.3,
                trim_5p: 2,
                trim_3p: 2,
            }),
        },
        BamStage::Authenticity => BamStageSpec {
            stage,
            required_inputs: &["bam"],
            artifact_policy: ArtifactPolicy {
                required_outputs: &["authenticity_report", "summary", "stage_metrics"],
                required_audit: required_audit_artifacts(stage),
            },
            allowed_tools: &["authenticity"],
            default_tool: "authenticity",
            default_params: BamEffectiveParams::Authenticity(AuthenticityEffectiveParams {
                mode: "aggregate".to_string(),
            }),
        },
        BamStage::Contamination => BamStageSpec {
            stage,
            required_inputs: &["bam"],
            artifact_policy: ArtifactPolicy {
                required_outputs: &["contamination_report", "summary", "stage_metrics"],
                required_audit: required_audit_artifacts(stage),
            },
            allowed_tools: &["authenticct"],
            default_tool: "authenticct",
            default_params: BamEffectiveParams::Contamination(ContaminationEffectiveParams {
                reference_panels: Vec::new(),
                scope: ContaminationScope::Both,
                prior: None,
                sex_specific: false,
                assumptions: None,
            }),
        },
        BamStage::Sex => BamStageSpec {
            stage,
            required_inputs: &["bam"],
            artifact_policy: ArtifactPolicy {
                required_outputs: &["sex_report", "summary", "stage_metrics"],
                required_audit: required_audit_artifacts(stage),
            },
            allowed_tools: &["rxy"],
            default_tool: "rxy",
            default_params: BamEffectiveParams::Sex(SexEffectiveParams {
                expected_sex: None,
                method: "rxy".to_string(),
            }),
        },
        BamStage::BiasMitigation => BamStageSpec {
            stage,
            required_inputs: &["bam"],
            artifact_policy: ArtifactPolicy {
                required_outputs: &["bias_report", "summary", "stage_metrics"],
                required_audit: required_audit_artifacts(stage),
            },
            allowed_tools: &["angsd"],
            default_tool: "angsd",
            default_params: BamEffectiveParams::BiasMitigation(BiasMitigationEffectiveParams {
                gc_bias_correction: true,
                map_bias_correction: false,
            }),
        },
        BamStage::Recalibration => BamStageSpec {
            stage,
            required_inputs: &["bam"],
            artifact_policy: ArtifactPolicy {
                required_outputs: &[
                    "recal_bam",
                    "recal_bai",
                    "recal_report",
                    "summary",
                    "stage_metrics",
                ],
                required_audit: required_audit_artifacts(stage),
            },
            allowed_tools: &["gatk"],
            default_tool: "gatk",
            default_params: BamEffectiveParams::Recalibration(BqsrEffectiveParams {
                known_sites: Vec::new(),
                mode: BqsrMode::Standard,
                skip_criteria: RecalibrationSkipCriteria {
                    min_mean_coverage: 2.0,
                    min_breadth_1x: 0.5,
                },
            }),
        },
        BamStage::Haplogroups => BamStageSpec {
            stage,
            required_inputs: &["bam"],
            artifact_policy: ArtifactPolicy {
                required_outputs: &["haplogroups", "summary", "stage_metrics"],
                required_audit: required_audit_artifacts(stage),
            },
            allowed_tools: &["yleaf"],
            default_tool: "yleaf",
            default_params: BamEffectiveParams::Haplogroups(HaplogroupEffectiveParams {
                reference_panel: "rcrs.fasta".to_string(),
                min_coverage: Some(5.0),
            }),
        },
        BamStage::Genotyping => BamStageSpec {
            stage,
            required_inputs: &["bam"],
            artifact_policy: ArtifactPolicy {
                required_outputs: &["genotyping_report", "summary", "stage_metrics"],
                required_audit: required_audit_artifacts(stage),
            },
            allowed_tools: &["angsd"],
            default_tool: "angsd",
            default_params: BamEffectiveParams::Genotyping(GenotypingEffectiveParams {
                caller: "angsd".to_string(),
                min_posterior: Some(0.8),
                min_call_rate: Some(0.7),
            }),
        },
        BamStage::Kinship => BamStageSpec {
            stage,
            required_inputs: &["bam"],
            artifact_policy: ArtifactPolicy {
                required_outputs: &["kinship_report", "summary", "stage_metrics"],
                required_audit: required_audit_artifacts(stage),
            },
            allowed_tools: &["king"],
            default_tool: "king",
            default_params: BamEffectiveParams::Kinship(KinshipEffectiveParams {
                reference_panel: "panel.vcf".to_string(),
                min_overlap_snps: 200,
            }),
        },
    }
}

#[must_use]
pub fn stage_specs() -> Vec<BamStageSpec> {
    BamStage::all()
        .iter()
        .map(|stage| stage_spec(*stage))
        .collect()
}

#[must_use]
pub fn contract_for_stage(stage_id: &str) -> Option<BamStageContract> {
    let stage = BamStage::try_from(stage_id).ok()?;
    match stage {
        BamStage::Filter | BamStage::Markdup | BamStage::Recalibration => Some(BamStageContract {
            input: BamArtifactKind::Bam,
            output: BamArtifactKind::Bam,
            emits_bam: true,
            emits_report: true,
        }),
        _ => Some(BamStageContract {
            input: BamArtifactKind::Bam,
            output: BamArtifactKind::Report,
            emits_bam: false,
            emits_report: true,
        }),
    }
}
