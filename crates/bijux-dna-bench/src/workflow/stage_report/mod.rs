//! Owner: bijux-dna-bench
//! Shared authority for source-locked benchmark stage report derivation.

mod evidence;
mod freshness;
mod metrics;
mod publication;
mod rendering;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::contract::{validate_stage_report, validate_stage_report_alignment};
use bijux_dna_bench_model::{
    BenchmarkCampaignPlan, BenchmarkComparisonMetric, BenchmarkComparisonStatus,
    BenchmarkCrossToolComparison, BenchmarkReportComparability, BenchmarkReportVerdict,
    BenchmarkScientificValue, BenchmarkStageCampaignSpec, BenchmarkStageInterfaceSpec,
    BenchmarkStageReport, BenchmarkStageReportDesign, BenchmarkToolReport,
    BENCHMARK_STAGE_REPORT_SCHEMA_V1,
};
use serde::Serialize;

use super::BenchmarkReportPresentationAuthority;

pub(super) use evidence::load_admitted_attempt_evidence;
pub(super) use freshness::{current_report_sources, report_source_is_current};
pub(super) use metrics::{distribution, read_resource_observation, ResourceObservation};
pub(super) use publication::{
    canonical_directory, ensure_bytes, ensure_directory, MAX_REPORT_INPUT_BYTES,
};
pub use rendering::render_stage_report_markdown;

#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkStageReportPublication {
    pub report_path: PathBuf,
    pub markdown_path: PathBuf,
    pub report_sha256: String,
    pub markdown_sha256: String,
    pub report: BenchmarkStageReport,
    pub presentation_authority: BenchmarkReportPresentationAuthority,
}

/// Publish one validated report using only its stage-interface presentation contract.
///
/// # Errors
/// Returns an error when report alignment, presentation rendering, output safety, or immutable
/// no-clobber publication fails.
pub fn publish_stage_report(
    output_root: &Path,
    campaign: &BenchmarkStageCampaignSpec,
    interface: &BenchmarkStageInterfaceSpec,
    report: BenchmarkStageReport,
    presentation_authority: &BenchmarkReportPresentationAuthority,
) -> Result<BenchmarkStageReportPublication> {
    validate_stage_report(&report).context("validate benchmark stage report before publication")?;
    validate_stage_report_alignment(&report, campaign, Some(interface))
        .context("align benchmark stage report before publication")?;
    let markdown = render_stage_report_markdown(&report, &interface.report_presentation)?;
    let mut report_bytes = serde_json::to_vec_pretty(&report)?;
    report_bytes.push(b'\n');
    ensure_directory(output_root)?;
    let stem = &interface.report_presentation.report_file_stem;
    let report_path = output_root.join(format!("{stem}.json"));
    let markdown_path = output_root.join(format!("{stem}.md"));
    ensure_bytes(&report_path, &report_bytes)?;
    ensure_bytes(&markdown_path, markdown.as_bytes())?;
    Ok(BenchmarkStageReportPublication {
        report_path,
        markdown_path,
        report_sha256: bijux_dna_infra::hash_bytes_sha256(&report_bytes),
        markdown_sha256: bijux_dna_infra::hash_bytes_sha256(markdown.as_bytes()),
        report,
        presentation_authority: presentation_authority.clone(),
    })
}

#[derive(Debug, Clone)]
pub(super) struct ComparableAttemptEvidence {
    pub(super) tool_report: BenchmarkToolReport,
    pub(super) input_lock_sha256: String,
    pub(super) parameter_lock_sha256: String,
    pub(super) interface: bijux_dna_bench_model::BenchmarkReportStageInterface,
}

pub(super) fn build_stage_report(
    repository_root: &Path,
    plan: &BenchmarkCampaignPlan,
    campaign: &BenchmarkStageCampaignSpec,
    interface: &BenchmarkStageInterfaceSpec,
    evidence_by_tool: BTreeMap<String, ComparableAttemptEvidence>,
) -> Result<BenchmarkStageReport> {
    build_stage_report_for_commit(
        repository_root,
        &plan.repository_commit,
        campaign,
        interface,
        evidence_by_tool,
    )
}

pub(super) fn build_stage_report_for_commit(
    repository_root: &Path,
    repository_commit: &str,
    campaign: &BenchmarkStageCampaignSpec,
    interface: &BenchmarkStageInterfaceSpec,
    mut evidence_by_tool: BTreeMap<String, ComparableAttemptEvidence>,
) -> Result<BenchmarkStageReport> {
    let comparable = validate_comparable_attempts(campaign, &evidence_by_tool)?;
    let tools = campaign
        .tool_ids
        .iter()
        .map(|tool| {
            evidence_by_tool
                .remove(tool)
                .map(|evidence| evidence.tool_report)
                .with_context(|| format!("derived report omits tool {tool}"))
        })
        .collect::<Result<Vec<_>>>()?;
    let comparison = compare_tools(campaign, &tools)?;
    let verdict = if comparison.status == BenchmarkComparisonStatus::OutsideTolerance {
        BenchmarkReportVerdict::Failed
    } else {
        BenchmarkReportVerdict::Passed
    };
    let catalog_sha256 =
        bijux_dna_containers::api::catalog_identity_at_commit(repository_root, repository_commit)?;
    let report = BenchmarkStageReport {
        schema_version: BENCHMARK_STAGE_REPORT_SCHEMA_V1.to_string(),
        verdict,
        run_id: campaign.run_id.clone(),
        stage_id: campaign.stage_id.clone(),
        catalog_sha256,
        evidence_tier: campaign.evidence_tier,
        benchmark_profile_id: campaign.profile_id.clone(),
        scientific_question: campaign.scientific_question.clone(),
        comparison_scope: campaign.comparison_scope.clone(),
        accuracy_claim: campaign.accuracy_claim.clone(),
        design: BenchmarkStageReportDesign {
            runtime: campaign.runtime,
            scheduler: campaign.scheduler,
            container_only: campaign.container_only,
            threads_by_tool: campaign_threads(campaign)?,
            cache_policy: campaign.cache_policy,
            warmup_repetitions: campaign.warmup_repetitions,
            measured_repetitions: campaign.measured_repetitions,
            tool_closure: campaign.tool_ids.clone(),
            corpus_ids: campaign.corpus_ids.clone(),
            corpus_catalog_ids: campaign.corpus_catalog_ids.clone(),
            reference_catalog_ids: campaign.reference_catalog_ids.clone(),
            database_catalog_ids: campaign.database_catalog_ids.clone(),
            input_size_keys: campaign.input_size_keys.clone(),
            result_artifact: campaign.result_artifact.clone(),
        },
        tools,
        comparability: Some(comparable),
        cross_tool_comparison: comparison,
        limitations: campaign.limitations.clone(),
        interpretation: campaign.interpretation.clone(),
    };
    validate_stage_report(&report).context("validate derived benchmark stage report")?;
    validate_stage_report_alignment(&report, campaign, Some(interface))
        .context("align derived benchmark stage report")?;
    Ok(report)
}

pub(super) fn require_complete_plan(
    campaign: &BenchmarkStageCampaignSpec,
    plan: &BenchmarkCampaignPlan,
) -> Result<()> {
    let selected_attempts = plan
        .cycle
        .attempts
        .iter()
        .filter(|attempt| is_selected_plan_attempt(attempt))
        .collect::<Vec<_>>();
    let selected_tools = selected_attempts
        .iter()
        .map(|attempt| attempt.identity.tool_id.as_str())
        .collect::<BTreeSet<_>>();
    let declared_tools = campaign.tool_ids.iter().map(String::as_str).collect::<BTreeSet<_>>();
    if selected_attempts.len() != declared_tools.len()
        || selected_tools != declared_tools
        || selected_attempts.iter().any(|attempt| {
            attempt.identity.run.run_id != campaign.run_id
                || attempt.identity.run.domain != campaign.domain
                || attempt.identity.run.stage_id != campaign.stage_id
                || attempt.identity.campaign_id != plan.cycle.campaign_id
        })
    {
        bail!("campaign plan does not close over the declared tools");
    }
    Ok(())
}

/// Return whether a source-locked plan record selects evidence for report publication.
///
/// Initial plans select their planned attempts. Selective retry plans retain passed lineage tips
/// and add planned replacements only for non-passing tips, so both states form the report closure.
pub(super) fn is_selected_plan_attempt(
    attempt: &bijux_dna_bench_model::BenchmarkAttemptRecordV2,
) -> bool {
    is_report_selected_status(attempt.current_status())
}

fn is_report_selected_status(
    status: Option<bijux_dna_bench_model::BenchmarkLifecycleStatus>,
) -> bool {
    matches!(
        status,
        Some(
            bijux_dna_bench_model::BenchmarkLifecycleStatus::Planned
                | bijux_dna_bench_model::BenchmarkLifecycleStatus::Passed
        )
    )
}

pub(super) fn validate_comparable_attempts(
    campaign: &BenchmarkStageCampaignSpec,
    evidence: &BTreeMap<String, ComparableAttemptEvidence>,
) -> Result<BenchmarkReportComparability> {
    if evidence.len() != campaign.tool_ids.len() {
        bail!("benchmark report lacks complete tool evidence");
    }
    let commits = evidence
        .values()
        .map(|item| item.tool_report.repository_commit.as_str())
        .collect::<BTreeSet<_>>();
    let input_locks =
        evidence.values().map(|item| item.input_lock_sha256.as_str()).collect::<BTreeSet<_>>();
    let parameter_locks =
        evidence.values().map(|item| item.parameter_lock_sha256.as_str()).collect::<BTreeSet<_>>();
    let interfaces = evidence
        .values()
        .map(|item| serde_json::to_vec(&item.interface))
        .collect::<std::result::Result<BTreeSet<_>, _>>()?;
    if commits.len() != 1
        || input_locks.len() != 1
        || parameter_locks.len() != 1
        || interfaces.len() != 1
    {
        bail!("benchmark tool attempts do not share one comparison closure");
    }
    let first = evidence.values().next().context("benchmark evidence is empty")?;
    Ok(BenchmarkReportComparability {
        repository_commit: first.tool_report.repository_commit.clone(),
        input_lock_sha256: first.input_lock_sha256.clone(),
        parameter_lock_sha256: first.parameter_lock_sha256.clone(),
        stage_interface: Some(first.interface.clone()),
    })
}

pub(super) fn compare_tools(
    campaign: &BenchmarkStageCampaignSpec,
    tools: &[BenchmarkToolReport],
) -> Result<BenchmarkCrossToolComparison> {
    let mut metrics = Vec::new();
    for tolerance in &campaign.comparison_metrics {
        let mut observations = BTreeMap::new();
        for tool in tools {
            let values = tool.scientific_observations.get(&tolerance.id).with_context(|| {
                format!("tool {} omits comparison metric {}", tool.tool, tolerance.id)
            })?;
            let numeric = values
                .iter()
                .map(|value| match value {
                    BenchmarkScientificValue::Number(value) => Ok(*value),
                    BenchmarkScientificValue::Text(_) => {
                        bail!("comparison metric {} is not numeric", tolerance.id)
                    }
                })
                .collect::<Result<Vec<_>>>()?;
            if numeric.is_empty() {
                bail!("comparison metric {} has no observations", tolerance.id);
            }
            observations
                .insert(tool.tool.clone(), numeric.iter().sum::<f64>() / numeric.len() as f64);
        }
        let minimum = observations.values().copied().fold(f64::INFINITY, f64::min);
        let maximum = observations.values().copied().fold(f64::NEG_INFINITY, f64::max);
        let absolute_range = maximum - minimum;
        let scale = observations.values().map(|value| value.abs()).fold(0.0, f64::max);
        let allowed_range = tolerance.absolute_tolerance + tolerance.relative_tolerance * scale;
        metrics.push(BenchmarkComparisonMetric {
            metric: tolerance.id.clone(),
            observations,
            minimum,
            maximum,
            absolute_range,
            allowed_range,
            within_tolerance: absolute_range <= allowed_range,
            absolute_tolerance: tolerance.absolute_tolerance,
            relative_tolerance: tolerance.relative_tolerance,
        });
    }
    let status = if metrics.is_empty() {
        BenchmarkComparisonStatus::NotApplicable
    } else if metrics.iter().all(|metric| metric.within_tolerance) {
        BenchmarkComparisonStatus::WithinTolerance
    } else {
        BenchmarkComparisonStatus::OutsideTolerance
    };
    Ok(BenchmarkCrossToolComparison { status, metrics })
}

pub(super) fn campaign_threads(
    campaign: &BenchmarkStageCampaignSpec,
) -> Result<BTreeMap<String, u32>> {
    if !campaign.threads_by_tool.is_empty() {
        return Ok(campaign.threads_by_tool.clone());
    }
    let threads = campaign.threads.context("benchmark campaign omits thread allocation")?;
    Ok(campaign.tool_ids.iter().map(|tool| (tool.clone(), threads)).collect())
}

pub(super) fn hash_json(value: &impl Serialize) -> Result<String> {
    Ok(bijux_dna_infra::hash_bytes_sha256(&serde_json::to_vec(value)?))
}

pub(super) fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes = bijux_dna_infra::read_to_end_bounded(path, MAX_REPORT_INPUT_BYTES)
        .with_context(|| format!("read benchmark report input {}", path.display()))?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("parse benchmark report input {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::is_report_selected_status;
    use bijux_dna_bench_model::BenchmarkLifecycleStatus;

    #[test]
    fn report_selection_retains_passed_tips_and_planned_retries() {
        assert!(is_report_selected_status(Some(BenchmarkLifecycleStatus::Passed)));
        assert!(is_report_selected_status(Some(BenchmarkLifecycleStatus::Planned)));
        for status in [
            BenchmarkLifecycleStatus::Ready,
            BenchmarkLifecycleStatus::Submitted,
            BenchmarkLifecycleStatus::Running,
            BenchmarkLifecycleStatus::Collecting,
            BenchmarkLifecycleStatus::Failed,
            BenchmarkLifecycleStatus::Cancelled,
            BenchmarkLifecycleStatus::Superseded,
            BenchmarkLifecycleStatus::BlockedExternal,
        ] {
            assert!(!is_report_selected_status(Some(status)));
        }
        assert!(!is_report_selected_status(None));
    }
}
