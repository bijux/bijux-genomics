use crate::params::{
    AuthenticityEffectiveParams, BamEffectiveParams, BiasMitigationEffectiveParams,
    BqsrEffectiveParams, BqsrMode, ContaminationEffectiveParams, ContaminationScope,
    DamageEffectiveParams, GenotypingEffectiveParams, HaplogroupEffectiveParams,
    KinshipEffectiveParams, RecalibrationSkipCriteria, SexEffectiveParams, UdgModel,
};
use crate::{ArtifactPolicy, BamStage, BamStageSpec};

#[must_use]
#[allow(clippy::too_many_lines)]
pub fn stage_spec_downstream(stage: BamStage) -> Option<BamStageSpec> {
    let spec = match stage {
        BamStage::Damage => BamStageSpec {
            stage,
            required_inputs: &["bam"],
            artifact_policy: ArtifactPolicy {
                required_outputs: &["damage_pydamage", "damage_mapdamage2", "stage_metrics"],
                required_audit: super::required_audit_artifacts(stage),
            },
            allowed_tools: &["pydamage", "mapdamage2"],
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
                required_audit: super::required_audit_artifacts(stage),
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
                required_audit: super::required_audit_artifacts(stage),
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
                required_audit: super::required_audit_artifacts(stage),
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
                required_audit: super::required_audit_artifacts(stage),
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
                required_audit: super::required_audit_artifacts(stage),
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
                required_audit: super::required_audit_artifacts(stage),
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
                required_audit: super::required_audit_artifacts(stage),
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
                required_audit: super::required_audit_artifacts(stage),
            },
            allowed_tools: &["king"],
            default_tool: "king",
            default_params: BamEffectiveParams::Kinship(KinshipEffectiveParams {
                reference_panel: "panel.vcf".to_string(),
                min_overlap_snps: 200,
            }),
        },
        _ => return None,
    };
    Some(spec)
}
