//! Owner: bijux-dna-bench
//! Deterministic construction of planned attempt records for one stage campaign.

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::contract::{
    validate_campaign_plan, validate_cycle_status_v2, validate_stage_campaign_registry,
};
use bijux_dna_bench_model::{
    BenchmarkAttemptIdentity, BenchmarkAttemptRecordV2, BenchmarkCampaignPlan,
    BenchmarkCaseMatrixV2, BenchmarkCycleStatusV2, BenchmarkLifecycleStatus, BenchmarkRunIdentity,
    BenchmarkStageCampaignRegistry,
};

use crate::workflow::{benchmark_case_matrix_sha256, validate_benchmark_case_matrix};

fn terminal_tip_requires_retry(status: BenchmarkLifecycleStatus) -> bool {
    status != BenchmarkLifecycleStatus::Passed
}

/// Derive one independently addressable attempt per generic benchmark case.
///
/// Case identity is retained in every attempt and in its source-locked campaign plan, so two
/// corpora for the same run/tool pair cannot collide in lifecycle storage or scheduler control.
///
/// # Errors
/// Returns an error when the matrix, timestamp, campaign identity, attempt identity, or resulting
/// case-scoped cycle is invalid.
pub fn plan_benchmark_case_attempts(
    matrix: &BenchmarkCaseMatrixV2,
    campaign_id: &str,
    attempt_id: &str,
    observed_at_utc: &str,
) -> Result<BenchmarkCampaignPlan> {
    validate_benchmark_case_matrix(matrix).context("validate benchmark case matrix")?;
    let mut cases = matrix.cases.iter().collect::<Vec<_>>();
    cases.sort_by(|left, right| left.case_id.cmp(&right.case_id));
    let attempts = cases
        .into_iter()
        .map(|case| {
            BenchmarkAttemptRecordV2::planned(
                BenchmarkAttemptIdentity {
                    campaign_id: campaign_id.to_string(),
                    run: BenchmarkRunIdentity {
                        run_id: case.run_id.clone(),
                        domain: case.domain.clone(),
                        stage_id: case.stage_id.clone(),
                    },
                    case_id: Some(case.case_id.clone()),
                    attempt_id: attempt_id.to_string(),
                    tool_id: case.tool_id.clone(),
                },
                observed_at_utc.to_string(),
            )
        })
        .collect();
    let cycle =
        BenchmarkCycleStatusV2::new(campaign_id.to_string(), observed_at_utc.to_string(), attempts);
    validate_cycle_status_v2(&cycle).context("validate case-scoped benchmark campaign cycle")?;
    let plan = BenchmarkCampaignPlan::for_case_matrix(
        matrix.repository_commit.clone(),
        matrix.campaign_registry_sha256.clone(),
        benchmark_case_matrix_sha256(matrix)?,
        cycle,
    );
    validate_campaign_plan(&plan).context("validate case-scoped benchmark campaign plan")?;
    Ok(plan)
}

/// Plan one retry for every non-passing terminal case lineage in an exact generic case matrix.
///
/// The matrix provides the complete expected case closure. Each case/tool pair must have exactly
/// one unreferenced terminal lineage tip, and every retained prior attempt must belong to that
/// matrix. Passed tips remain the selected evidence; every other terminal tip receives a new
/// attempt with its case identity and explicit supersession edge preserved.
///
/// # Errors
/// Returns an error when matrix closure, prior lifecycle state, lineage, retry identity, timestamp,
/// or the resulting source-locked plan is invalid.
pub fn plan_benchmark_case_retry_attempts(
    matrix: &BenchmarkCaseMatrixV2,
    prior_attempts: Vec<BenchmarkAttemptRecordV2>,
    attempt_id: &str,
    observed_at_utc: &str,
) -> Result<BenchmarkCampaignPlan> {
    validate_benchmark_case_matrix(matrix).context("validate benchmark retry case matrix")?;
    let first = prior_attempts.first().context("benchmark case retry requires prior attempts")?;
    let campaign_id = first.identity.campaign_id.clone();
    let prior_cycle = BenchmarkCycleStatusV2::new(
        campaign_id.clone(),
        observed_at_utc.to_string(),
        prior_attempts.clone(),
    );
    validate_cycle_status_v2(&prior_cycle).context("validate prior case retry lineage")?;

    let cases = matrix
        .cases
        .iter()
        .map(|case| (case.case_id.as_str(), case))
        .collect::<std::collections::BTreeMap<_, _>>();
    for prior in &prior_attempts {
        let case_id = prior
            .identity
            .case_id
            .as_deref()
            .context("benchmark case retry lineage omits case identity")?;
        let case = cases
            .get(case_id)
            .copied()
            .context("benchmark case retry lineage contains an undeclared case")?;
        if prior.identity.campaign_id != campaign_id
            || prior.identity.run.run_id != case.run_id
            || prior.identity.run.domain != case.domain
            || prior.identity.run.stage_id != case.stage_id
            || prior.identity.tool_id != case.tool_id
            || !prior.current_status().is_some_and(|status| {
                matches!(
                    status,
                    BenchmarkLifecycleStatus::Passed
                        | BenchmarkLifecycleStatus::Failed
                        | BenchmarkLifecycleStatus::Cancelled
                        | BenchmarkLifecycleStatus::Superseded
                        | BenchmarkLifecycleStatus::BlockedExternal
                )
            })
        {
            bail!("benchmark case retry lineage differs from its matrix or is nonterminal");
        }
    }

    let referenced = prior_attempts
        .iter()
        .filter_map(|attempt| {
            attempt.supersedes_attempt_id.as_ref().map(|prior| {
                (
                    attempt.identity.case_id.as_deref().unwrap_or_default(),
                    attempt.identity.tool_id.as_str(),
                    prior.as_str(),
                )
            })
        })
        .collect::<std::collections::BTreeSet<_>>();
    let mut tips = std::collections::BTreeMap::new();
    for attempt in &prior_attempts {
        let case_id = attempt.identity.case_id.as_deref().unwrap_or_default();
        if !referenced.contains(&(
            case_id,
            attempt.identity.tool_id.as_str(),
            attempt.identity.attempt_id.as_str(),
        )) && tips
            .insert(
                (case_id.to_string(), attempt.identity.tool_id.clone()),
                (
                    attempt.identity.attempt_id.clone(),
                    attempt.current_status().context("benchmark case retry tip is nonterminal")?,
                ),
            )
            .is_some()
        {
            bail!("benchmark case retry lineage has multiple terminal tips for one case and tool");
        }
    }
    let expected = matrix
        .cases
        .iter()
        .map(|case| (case.case_id.clone(), case.tool_id.clone()))
        .collect::<std::collections::BTreeSet<_>>();
    if tips.keys().cloned().collect::<std::collections::BTreeSet<_>>() != expected {
        bail!("benchmark case retry lineage does not close the exact case matrix");
    }

    let mut attempts = prior_attempts;
    let mut ordered_cases = matrix.cases.iter().collect::<Vec<_>>();
    ordered_cases.sort_by(|left, right| left.case_id.cmp(&right.case_id));
    let mut retry_count = 0;
    for case in ordered_cases {
        let key = (case.case_id.clone(), case.tool_id.clone());
        let (prior_attempt_id, status) =
            tips.get(&key).context("benchmark case retry tip is absent")?;
        if !terminal_tip_requires_retry(*status) {
            continue;
        }
        let mut retry = BenchmarkAttemptRecordV2::planned(
            BenchmarkAttemptIdentity {
                campaign_id: campaign_id.clone(),
                run: BenchmarkRunIdentity {
                    run_id: case.run_id.clone(),
                    domain: case.domain.clone(),
                    stage_id: case.stage_id.clone(),
                },
                case_id: Some(case.case_id.clone()),
                attempt_id: attempt_id.to_string(),
                tool_id: case.tool_id.clone(),
            },
            observed_at_utc.to_string(),
        );
        retry.supersedes_attempt_id = Some(prior_attempt_id.clone());
        attempts.push(retry);
        retry_count += 1;
    }
    if retry_count == 0 {
        bail!("benchmark case retry has no non-passing terminal tips");
    }
    let cycle = BenchmarkCycleStatusV2::new(campaign_id, observed_at_utc.to_string(), attempts);
    validate_cycle_status_v2(&cycle).context("validate planned benchmark case retry cycle")?;
    let plan = BenchmarkCampaignPlan::for_case_matrix(
        matrix.repository_commit.clone(),
        matrix.campaign_registry_sha256.clone(),
        benchmark_case_matrix_sha256(matrix)?,
        cycle,
    );
    validate_campaign_plan(&plan).context("validate source-locked benchmark case retry plan")?;
    Ok(plan)
}

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
                    case_id: None,
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

/// Derive one new planned attempt per non-passing tool while retaining terminal retry lineage.
///
/// `prior_attempts` must be a complete, terminal cycle snapshot. The unreferenced lineage tip for
/// each declared tool is retained. Passed tips remain selected evidence; every other terminal tip
/// becomes the explicit supersession target of its new attempt.
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
                        | BenchmarkLifecycleStatus::Cancelled
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
                .insert(
                    attempt.identity.tool_id.clone(),
                    (
                        attempt.identity.attempt_id.clone(),
                        attempt.current_status().context("benchmark retry tip is nonterminal")?,
                    ),
                )
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
    let mut retry_count = 0;
    for tool in tools {
        let (prior_attempt_id, status) =
            tips.get(&tool).context("benchmark retry tip is absent")?;
        if !terminal_tip_requires_retry(*status) {
            continue;
        }
        let mut retry = BenchmarkAttemptRecordV2::planned(
            BenchmarkAttemptIdentity {
                campaign_id: campaign_id.clone(),
                run: run.clone(),
                case_id: None,
                attempt_id: attempt_id.to_string(),
                tool_id: tool.clone(),
            },
            observed_at_utc.to_string(),
        );
        retry.supersedes_attempt_id = Some(prior_attempt_id.clone());
        attempts.push(retry);
        retry_count += 1;
    }
    if retry_count == 0 {
        bail!("benchmark retry has no non-passing terminal tips");
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
    if plan.case_matrix_sha256.is_some() {
        bail!("case-scoped benchmark plan requires case-matrix source verification");
    }
    if plan.repository_commit != repository_commit {
        bail!("benchmark campaign plan repository commit is stale");
    }
    if plan.campaign_registry_sha256 != campaign_registry_sha256 {
        bail!("benchmark campaign plan registry digest is stale");
    }
    Ok(())
}

/// Verify a case-scoped campaign plan against its exact generic matrix authority.
///
/// # Errors
/// Returns an error when either document is invalid or any repository, campaign-registry, or
/// matrix identity differs.
pub fn verify_case_campaign_plan_source(
    plan: &BenchmarkCampaignPlan,
    matrix: &BenchmarkCaseMatrixV2,
) -> Result<()> {
    validate_campaign_plan(plan).context("validate case-scoped benchmark campaign plan")?;
    validate_benchmark_case_matrix(matrix).context("validate benchmark case matrix authority")?;
    let matrix_sha256 = benchmark_case_matrix_sha256(matrix)?;
    if plan.repository_commit != matrix.repository_commit
        || plan.campaign_registry_sha256 != matrix.campaign_registry_sha256
        || plan.case_matrix_sha256.as_deref() != Some(matrix_sha256.as_str())
    {
        bail!("case-scoped benchmark plan source identity is stale");
    }
    Ok(())
}
