//! Owner: bijux-dna-bench
//! Source-locked report publication from admitted FASTQ read-length attempts.

mod evidence;
mod rendering;
mod science;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::contract::{validate_stage_report, validate_stage_report_alignment};
use bijux_dna_bench_model::{
    BenchmarkCampaignPlan, BenchmarkComparisonMetric, BenchmarkComparisonStatus,
    BenchmarkCrossToolComparison, BenchmarkReportComparability, BenchmarkReportVerdict,
    BenchmarkScientificValue, BenchmarkStageCampaignSpec, BenchmarkStageReport,
    BenchmarkStageReportDesign, BenchmarkToolReport, BENCHMARK_STAGE_REPORT_SCHEMA_V1,
};
use serde::Serialize;

use self::evidence::{load_attempt_evidence, ComparableAttemptEvidence};
pub use self::rendering::render_fastq_profile_read_lengths_markdown;
use super::super::{
    load_stage_campaign_registry, load_stage_interface_catalog_from_repo,
    verify_campaign_plan_source,
};

pub(super) const RUN_ID: &str = "run-2042";
pub(super) const STAGE_ID: &str = "fastq.profile_read_lengths";
pub(super) const BUNDLE_SCHEMA: &str = "bijux.benchmark.fastq.profile_read_lengths.bundle.v1";
pub(super) const METRICS_SCHEMA: &str = "bijux.benchmark.fastq.profile_read_lengths.metrics.v1";
pub(super) const MAX_REPORT_INPUT_BYTES: usize = 16 * 1024 * 1024;
const REPORT_FILE_STEM: &str = "run-2042-fastq-profile-read-lengths";

#[derive(Debug, Clone, PartialEq)]
pub struct FastqProfileReadLengthsStageReportPublication {
    pub report_path: PathBuf,
    pub markdown_path: PathBuf,
    pub report_sha256: String,
    pub markdown_sha256: String,
    pub report: BenchmarkStageReport,
}

/// Derive, validate, and no-clobber publish the stage report for admitted run-2042 attempts.
///
/// The report is accepted only when every planned tool attempt is passed, its admission can be
/// replayed against the complete output closure, scheduler accounting is terminal and successful,
/// and all tools share one source, input, parameter, and stage-interface identity.
///
/// # Errors
/// Returns an error when source authority, attempt closure, admitted evidence, scientific output,
/// comparability, or an existing publication is missing, malformed, stale, or conflicting.
pub fn publish_fastq_profile_read_lengths_stage_report(
    repository_root: &Path,
    study_root: &Path,
    benchmark_root: &Path,
    plan: &BenchmarkCampaignPlan,
    output_root: &Path,
) -> Result<FastqProfileReadLengthsStageReportPublication> {
    let repository_root = canonical_directory(repository_root, "benchmark repository")?;
    let study_root = canonical_directory(study_root, "STUDY root")?;
    let benchmark_root = canonical_directory(benchmark_root, "benchmark result root")?;
    if !benchmark_root.starts_with(&study_root) {
        bail!("benchmark result root is outside STUDY");
    }
    let registry_path = repository_root.join("benchmarks/contracts/stage-campaigns.toml");
    let interface_path = repository_root.join("benchmarks/contracts/stage-interfaces.toml");
    let registry_sha256 = bijux_dna_infra::hash_file_sha256(&registry_path)?;
    verify_campaign_plan_source(plan, &plan.repository_commit, &registry_sha256)?;
    let repository_commit = bijux_dna_environment::api::repository_head_commit(&repository_root)?;
    if repository_commit != plan.repository_commit {
        bail!("benchmark report source worktree differs from campaign plan");
    }
    let registry = load_stage_campaign_registry(&registry_path)?;
    let campaign = registry
        .campaigns
        .iter()
        .find(|campaign| campaign.run_id == RUN_ID)
        .context("benchmark registry omits run-2042")?;
    require_governed_campaign(campaign, plan)?;
    let interfaces = load_stage_interface_catalog_from_repo(&interface_path, &repository_root)?;
    let interface = interfaces
        .interfaces
        .iter()
        .find(|interface| interface.stage_id == STAGE_ID)
        .context("benchmark stage interface omits FASTQ read lengths")?;

    let mut evidence_by_tool = BTreeMap::new();
    for tool in &campaign.tool_ids {
        let planned = plan
            .cycle
            .attempts
            .iter()
            .filter(|attempt| is_active_plan_origin(attempt))
            .find(|attempt| attempt.identity.tool_id == *tool)
            .with_context(|| format!("campaign plan omits tool attempt {tool}"))?;
        let evidence = load_attempt_evidence(
            &repository_root,
            &study_root,
            &benchmark_root,
            campaign,
            interface,
            plan,
            &planned.identity,
        )?;
        evidence_by_tool.insert(tool.clone(), evidence);
    }
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
    let catalog_sha256 = bijux_dna_environment::api::catalog_identity_at_commit(
        &repository_root,
        &plan.repository_commit,
    )?;
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
    validate_stage_report(&report).context("validate derived FASTQ read-length report")?;
    validate_stage_report_alignment(&report, campaign, Some(interface))
        .context("align derived FASTQ read-length report")?;

    ensure_directory(output_root)?;
    let report_path = output_root.join(format!("{REPORT_FILE_STEM}.json"));
    let markdown_path = output_root.join(format!("{REPORT_FILE_STEM}.md"));
    let mut report_bytes = serde_json::to_vec_pretty(&report)?;
    report_bytes.push(b'\n');
    let markdown = render_fastq_profile_read_lengths_markdown(&report);
    ensure_bytes(&report_path, &report_bytes)?;
    ensure_bytes(&markdown_path, markdown.as_bytes())?;
    Ok(FastqProfileReadLengthsStageReportPublication {
        report_path,
        markdown_path,
        report_sha256: bijux_dna_infra::hash_bytes_sha256(&report_bytes),
        markdown_sha256: bijux_dna_infra::hash_bytes_sha256(markdown.as_bytes()),
        report,
    })
}

fn require_governed_campaign(
    campaign: &BenchmarkStageCampaignSpec,
    plan: &BenchmarkCampaignPlan,
) -> Result<()> {
    if campaign.run_id != RUN_ID
        || campaign.stage_id != STAGE_ID
        || campaign.result_artifact != "read-lengths.json"
        || campaign.input_size_keys != ["fastq_input_bytes"]
        || campaign.measured_repetitions < 3
    {
        bail!("campaign is not the governed FASTQ read-length comparison");
    }
    let planned_tools = plan
        .cycle
        .attempts
        .iter()
        .filter(|attempt| is_active_plan_origin(attempt))
        .map(|attempt| attempt.identity.tool_id.as_str())
        .collect::<BTreeSet<_>>();
    let declared_tools = campaign.tool_ids.iter().map(String::as_str).collect::<BTreeSet<_>>();
    if planned_tools != declared_tools
        || plan.cycle.attempts.iter().filter(|attempt| is_active_plan_origin(attempt)).any(
            |attempt| {
                attempt.identity.run.run_id != campaign.run_id
                    || attempt.identity.run.domain != campaign.domain
                    || attempt.identity.run.stage_id != campaign.stage_id
                    || attempt.identity.campaign_id != plan.cycle.campaign_id
            },
        )
    {
        bail!("campaign plan does not close over the declared FASTQ tools");
    }
    Ok(())
}

fn is_active_plan_origin(attempt: &bijux_dna_bench_model::BenchmarkAttemptRecordV2) -> bool {
    attempt.transitions.len() == 1
        && attempt.current_status()
            == Some(bijux_dna_bench_model::BenchmarkLifecycleStatus::Planned)
}

fn validate_comparable_attempts(
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

fn compare_tools(
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

fn canonical_directory(path: &Path, label: &str) -> Result<PathBuf> {
    let canonical = path.canonicalize().with_context(|| format!("canonicalize {label}"))?;
    let metadata = fs::symlink_metadata(&canonical).with_context(|| format!("inspect {label}"))?;
    if !metadata.is_dir() {
        bail!("{label} is not a directory");
    }
    Ok(canonical)
}

fn ensure_directory(path: &Path) -> Result<()> {
    fs::create_dir_all(path)
        .with_context(|| format!("create benchmark report directory {}", path.display()))?;
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect benchmark report directory {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!("benchmark report output is not a regular directory");
    }
    Ok(())
}

fn ensure_bytes(path: &Path, expected: &[u8]) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!("benchmark report publication is not a regular file");
            }
            let observed = bijux_dna_infra::read_to_end_bounded(path, MAX_REPORT_INPUT_BYTES)?;
            if observed != expected {
                bail!("benchmark report publication conflicts with immutable evidence");
            }
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            bijux_dna_infra::atomic_create_bytes(path, expected)
                .with_context(|| format!("publish benchmark report {}", path.display()))
        }
        Err(error) => {
            Err(error).with_context(|| format!("inspect benchmark report {}", path.display()))
        }
    }
}
