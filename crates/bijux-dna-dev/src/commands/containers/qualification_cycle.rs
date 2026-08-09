//! Thin operator composition for governed container qualification cycles.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use anyhow::{bail, Context};

use super::{path_from_arg, write_utf8, ContainerCommandOutcome, Result, Workspace};

const USAGE: &str = "Usage:\n  bijux-dna-dev containers cycle plan <registry> <publication-root> <submission-root> [<external-artifact-admission-root>] <observed-at-utc>\n  bijux-dna-dev containers cycle advance <registry> <publication-root> <submission-root> [<external-artifact-admission-root>] <observed-at-utc>\n  bijux-dna-dev containers cycle status <registry> <publication-root> <submission-root> [<external-artifact-admission-root>] <observed-at-utc>\n  bijux-dna-dev containers cycle collect <registry> <publication-root> <submission-root> [<external-artifact-admission-root>] <observed-at-utc>\n  bijux-dna-dev containers cycle book <runs-root> <json-output> <markdown-output>\n  bijux-dna-dev containers cycle replay <result-root> <empty-publication-root> <json-output> <markdown-output>";

pub(super) fn run(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let Some((operation, options)) = args.split_first() else {
        bail!(USAGE);
    };
    if matches!(operation.as_str(), "--help" | "-h") {
        return Ok(ContainerCommandOutcome::success(format!("{USAGE}\n")));
    }
    match operation.as_str() {
        "plan" => scheduler_pass(workspace, options, SchedulerPass::Plan),
        "advance" => scheduler_pass(workspace, options, SchedulerPass::Advance),
        "status" => scheduler_pass(workspace, options, SchedulerPass::Status),
        "collect" => scheduler_pass(workspace, options, SchedulerPass::Collect),
        "book" => book(workspace, options),
        "replay" => replay(workspace, options),
        _ => bail!(USAGE),
    }
}

#[derive(Clone, Copy)]
enum SchedulerPass {
    Plan,
    Advance,
    Status,
    Collect,
}

fn scheduler_pass(
    workspace: &Workspace,
    args: &[String],
    pass: SchedulerPass,
) -> Result<ContainerCommandOutcome> {
    let (
        registry,
        publication_root,
        submission_root,
        external_artifact_admission_root,
        observed_at_utc,
    ) = match args {
        [registry, publication_root, submission_root, observed_at_utc] => {
            (registry, publication_root, submission_root, None, observed_at_utc)
        }
        [registry, publication_root, submission_root, admission_root, observed_at_utc] => (
            registry,
            publication_root,
            submission_root,
            Some(path_from_arg(workspace, admission_root)),
            observed_at_utc,
        ),
        _ => bail!(USAGE),
    };
    let request = bijux_dna_runner::api::ContainerQualificationCycleRequest {
        repository_root: workspace.root.clone(),
        registry_path: path_from_arg(workspace, registry),
        publication_root: path_from_arg(workspace, publication_root),
        submission_root: path_from_arg(workspace, submission_root),
        external_artifact_admission_root,
        observed_at_utc: observed_at_utc.clone(),
    };
    let paths = bijux_dna_runner::api::SlurmClientPaths::default();
    let status = match pass {
        SchedulerPass::Plan => bijux_dna_runner::api::plan_container_qualification_cycle(&request)?,
        SchedulerPass::Advance => {
            bijux_dna_runner::api::advance_container_qualification_cycle(&paths, &request)?
        }
        SchedulerPass::Status => {
            bijux_dna_runner::api::status_container_qualification_cycle(&paths, &request)?
        }
        SchedulerPass::Collect => {
            bijux_dna_runner::api::collect_container_qualification_cycle(&paths, &request)?
        }
    };
    json_out(&status)
}

fn book(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [runs_root, json_output, markdown_output] = args else {
        bail!(USAGE);
    };
    publish_book(
        &path_from_arg(workspace, runs_root),
        &path_from_arg(workspace, json_output),
        &path_from_arg(workspace, markdown_output),
    )
}

fn replay(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [result_root, publication_root, json_output, markdown_output] = args else {
        bail!(USAGE);
    };
    let result_root = path_from_arg(workspace, result_root);
    let publication_root = path_from_arg(workspace, publication_root);
    require_empty_publication_root(&publication_root)?;
    fs::create_dir_all(&publication_root)
        .with_context(|| format!("create replay root {}", publication_root.display()))?;
    bijux_dna_containers::api::import_run_evidence(
        &bijux_dna_containers::api::RunEvidenceImportRequest {
            result_root,
            output_root: publication_root.clone(),
            minimum_run: None,
            maximum_run: None,
        },
    )?;
    let ledger = publication_root.join("attempt-ledger.json");
    let plan = bijux_dna_containers::api::plan_final_run_evidence(&publication_root, &ledger)?;
    bijux_dna_containers::api::apply_final_run_evidence_plan(&publication_root, &ledger, &plan)?;
    publish_book(
        &publication_root,
        &path_from_arg(workspace, json_output),
        &path_from_arg(workspace, markdown_output),
    )
}

fn publish_book(
    runs_root: &Path,
    json_output: &Path,
    markdown_output: &Path,
) -> Result<ContainerCommandOutcome> {
    let index = bijux_dna_containers::api::derive_run_evidence_index(runs_root)?;
    write_utf8(json_output, &(serde_json::to_string_pretty(&index)? + "\n"))?;
    write_utf8(markdown_output, &bijux_dna_containers::api::render_run_evidence_markdown(&index))?;
    json_out(&serde_json::json!({
        "runs_root": runs_root,
        "selected_run_count": index.runs.len(),
        "json_output": json_output,
        "markdown_output": markdown_output,
    }))
}

fn require_empty_publication_root(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            bail!("replay publication root is not a regular directory: {}", path.display())
        }
        Ok(_) => {
            if fs::read_dir(path)
                .with_context(|| format!("read replay root {}", path.display()))?
                .next()
                .is_some()
            {
                bail!("replay publication root must be empty: {}", path.display());
            }
            Ok(())
        }
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error).with_context(|| format!("inspect replay root {}", path.display())),
    }
}

fn json_out(value: &impl serde::Serialize) -> Result<ContainerCommandOutcome> {
    Ok(ContainerCommandOutcome::success(format!("{}\n", serde_json::to_string_pretty(value)?)))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn help_exposes_complete_non_waiting_surface() -> Result<()> {
        let workspace = Workspace { root: PathBuf::from("/workspace") };
        let output = run(&workspace, &["--help".to_string()])?;
        for operation in ["plan", "advance", "status", "collect", "book", "replay"] {
            assert!(output.stdout.contains(operation));
        }
        Ok(())
    }

    #[test]
    fn replay_rejects_nonempty_publication_root() -> Result<()> {
        let temporary = bijux_dna_infra::temp_dir("container-cycle-replay")?;
        fs::write(temporary.path().join("retained.json"), b"{}\n")?;
        assert!(require_empty_publication_root(temporary.path()).is_err());
        Ok(())
    }
}
