use anyhow::{anyhow, Result};
use bijux_dna_pipelines::PipelineProfile;
use bijux_dna_planner_bam::stage_api::BamStage;
use bijux_dna_planner_bam::stage_api::params::{
    BamEffectiveParams, BqsrMode, ContaminationScope, DuplicateAction, OpticalDuplicatePolicy,
    UdgModel, UmiPolicy,
};
use bijux_dna_planner_bam::stage_api::types::{ExpectedSex, ReadGroupPolicy};

pub(super) fn default_params_for_stage(
    profile: &PipelineProfile,
    stage: BamStage,
) -> BamEffectiveParams {
    let stage_key = bijux_dna_core::ids::StageId::from_static(stage.as_str());
    profile
        .defaults
        .params
        .get(&stage_key)
        .map(bijux_dna_pipelines::DefaultParams::to_json)
        .and_then(|value| stage.parse_effective_params(&value).ok())
        .unwrap_or_else(|| bijux_dna_planner_bam::stage_api::stage_spec(stage).default_params)
}

pub(super) fn parse_read_group_policy(value: &str) -> Result<ReadGroupPolicy> {
    match value {
        "preserve" => Ok(ReadGroupPolicy::Preserve),
        "merge" => Ok(ReadGroupPolicy::Merge),
        "regenerate" => Ok(ReadGroupPolicy::Regenerate),
        _ => Err(anyhow!("unknown read group policy: {value}")),
    }
}

pub(super) fn parse_optical_duplicates(value: &str) -> Result<OpticalDuplicatePolicy> {
    match value {
        "none" => Ok(OpticalDuplicatePolicy::None),
        "mark_only" => Ok(OpticalDuplicatePolicy::MarkOnly),
        "remove" => Ok(OpticalDuplicatePolicy::Remove),
        _ => Err(anyhow!("unknown optical duplicate policy: {value}")),
    }
}

pub(super) fn parse_umi_policy(value: &str) -> Result<UmiPolicy> {
    match value {
        "ignore" => Ok(UmiPolicy::Ignore),
        "use_tag" => Ok(UmiPolicy::UseTag),
        "collapse" => Ok(UmiPolicy::Collapse),
        _ => Err(anyhow!("unknown UMI policy: {value}")),
    }
}

pub(super) fn parse_duplicate_action(value: &str) -> Result<DuplicateAction> {
    match value {
        "mark" => Ok(DuplicateAction::Mark),
        "remove" => Ok(DuplicateAction::Remove),
        _ => Err(anyhow!("unknown duplicate action: {value}")),
    }
}

pub(super) fn parse_udg_model(value: &str) -> Result<UdgModel> {
    match value {
        "non_udg" => Ok(UdgModel::NonUdg),
        "half_udg" => Ok(UdgModel::HalfUdg),
        "udg" => Ok(UdgModel::Udg),
        _ => Err(anyhow!("unknown UDG model: {value}")),
    }
}

pub(super) fn parse_contamination_scope(value: &str) -> Result<ContaminationScope> {
    match value {
        "mito" => Ok(ContaminationScope::Mito),
        "nuclear" => Ok(ContaminationScope::Nuclear),
        "both" => Ok(ContaminationScope::Both),
        _ => Err(anyhow!("unknown contamination scope: {value}")),
    }
}

pub(super) fn parse_expected_sex(value: &str) -> Result<ExpectedSex> {
    match value {
        "xx" => Ok(ExpectedSex::XX),
        "xy" => Ok(ExpectedSex::XY),
        "unknown" => Ok(ExpectedSex::Unknown),
        _ => Err(anyhow!("unknown expected sex: {value}")),
    }
}

pub(super) fn parse_bqsr_mode(value: &str) -> Result<BqsrMode> {
    match value {
        "standard" => Ok(BqsrMode::Standard),
        "skip" => Ok(BqsrMode::Skip),
        "emit_only" => Ok(BqsrMode::EmitOnly),
        _ => Err(anyhow!("unknown BQSR mode: {value}")),
    }
}

pub(super) fn parse_flag_list(values: &[String]) -> Result<Vec<u16>> {
    values
        .iter()
        .map(|value| {
            value
                .parse::<u16>()
                .map_err(|_| anyhow!("invalid flag value: {value}"))
        })
        .collect()
}
