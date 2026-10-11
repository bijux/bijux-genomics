use super::shell::{render_shell_command, rendered_stage_materialize_argv};
use crate::paths::path_relative_to_repo;
use crate::{
    BenchmarkLocalStageArtifactEntry as BenchLocalStageArtifactEntry,
    BenchmarkLocalStageCommandEntry as BenchLocalStageCommandEntry,
    BenchmarkLocalStageReadinessKind,
};
use anyhow::{anyhow, Result};
use bijux_dna_core::contract::ArtifactRef;
use bijux_dna_stage_contract::StagePlanV1;
use std::path::Path;

/// Canonical plans supplied by the domain planner composition.
#[derive(Debug, Clone)]
pub struct StagePlanBundle {
    pub stage_id: String,
    pub readiness_kind: BenchmarkLocalStageReadinessKind,
    pub plans: Vec<StagePlanV1>,
}
/// Project one declared stage's primary-tool plans into a portable command record.
///
/// # Errors
/// Refuses empty plans, inconsistent resources and memory unit overflow.
pub fn build_local_stage_command_entry(
    repo_root: &Path,
    bundle: StagePlanBundle,
) -> Result<BenchLocalStageCommandEntry> {
    let primary_tool_id = bundle
        .plans
        .first()
        .ok_or_else(|| {
            anyhow!("local benchmark stage `{}` did not yield any governed plans", bundle.stage_id)
        })?
        .tool_id
        .clone();
    let plans =
        bundle.plans.into_iter().filter(|plan| plan.tool_id == primary_tool_id).collect::<Vec<_>>();
    if plans.is_empty() {
        return Err(anyhow!(
            "local benchmark stage `{}` did not yield any governed plans",
            bundle.stage_id
        ));
    }

    let tool_id = plans[0].tool_id.as_str().to_string();
    let threads = plans[0].resources.threads.max(1);
    let memory_mb = memory_mb_for(plans[0].resources.mem_gb)?;
    for plan in &plans[1..] {
        if plan.resources.threads.max(1) != threads {
            return Err(anyhow!(
                "local benchmark stage `{}` mixes thread counts `{threads}` and `{}` across governed plans",
                bundle.stage_id,
                plan.resources.threads.max(1)
            ));
        }
        if memory_mb_for(plan.resources.mem_gb)? != memory_mb {
            return Err(anyhow!(
                "local benchmark stage `{}` mixes memory ceilings `{memory_mb}` and `{}` MB across governed plans",
                bundle.stage_id,
                memory_mb_for(plan.resources.mem_gb)?
            ));
        }
    }

    let argv = rendered_stage_materialize_argv(&bundle.stage_id);

    Ok(BenchLocalStageCommandEntry {
        stage_id: bundle.stage_id.clone(),
        readiness_kind: bundle.readiness_kind,
        tool_id,
        inputs: aggregate_plan_artifacts(repo_root, &plans, |plan| &plan.io.inputs),
        outputs: aggregate_plan_artifacts(repo_root, &plans, |plan| &plan.io.outputs),
        threads,
        memory_mb,
        command: render_shell_command(&argv),
        argv,
    })
}

fn aggregate_plan_artifacts<F>(
    repo_root: &Path,
    plans: &[StagePlanV1],
    select: F,
) -> Vec<BenchLocalStageArtifactEntry>
where
    F: Fn(&StagePlanV1) -> &[ArtifactRef],
{
    let mut artifacts = plans
        .iter()
        .flat_map(|plan| select(plan).iter())
        .map(|artifact| BenchLocalStageArtifactEntry {
            artifact_id: artifact.name.as_str().to_string(),
            path: path_relative_to_repo(repo_root, &artifact.path),
            role: artifact.role.as_str().to_string(),
            optional: artifact.optional,
        })
        .collect::<Vec<_>>();
    artifacts.sort();
    artifacts.dedup();
    artifacts
}

fn memory_mb_for(mem_gb: u32) -> Result<u32> {
    mem_gb
        .max(1)
        .checked_mul(1024)
        .ok_or_else(|| anyhow!("local stage memory exceeds the supported MiB range"))
}
