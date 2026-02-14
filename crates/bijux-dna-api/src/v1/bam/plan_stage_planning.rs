use std::path::Path;

use anyhow::{anyhow, Result};
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_infra::hash_file_sha256;
use bijux_dna_pipelines::registry;
use bijux_dna_pipelines::PipelineProfile;
use bijux_dna_planner_bam::stage_api::{BamStage, StagePlanRequest};
use bijux_dna_stage_contract::StagePlanV1;

use crate::request_args::BamRunArgs;
#[path = "plan_parse.rs"]
mod plan_parse;
use self::plan_parse::{
    default_params_for_stage, parse_bqsr_mode, parse_contamination_scope, parse_duplicate_action,
    parse_expected_sex, parse_flag_list, parse_optical_duplicates, parse_read_group_policy,
    parse_udg_model, parse_umi_policy,
};

fn stage_status(stage_id: &str) -> Option<String> {
    let cwd = std::env::current_dir().ok()?;
    let path = bijux_dna_infra::configs_file(&cwd, "ci/stages/stages.toml");
    let raw = std::fs::read_to_string(path).ok()?;
    let parsed = raw.parse::<toml::Value>().ok()?;
    let entries = parsed.get("stages")?.as_array()?;
    entries.iter().find_map(|entry| {
        let id = entry.get("id").and_then(toml::Value::as_str)?;
        if id == stage_id {
            entry
                .get("status")
                .and_then(toml::Value::as_str)
                .map(std::string::ToString::to_string)
        } else {
            None
        }
    })
}

fn enforce_stage_status(stage_id: &str, allow_planned: bool) -> Result<()> {
    match stage_status(stage_id).as_deref() {
        Some("supported") | None => Ok(()),
        Some("planned" | "out_of_scope") if allow_planned => Ok(()),
        Some("planned" | "out_of_scope") => Err(anyhow!(
            "stage {stage_id} is not active in current scope; re-run with --allow-planned to override"
        )),
        Some(other) => Err(anyhow!("stage {stage_id} has unknown status {other}")),
    }
}

/// # Errors
/// Returns an error if planning fails for the stage.
pub fn plan_for_bam_stage(
    stage: bijux_dna_planner_bam::stage_api::BamStage,
    spec: &ToolExecutionSpecV1,
    args: &BamRunArgs,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    let profile = registry::profile_by_id(bijux_dna_pipelines::Domain::Bam, &args.profile)?;
    plan_for_bam_stage_with_profile(stage, spec, args, &profile, out_dir)
}

#[allow(clippy::too_many_lines)]
/// # Errors
/// Returns an error if stage arguments are invalid or planning fails.
pub fn plan_for_bam_stage_with_profile(
    stage: bijux_dna_planner_bam::stage_api::BamStage,
    spec: &ToolExecutionSpecV1,
    args: &BamRunArgs,
    profile: &PipelineProfile,
    out_dir: &Path,
) -> Result<StagePlanV1> {
    enforce_stage_status(stage.as_str(), args.allow_planned)?;
    if !super::feature_flags::downstream_enabled()
        && matches!(
            stage,
            bijux_dna_planner_bam::stage_api::BamStage::Haplogroups
                | bijux_dna_planner_bam::stage_api::BamStage::Genotyping
                | bijux_dna_planner_bam::stage_api::BamStage::Kinship
                | bijux_dna_planner_bam::stage_api::BamStage::BiasMitigation
        )
    {
        return Err(anyhow!(
            "downstream BAM stages are disabled (enable feature 'bam_downstream')"
        ));
    }

    if let Some(plan) = plan_alignment_qc_stage(stage, spec, args, profile, out_dir)? {
        return Ok(plan);
    }
    if let Some(plan) = plan_damage_recalibration_stage(stage, spec, args, profile, out_dir)? {
        return Ok(plan);
    }
    if let Some(plan) = plan_downstream_stage(stage, spec, args, profile, out_dir)? {
        return Ok(plan);
    }

    Err(anyhow!("unhandled BAM stage {}", stage.as_str()))
}

include!("plan_stage_sections/alignment_qc_handlers.rs");
include!("plan_stage_sections/damage_recalibration_handlers.rs");
include!("plan_stage_sections/downstream_handlers.rs");
