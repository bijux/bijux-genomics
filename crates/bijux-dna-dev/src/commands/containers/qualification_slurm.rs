use super::{
    command_hostname, path_from_arg, success_line, ContainerCommandOutcome, Result, Workspace,
};

const USAGE: &str = "Usage:\n  bijux-dna-dev containers run qualification-slurm -- prepare <governance> <pipeline> <credential-reference> <executor-program> <submitted-at-utc>\n  bijux-dna-dev containers run qualification-slurm -- submit <qualification-spec>\n  bijux-dna-dev containers run qualification-slurm -- observe <qualification-spec> <observed-at-utc>\n  bijux-dna-dev containers run qualification-slurm -- collect <qualification-spec> <collected-at-utc>\n  bijux-dna-dev containers run qualification-slurm -- cancel <qualification-spec>";

pub(super) fn run(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let Some((operation, options)) = args.split_first() else {
        anyhow::bail!(USAGE);
    };
    if matches!(operation.as_str(), "--help" | "-h") {
        return success_line(USAGE);
    }
    match operation.as_str() {
        "prepare" => prepare(workspace, options),
        "submit" => submit(workspace, options),
        "observe" => observe(workspace, options),
        "collect" => collect(workspace, options),
        "cancel" => cancel(workspace, options),
        _ => anyhow::bail!(USAGE),
    }
}

fn prepare(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [governance, pipeline, credential_reference, executor_program, submitted_at_utc] = args
    else {
        anyhow::bail!(USAGE);
    };
    let prepared = bijux_dna_runner::api::prepare_container_qualification_slurm_job(
        &bijux_dna_runner::api::ContainerQualificationPreparationRequest {
            hostname: command_hostname(),
            governance_path: path_from_arg(workspace, governance),
            pipeline_path: path_from_arg(workspace, pipeline),
            credential_reference_path: path_from_arg(workspace, credential_reference),
            executor_program_path: path_from_arg(workspace, executor_program),
        },
        submitted_at_utc,
    )?;
    let specification_path = prepared
        .request
        .script_path
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("prepared qualification script lacks an evidence root"))?
        .join("qualification-spec.json");
    json_out(&serde_json::json!({
        "specification_path": specification_path,
        "prepared_job": prepared,
    }))
}

fn submit(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [specification] = args else {
        anyhow::bail!(USAGE);
    };
    let spec = load_spec(workspace, specification)?;
    let prepared = bijux_dna_runner::api::load_prepared_container_qualification_slurm_job(&spec)?;
    let handle = bijux_dna_runner::api::submit_prepared_container_qualification_job(
        &bijux_dna_runner::api::SlurmClientPaths::default(),
        &prepared,
    )?;
    json_out(&serde_json::to_value(handle)?)
}

fn observe(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [specification, observed_at_utc] = args else {
        anyhow::bail!(USAGE);
    };
    let observation = bijux_dna_runner::api::observe_container_qualification_slurm_job(
        &bijux_dna_runner::api::SlurmClientPaths::default(),
        &load_spec(workspace, specification)?,
        observed_at_utc,
    )?;
    json_out(&serde_json::to_value(observation)?)
}

fn collect(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [specification, collected_at_utc] = args else {
        anyhow::bail!(USAGE);
    };
    let collection = bijux_dna_runner::api::collect_container_qualification_slurm_job(
        &bijux_dna_runner::api::SlurmClientPaths::default(),
        &load_spec(workspace, specification)?,
        collected_at_utc,
    )?;
    json_out(&serde_json::to_value(collection)?)
}

fn cancel(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [specification] = args else {
        anyhow::bail!(USAGE);
    };
    let spec = load_spec(workspace, specification)?;
    bijux_dna_runner::api::cancel_container_qualification_slurm_job(
        &bijux_dna_runner::api::SlurmClientPaths::default(),
        &spec,
    )?;
    success_line(format!("requested cancellation for {}:{}", spec.run_id, spec.tool_id))
}

fn load_spec(
    workspace: &Workspace,
    value: &str,
) -> Result<bijux_dna_runner::api::ContainerQualificationSlurmSpec> {
    bijux_dna_runner::api::load_container_qualification_slurm_spec(&path_from_arg(workspace, value))
}

fn json_out(value: &serde_json::Value) -> Result<ContainerCommandOutcome> {
    Ok(ContainerCommandOutcome::success(format!("{}\n", serde_json::to_string_pretty(value)?)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_lists_every_non_waiting_lifecycle_operation() -> Result<()> {
        let workspace = Workspace { root: "/workspace".into() };
        let output = run(&workspace, &["--help".to_string()])?;
        for operation in ["prepare", "submit", "observe", "collect", "cancel"] {
            assert!(output.stdout.contains(operation));
        }
        Ok(())
    }
}
