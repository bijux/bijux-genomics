//! One-shot preparation of independently ready benchmark campaign attempts.

use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_cycle_preparation, validate_utc_timestamp, BenchmarkCyclePreparation,
    BenchmarkCyclePreparationState, BenchmarkCyclePreparationUnit, BenchmarkExecutionSpec,
    BenchmarkLifecycleStatus,
};

use super::{
    ensure_execution_spec, ensure_stage_campaign_attempts, load_checked_in_benchmark_campaign_plan,
    load_execution_authorities, load_execution_spec,
};
use crate::workflow::{
    admit_benchmark_campaign_evidence, prepare_benchmark_campaign_material,
    BenchmarkCampaignMaterialRequest,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkCyclePreparationRequest {
    pub repository_root: PathBuf,
    pub study_root: PathBuf,
    pub data_index: PathBuf,
    pub inventory: PathBuf,
    pub benchmark_root: PathBuf,
    pub plan: PathBuf,
    pub operator: String,
    pub prepared_at_utc: String,
    pub account: String,
    pub partition: String,
}

/// Materialize every independently ready attempt without submitting or waiting.
///
/// # Errors
/// Returns an error only when shared source, plan, inventory, data, or root authorities are invalid.
/// Attempt-local artifact or material failures are returned in the corresponding unit.
pub fn prepare_checked_in_benchmark_campaign(
    request: &BenchmarkCyclePreparationRequest,
) -> Result<BenchmarkCyclePreparation> {
    validate_utc_timestamp(&request.prepared_at_utc, "benchmark preparation prepared_at_utc")?;
    let repository_root = request.repository_root.canonicalize()?;
    let study_root = request.study_root.canonicalize()?;
    let data_index = request.data_index.canonicalize()?;
    let inventory = request.inventory.canonicalize()?;
    let plan = load_checked_in_benchmark_campaign_plan(&repository_root, &request.plan)?;
    ensure_stage_campaign_attempts(&request.benchmark_root, &plan)?;
    let benchmark_root = request.benchmark_root.canonicalize()?;
    let authorities =
        load_execution_authorities(&repository_root, &inventory, &plan.repository_commit, &plan)?;
    let run_id = campaign_run_id(&plan)?;
    let evidence =
        admit_benchmark_campaign_evidence(&repository_root, &study_root, &data_index, run_id)?;
    let mut attempts = plan
        .cycle
        .attempts
        .iter()
        .filter(|attempt| attempt.transitions.len() == 1)
        .collect::<Vec<_>>();
    attempts.sort_by(|left, right| left.identity.tool_id.cmp(&right.identity.tool_id));
    let mut units = Vec::with_capacity(attempts.len());
    for attempt in attempts {
        let identity = &attempt.identity;
        let lifecycle = bijux_dna_runtime::load_attempt_lifecycle(&benchmark_root, identity)?;
        let prior_status = lifecycle.current_status();
        let result = match prior_status {
            Some(BenchmarkLifecycleStatus::Planned) => (|| -> Result<BenchmarkExecutionSpec> {
                let image = authorities.resolve_current_image(&study_root, &identity.tool_id)?;
                let material = prepare_benchmark_campaign_material(
                    &BenchmarkCampaignMaterialRequest {
                        repository_root: &repository_root,
                        benchmark_root: &benchmark_root,
                        identity,
                        source_contract_sha256: authorities.source_contract_sha256(),
                        operator: &request.operator,
                        prepared_at_utc: &request.prepared_at_utc,
                        account: &request.account,
                        partition: &request.partition,
                    },
                    &evidence,
                )?;
                let spec = authorities.build_execution_spec(&plan, identity, &image, material)?;
                ensure_execution_spec(&benchmark_root, &spec)?;
                bijux_dna_runtime::record_attempt_ready(
                    &benchmark_root,
                    identity,
                    &request.prepared_at_utc,
                )?;
                Ok(spec)
            })(),
            Some(BenchmarkLifecycleStatus::Ready) => (|| -> Result<BenchmarkExecutionSpec> {
                let spec = load_execution_spec(&benchmark_root, identity)?;
                let image = authorities.resolve_current_image(&study_root, &identity.tool_id)?;
                let indexed_input = spec
                    .inputs
                    .iter()
                    .find(|input| input.id == "corpus_data_index")
                    .context("prepared benchmark specification omits its data index")?;
                if spec.source.repository_path != repository_root.display().to_string()
                    || &spec.source.contract_sha256 != authorities.source_contract_sha256()
                    || spec.image.path != image.path.display().to_string()
                    || spec.image.sha256 != image.sha256
                    || indexed_input.path != data_index.display().to_string()
                    || spec.operator != request.operator.as_str()
                    || spec.resources.account != request.account.as_str()
                    || spec.resources.partition != request.partition.as_str()
                {
                    bail!("prepared benchmark specification differs from current authorities");
                }
                Ok(spec)
            })(),
            _ => {
                units.push(BenchmarkCyclePreparationUnit {
                    identity: identity.clone(),
                    prior_status,
                    current_status: prior_status,
                    state: BenchmarkCyclePreparationState::NotReady,
                    execution_spec: None,
                    error: None,
                });
                continue;
            }
        };
        match result {
            Ok(spec) => units.push(BenchmarkCyclePreparationUnit {
                identity: identity.clone(),
                prior_status,
                current_status: Some(BenchmarkLifecycleStatus::Ready),
                state: if prior_status == Some(BenchmarkLifecycleStatus::Ready) {
                    BenchmarkCyclePreparationState::AlreadyPrepared
                } else {
                    BenchmarkCyclePreparationState::Prepared
                },
                execution_spec: Some(spec),
                error: None,
            }),
            Err(error) => units.push(BenchmarkCyclePreparationUnit {
                identity: identity.clone(),
                prior_status,
                current_status: bijux_dna_runtime::load_attempt_lifecycle(
                    &benchmark_root,
                    identity,
                )
                .ok()
                .and_then(|value| value.current_status())
                .or(prior_status),
                state: BenchmarkCyclePreparationState::Failed,
                execution_spec: None,
                error: Some(format!("{error:#}")),
            }),
        }
    }
    let preparation = BenchmarkCyclePreparation::new(
        plan.cycle.campaign_id,
        request.prepared_at_utc.clone(),
        units,
    );
    validate_cycle_preparation(&preparation)?;
    Ok(preparation)
}

fn campaign_run_id(plan: &bijux_dna_bench_model::BenchmarkCampaignPlan) -> Result<&str> {
    let run_id = &plan
        .cycle
        .attempts
        .first()
        .context("benchmark campaign plan has no attempts")?
        .identity
        .run
        .run_id;
    if plan.cycle.attempts.iter().any(|attempt| attempt.identity.run.run_id != *run_id) {
        bail!("benchmark campaign plan mixes run identities");
    }
    Ok(run_id)
}
