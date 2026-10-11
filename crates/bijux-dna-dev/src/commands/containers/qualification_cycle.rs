//! Thin operator composition for governed container qualification cycles.

use anyhow::bail;

use super::{path_from_arg, ContainerCommandOutcome, Result, Workspace};

const USAGE: &str = "Usage:\n  bijux-dna-dev containers cycle plan <registry> <image-inventory> <publication-root> <submission-root> [<external-artifact-admission-root> [<preparation>]] <observed-at-utc>\n  bijux-dna-dev containers cycle advance <registry> <image-inventory> <publication-root> <submission-root> [<external-artifact-admission-root> [<preparation>]] <observed-at-utc>\n  bijux-dna-dev containers cycle status <registry> <image-inventory> <publication-root> <submission-root> [<external-artifact-admission-root> [<preparation>]] <observed-at-utc>\n  bijux-dna-dev containers cycle collect <registry> <image-inventory> <publication-root> <submission-root> [<external-artifact-admission-root> [<preparation>]] <observed-at-utc>\n  bijux-dna-dev containers cycle admit <registry> <image-inventory> <publication-root> <submission-root> [<external-artifact-admission-root> [<preparation>]] <observed-at-utc>\n  bijux-dna-dev containers cycle book <runs-root> <json-output> <markdown-output>\n  bijux-dna-dev containers cycle replay <result-root> <empty-publication-root> <json-output> <markdown-output>";

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
        "admit" => scheduler_pass(workspace, options, SchedulerPass::Admit),
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
    Admit,
}

fn scheduler_pass(
    workspace: &Workspace,
    args: &[String],
    pass: SchedulerPass,
) -> Result<ContainerCommandOutcome> {
    let (
        registry,
        inventory,
        publication_root,
        submission_root,
        external_artifact_admission_root,
        preparation,
        observed_at_utc,
    ) = match args {
        [registry, inventory, publication_root, submission_root, observed_at_utc] => {
            (registry, inventory, publication_root, submission_root, None, None, observed_at_utc)
        }
        [registry, inventory, publication_root, submission_root, admission_root, observed_at_utc] => {
            (
                registry,
                inventory,
                publication_root,
                submission_root,
                Some(path_from_arg(workspace, admission_root)),
                None,
                observed_at_utc,
            )
        }
        [registry, inventory, publication_root, submission_root, admission_root, preparation, observed_at_utc] => {
            (
                registry,
                inventory,
                publication_root,
                submission_root,
                Some(path_from_arg(workspace, admission_root)),
                Some(bijux_dna_runner::api::load_container_qualification_cycle_preparation(
                    &path_from_arg(workspace, preparation),
                )?),
                observed_at_utc,
            )
        }
        _ => bail!(USAGE),
    };
    let request = bijux_dna_runner::api::ContainerQualificationCycleRequest {
        repository_root: workspace.root.clone(),
        registry_path: path_from_arg(workspace, registry),
        inventory_path: path_from_arg(workspace, inventory),
        publication_root: path_from_arg(workspace, publication_root),
        submission_root: path_from_arg(workspace, submission_root),
        external_artifact_admission_root,
        preparation,
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
        SchedulerPass::Admit => {
            bijux_dna_runner::api::admit_container_qualification_cycle(&request)?
        }
    };
    json_out(&status)
}

fn book(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [runs_root, json_output, markdown_output] = args else {
        bail!(USAGE);
    };
    let runs_root = path_from_arg(workspace, runs_root);
    let json_output = path_from_arg(workspace, json_output);
    let markdown_output = path_from_arg(workspace, markdown_output);
    let index = bijux_dna_containers::api::publish_run_evidence_book(
        &runs_root,
        &json_output,
        &markdown_output,
    )?;
    book_out(&runs_root, &json_output, &markdown_output, index.runs.len())
}

fn replay(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [result_root, publication_root, json_output, markdown_output] = args else {
        bail!(USAGE);
    };
    let result_root = path_from_arg(workspace, result_root);
    let publication_root = path_from_arg(workspace, publication_root);
    let json_output = path_from_arg(workspace, json_output);
    let markdown_output = path_from_arg(workspace, markdown_output);
    let index = bijux_dna_containers::api::replay_run_evidence_book(
        &result_root,
        &publication_root,
        &json_output,
        &markdown_output,
    )?;
    book_out(&publication_root, &json_output, &markdown_output, index.runs.len())
}

fn book_out(
    runs_root: &std::path::Path,
    json_output: &std::path::Path,
    markdown_output: &std::path::Path,
    selected_run_count: usize,
) -> Result<ContainerCommandOutcome> {
    json_out(&serde_json::json!({
        "runs_root": runs_root,
        "selected_run_count": selected_run_count,
        "json_output": json_output,
        "markdown_output": markdown_output,
    }))
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
        for operation in ["plan", "advance", "status", "collect", "admit", "book", "replay"] {
            assert!(output.stdout.contains(operation));
        }
        Ok(())
    }
}
