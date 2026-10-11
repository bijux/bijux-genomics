//! Owner: bijux-dna-bench
//! Deterministic construction of planned attempt records for one stage campaign.

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::contract::{
    validate_campaign_plan, validate_cycle_status_v2, validate_stage_campaign_registry,
};
use bijux_dna_bench_model::{
    BenchmarkAttemptIdentity, BenchmarkAttemptRecordV2, BenchmarkCampaignPlan,
    BenchmarkCycleStatusV2, BenchmarkLifecycleStatus, BenchmarkRunIdentity,
    BenchmarkStageCampaignRegistry,
};

/// Derive one planned attempt per declared tool for a governed stage campaign.
///
/// The caller supplies durable campaign and attempt identities. Tool ordering is canonicalized so
/// identical inputs always produce byte-equivalent records after serialization.
///
/// # Errors
/// Returns an error when the registry, selected run, timestamp, campaign identity, or attempt
/// identity is invalid.
pub fn plan_stage_campaign_attempts(
    registry: &BenchmarkStageCampaignRegistry,
    repository_commit: &str,
    campaign_registry_sha256: &str,
    run_id: &str,
    campaign_id: &str,
    attempt_id: &str,
    observed_at_utc: &str,
) -> Result<BenchmarkCampaignPlan> {
    validate_stage_campaign_registry(registry).context("validate benchmark campaign registry")?;
    let campaign = registry
        .campaigns
        .iter()
        .find(|campaign| campaign.run_id == run_id)
        .with_context(|| format!("benchmark campaign registry does not define {run_id}"))?;
    if campaign.tool_ids.is_empty() {
        bail!("benchmark campaign {run_id} has no declared tools");
    }

    let run = BenchmarkRunIdentity {
        run_id: campaign.run_id.clone(),
        domain: campaign.domain.clone(),
        stage_id: campaign.stage_id.clone(),
    };
    let mut tool_ids = campaign.tool_ids.clone();
    tool_ids.sort();
    let attempts = tool_ids
        .into_iter()
        .map(|tool_id| {
            BenchmarkAttemptRecordV2::planned(
                BenchmarkAttemptIdentity {
                    campaign_id: campaign_id.to_string(),
                    run: run.clone(),
                    attempt_id: attempt_id.to_string(),
                    tool_id,
                },
                observed_at_utc.to_string(),
            )
        })
        .collect();
    let cycle =
        BenchmarkCycleStatusV2::new(campaign_id.to_string(), observed_at_utc.to_string(), attempts);
    validate_cycle_status_v2(&cycle).context("validate planned benchmark campaign cycle")?;
    let plan = BenchmarkCampaignPlan::new(
        repository_commit.to_string(),
        campaign_registry_sha256.to_string(),
        cycle,
    );
    validate_campaign_plan(&plan).context("validate source-locked benchmark campaign plan")?;
    Ok(plan)
}

/// Derive one new planned attempt per tool while retaining complete terminal retry lineage.
///
/// `prior_attempts` must be a complete, terminal cycle snapshot. The unreferenced lineage tip for
/// each declared tool becomes the explicit supersession target of its new attempt.
///
/// # Errors
/// Returns an error when prior lineage is incomplete, nonterminal, source-inconsistent, or does
/// not provide exactly one terminal tip for every declared campaign tool.
pub fn plan_stage_campaign_retry_attempts(
    registry: &BenchmarkStageCampaignRegistry,
    repository_commit: &str,
    campaign_registry_sha256: &str,
    prior_attempts: Vec<BenchmarkAttemptRecordV2>,
    attempt_id: &str,
    observed_at_utc: &str,
) -> Result<BenchmarkCampaignPlan> {
    let first = prior_attempts.first().context("benchmark retry requires prior attempts")?;
    let campaign_id = first.identity.campaign_id.clone();
    let run = first.identity.run.clone();
    let run_id = first.identity.run.run_id.clone();
    let prior_cycle = BenchmarkCycleStatusV2::new(
        campaign_id.clone(),
        observed_at_utc.to_string(),
        prior_attempts.clone(),
    );
    validate_cycle_status_v2(&prior_cycle).context("validate prior benchmark retry lineage")?;
    if prior_attempts.iter().any(|attempt| {
        attempt.identity.campaign_id != campaign_id
            || attempt.identity.run != run
            || !matches!(
                attempt.current_status(),
                Some(
                    BenchmarkLifecycleStatus::Passed
                        | BenchmarkLifecycleStatus::Failed
                        | BenchmarkLifecycleStatus::Superseded
                        | BenchmarkLifecycleStatus::BlockedExternal
                )
            )
    }) {
        bail!("benchmark retry lineage is not one terminal campaign run");
    }
    let campaign = registry
        .campaigns
        .iter()
        .find(|campaign| campaign.run_id == run_id)
        .context("benchmark retry run is absent from the campaign registry")?;
    let referenced = prior_attempts
        .iter()
        .filter_map(|attempt| {
            attempt
                .supersedes_attempt_id
                .as_ref()
                .map(|prior| (attempt.identity.tool_id.as_str(), prior.as_str()))
        })
        .collect::<std::collections::BTreeSet<_>>();
    let mut tips = std::collections::BTreeMap::new();
    for attempt in &prior_attempts {
        if !referenced
            .contains(&(attempt.identity.tool_id.as_str(), attempt.identity.attempt_id.as_str()))
            && tips
                .insert(attempt.identity.tool_id.clone(), attempt.identity.attempt_id.clone())
                .is_some()
        {
            bail!("benchmark retry lineage has multiple terminal tips for one tool");
        }
    }
    let declared = campaign.tool_ids.iter().cloned().collect::<std::collections::BTreeSet<_>>();
    let observed = tips.keys().cloned().collect::<std::collections::BTreeSet<_>>();
    if observed != declared {
        bail!("benchmark retry lineage lacks the declared tool closure");
    }

    let mut attempts = prior_attempts;
    let mut tools = campaign.tool_ids.clone();
    tools.sort();
    for tool in tools {
        let prior_attempt_id = tips.get(&tool).context("benchmark retry tip is absent")?;
        let mut retry = BenchmarkAttemptRecordV2::planned(
            BenchmarkAttemptIdentity {
                campaign_id: campaign_id.clone(),
                run: run.clone(),
                attempt_id: attempt_id.to_string(),
                tool_id: tool.clone(),
            },
            observed_at_utc.to_string(),
        );
        retry.supersedes_attempt_id = Some(prior_attempt_id.clone());
        attempts.push(retry);
    }
    let cycle = BenchmarkCycleStatusV2::new(campaign_id, observed_at_utc.to_string(), attempts);
    validate_cycle_status_v2(&cycle).context("validate planned benchmark retry cycle")?;
    let plan = BenchmarkCampaignPlan::new(
        repository_commit.to_string(),
        campaign_registry_sha256.to_string(),
        cycle,
    );
    validate_campaign_plan(&plan).context("validate source-locked benchmark retry plan")?;
    Ok(plan)
}

/// Verify that a campaign plan is locked to the exact executing source and registry bytes.
///
/// # Errors
/// Returns an error when the plan is invalid, stale, or bound to different campaign declarations.
pub fn verify_campaign_plan_source(
    plan: &BenchmarkCampaignPlan,
    repository_commit: &str,
    campaign_registry_sha256: &str,
) -> Result<()> {
    validate_campaign_plan(plan).context("validate benchmark campaign plan")?;
    if plan.repository_commit != repository_commit {
        bail!("benchmark campaign plan repository commit is stale");
    }
    if plan.campaign_registry_sha256 != campaign_registry_sha256 {
        bail!("benchmark campaign plan registry digest is stale");
    }
    Ok(())
}
