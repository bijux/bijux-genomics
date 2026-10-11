//! Benchmark campaign lifecycle planning and persistent state operations.

use std::collections::BTreeMap;

use anyhow::{bail, Context};

use super::{
    active_plan_attempts, ensure_json_identity, read_plan, require_exact_options, required,
    resolve_path, source_identity_at, OpsCommandOutcome, Result, Workspace,
};

pub(super) fn execution_schema(
    workspace: &Workspace,
    options: &BTreeMap<String, String>,
) -> Result<OpsCommandOutcome> {
    require_exact_options(options, &["--output"])?;
    let output = resolve_path(workspace, required(options, "--output")?);
    let schema = serde_json::to_value(bijux_dna_bench::benchmark_execution_spec_schema())?;
    ensure_json_identity(&output, &schema)?;
    Ok(OpsCommandOutcome::success(format!(
        "published benchmark execution specification schema at {}\n",
        output.display()
    )))
}

pub(super) fn scheduler_observation_schema(
    workspace: &Workspace,
    options: &BTreeMap<String, String>,
) -> Result<OpsCommandOutcome> {
    require_exact_options(options, &["--output"])?;
    let output = resolve_path(workspace, required(options, "--output")?);
    let schema = serde_json::to_value(bijux_dna_bench::benchmark_scheduler_observation_schema())?;
    ensure_json_identity(&output, &schema)?;
    Ok(OpsCommandOutcome::success(format!(
        "published benchmark scheduler observation schema at {}\n",
        output.display()
    )))
}

pub(super) fn schema(
    workspace: &Workspace,
    options: &BTreeMap<String, String>,
) -> Result<OpsCommandOutcome> {
    require_exact_options(options, &["--output"])?;
    let output = resolve_path(workspace, required(options, "--output")?);
    let schema = serde_json::to_value(bijux_dna_bench::benchmark_campaign_plan_schema())?;
    ensure_json_identity(&output, &schema)?;
    Ok(OpsCommandOutcome::success(format!(
        "published benchmark campaign plan schema at {}\n",
        output.display()
    )))
}

pub(super) fn plan(
    workspace: &Workspace,
    options: &BTreeMap<String, String>,
) -> Result<OpsCommandOutcome> {
    require_exact_options(
        options,
        &[
            "--attempt-id",
            "--campaign-id",
            "--observed-at-utc",
            "--output",
            "--repository-root",
            "--run-id",
        ],
    )?;
    let repository_root = resolve_path(workspace, required(options, "--repository-root")?)
        .canonicalize()
        .context("canonicalize benchmark source repository")?;
    let (repository_commit, campaign_registry_sha256) = source_identity_at(&repository_root)?;
    let registry = bijux_dna_bench::load_stage_campaign_registry(
        &repository_root.join("benchmarks/contracts/stage-campaigns.toml"),
    )?;
    let cycle = bijux_dna_bench::plan_stage_campaign_attempts(
        &registry,
        &repository_commit,
        &campaign_registry_sha256,
        required(options, "--run-id")?,
        required(options, "--campaign-id")?,
        required(options, "--attempt-id")?,
        required(options, "--observed-at-utc")?,
    )?;
    let output = resolve_path(workspace, required(options, "--output")?);
    ensure_json_identity(&output, &cycle)?;
    Ok(OpsCommandOutcome::success(format!(
        "planned {} benchmark attempts at {}\n",
        cycle.cycle.attempts.len(),
        output.display()
    )))
}

pub(super) fn plan_retry(
    workspace: &Workspace,
    options: &BTreeMap<String, String>,
) -> Result<OpsCommandOutcome> {
    require_exact_options(
        options,
        &[
            "--attempt-id",
            "--benchmark-root",
            "--observed-at-utc",
            "--output",
            "--prior-plan",
            "--repository-root",
        ],
    )?;
    let repository_root = resolve_path(workspace, required(options, "--repository-root")?)
        .canonicalize()
        .context("canonicalize benchmark retry repository")?;
    let (repository_commit, campaign_registry_sha256) = source_identity_at(&repository_root)?;
    let prior_plan_path = resolve_path(workspace, required(options, "--prior-plan")?);
    let prior_plan = read_plan(&prior_plan_path)?;
    bijux_dna_bench::verify_campaign_plan_source(
        &prior_plan,
        &prior_plan.repository_commit,
        &prior_plan.campaign_registry_sha256,
    )?;
    require_retry_source_ancestry(&repository_root, &prior_plan.repository_commit)?;
    let benchmark_root = resolve_path(workspace, required(options, "--benchmark-root")?)
        .canonicalize()
        .context("canonicalize benchmark retry result root")?;
    let prior_attempts = prior_plan
        .cycle
        .attempts
        .iter()
        .map(|attempt| bijux_dna_bench::load_attempt_lifecycle(&benchmark_root, &attempt.identity))
        .collect::<Result<Vec<_>>>()?;
    let registry = bijux_dna_bench::load_stage_campaign_registry(
        &repository_root.join("benchmarks/contracts/stage-campaigns.toml"),
    )?;
    let retry = bijux_dna_bench::plan_stage_campaign_retry_attempts(
        &registry,
        &repository_commit,
        &campaign_registry_sha256,
        prior_attempts,
        required(options, "--attempt-id")?,
        required(options, "--observed-at-utc")?,
    )?;
    let output = resolve_path(workspace, required(options, "--output")?);
    ensure_json_identity(&output, &retry)?;
    Ok(OpsCommandOutcome::success(format!(
        "planned {} active benchmark retries with {} retained lineage records at {}\n",
        active_plan_attempts(&retry).len(),
        retry.cycle.attempts.len() - active_plan_attempts(&retry).len(),
        output.display()
    )))
}

fn require_retry_source_ancestry(
    repository_root: &std::path::Path,
    prior_commit: &str,
) -> Result<()> {
    if !bijux_dna_environment::api::repository_commit_is_ancestor_of_head(
        repository_root,
        prior_commit,
    )? {
        bail!("benchmark retry prior source is not an ancestor of the current source");
    }
    Ok(())
}

pub(super) fn supersede(
    workspace: &Workspace,
    options: &BTreeMap<String, String>,
) -> Result<OpsCommandOutcome> {
    require_exact_options(
        options,
        &["--benchmark-root", "--observed-at-utc", "--plan", "--reason", "--repository-root"],
    )?;
    let repository_root = resolve_path(workspace, required(options, "--repository-root")?)
        .canonicalize()
        .context("canonicalize benchmark supersession repository")?;
    let (repository_commit, campaign_registry_sha256) = source_identity_at(&repository_root)?;
    let plan_path = resolve_path(workspace, required(options, "--plan")?);
    let plan = read_plan(&plan_path)?;
    bijux_dna_bench::verify_campaign_plan_source(
        &plan,
        &repository_commit,
        &campaign_registry_sha256,
    )?;
    let benchmark_root = resolve_path(workspace, required(options, "--benchmark-root")?)
        .canonicalize()
        .context("canonicalize benchmark supersession root")?;
    let attempts = active_plan_attempts(&plan);
    if attempts.is_empty() {
        bail!("benchmark supersession plan has no active attempt origins");
    }
    let reason = required(options, "--reason")?;
    for attempt in &attempts {
        let persisted =
            bijux_dna_bench::load_attempt_lifecycle(&benchmark_root, &attempt.identity)?;
        let is_active = matches!(
            persisted.current_status(),
            Some(
                bijux_dna_bench::BenchmarkLifecycleStatus::Planned
                    | bijux_dna_bench::BenchmarkLifecycleStatus::Ready
                    | bijux_dna_bench::BenchmarkLifecycleStatus::Submitted
                    | bijux_dna_bench::BenchmarkLifecycleStatus::Running
                    | bijux_dna_bench::BenchmarkLifecycleStatus::Collecting
            )
        );
        let is_identical_replay = persisted.current_status()
            == Some(bijux_dna_bench::BenchmarkLifecycleStatus::Superseded)
            && persisted.transitions.last().and_then(|transition| transition.reason.as_deref())
                == Some(reason);
        if !is_active && !is_identical_replay {
            bail!("benchmark supersession requires active persisted attempts");
        }
    }
    for attempt in &attempts {
        bijux_dna_runtime::record_attempt_superseded(
            &benchmark_root,
            &attempt.identity,
            required(options, "--observed-at-utc")?,
            reason,
        )?;
    }
    Ok(OpsCommandOutcome::success(format!(
        "ensured {} benchmark attempts are superseded with the retained reason\n",
        attempts.len()
    )))
}

pub(super) fn initialize(
    workspace: &Workspace,
    options: &BTreeMap<String, String>,
) -> Result<OpsCommandOutcome> {
    require_exact_options(options, &["--benchmark-root", "--plan", "--repository-root"])?;
    let plan_path = resolve_path(workspace, required(options, "--plan")?);
    let cycle = read_plan(&plan_path)?;
    let repository_root = resolve_path(workspace, required(options, "--repository-root")?)
        .canonicalize()
        .context("canonicalize benchmark source repository")?;
    let (repository_commit, campaign_registry_sha256) = source_identity_at(&repository_root)?;
    bijux_dna_bench::verify_campaign_plan_source(
        &cycle,
        &repository_commit,
        &campaign_registry_sha256,
    )?;
    let benchmark_root = resolve_path(workspace, required(options, "--benchmark-root")?);
    let directories = bijux_dna_bench::ensure_stage_campaign_attempts(&benchmark_root, &cycle)?;
    Ok(OpsCommandOutcome::success(format!(
        "initialized {} benchmark attempts under {}\n",
        directories.len(),
        benchmark_root.display()
    )))
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::process::Command;

    use super::require_retry_source_ancestry;

    #[test]
    fn retry_source_accepts_descendants_and_rejects_unrelated_history() -> anyhow::Result<()> {
        let temporary = bijux_dna_infra::temp_dir("benchmark-retry-source")?;
        let root = temporary.path();
        git(root, &["init", "--quiet"])?;
        git(root, &["config", "user.email", "tests@bijux.invalid"])?;
        git(root, &["config", "user.name", "Bijux Tests"])?;
        std::fs::write(root.join("source.rs"), b"const SOURCE: u8 = 1;\n")?;
        git(root, &["add", "source.rs"])?;
        git(root, &["commit", "--quiet", "-m", "initial source"])?;
        let prior = bijux_dna_environment::api::repository_head_commit(root)?;

        std::fs::write(root.join("source.rs"), b"const SOURCE: u8 = 2;\n")?;
        git(root, &["add", "source.rs"])?;
        git(root, &["commit", "--quiet", "-m", "correct source"])?;
        require_retry_source_ancestry(root, &prior)?;

        git(root, &["checkout", "--quiet", "--orphan", "unrelated"])?;
        std::fs::write(root.join("source.rs"), b"const SOURCE: u8 = 3;\n")?;
        git(root, &["add", "source.rs"])?;
        git(root, &["commit", "--quiet", "-m", "unrelated source"])?;
        assert!(require_retry_source_ancestry(root, &prior).is_err());
        Ok(())
    }

    fn git(root: &Path, arguments: &[&str]) -> anyhow::Result<()> {
        let output = Command::new("git").arg("-C").arg(root).args(arguments).output()?;
        anyhow::ensure!(
            output.status.success(),
            "git command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(())
    }
}
