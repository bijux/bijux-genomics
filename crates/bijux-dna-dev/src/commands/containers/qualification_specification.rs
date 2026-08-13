//! Thin operator composition for generic container qualification specifications.

use std::fs;

use anyhow::{bail, Context};

use super::{path_from_arg, write_utf8, ContainerCommandOutcome, Result, Workspace};

const SCHEMA_PATH: &str = "containers/contracts/schemas/qualification-specification-source.v1.json";
const SELECTION_SCHEMA_PATH: &str =
    "containers/contracts/schemas/qualification-source-selection.v1.json";
const USAGE: &str = "Usage:\n  bijux-dna-dev containers run qualification-specification -- materialize <selection> <registry> <inventory> <data-index> <study-root> <commit> <submitted-at-utc> <account> <partition> <walltime> <output>\n  bijux-dna-dev containers run qualification-specification -- bind-frozen-repository <source> <governance> <frozen-repository> <output>\n  bijux-dna-dev containers run qualification-specification -- compile <source> <registry> <inventory> <pipeline-output>\n  bijux-dna-dev containers run qualification-specification -- coverage <registry> <inventory> <sources-root> <json-output>\n  bijux-dna-dev containers run qualification-specification -- generate-schema [<output>]\n  bijux-dna-dev containers run qualification-specification -- generate-selection-schema [<output>]\n  bijux-dna-dev containers run qualification-specification -- check-schema\n  bijux-dna-dev containers run qualification-specification -- check-selection-schema";

pub(super) fn run(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let Some((operation, options)) = args.split_first() else {
        bail!(USAGE);
    };
    match operation.as_str() {
        "materialize" => materialize(workspace, options),
        "bind-frozen-repository" => bind_frozen_repository(workspace, options),
        "compile" => compile(workspace, options),
        "coverage" => coverage(workspace, options),
        "generate-schema" => generate_schema(workspace, options),
        "generate-selection-schema" => generate_selection_schema(workspace, options),
        "check-schema" => check_schema(workspace, options),
        "check-selection-schema" => check_selection_schema(workspace, options),
        "--help" | "-h" => Ok(ContainerCommandOutcome::success(format!("{USAGE}\n"))),
        _ => bail!(USAGE),
    }
}

fn materialize(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [selection, registry, inventory, data_index, study_root, commit, submitted_at_utc, account, partition, walltime, output] =
        args
    else {
        bail!(USAGE);
    };
    let selection = bijux_dna_runner::api::load_qualification_source_selection(&path_from_arg(
        workspace, selection,
    ))?;
    let source = bijux_dna_runner::api::materialize_qualification_specification_source(
        &bijux_dna_runner::api::QualificationSourceMaterializationRequest {
            repository_root: workspace.root.clone(),
            registry_path: path_from_arg(workspace, registry),
            inventory_path: path_from_arg(workspace, inventory),
            data_index_path: path_from_arg(workspace, data_index),
            governed_study_root: path_from_arg(workspace, study_root),
            repository_commit: commit.clone(),
            operator: "bijux-dna-dev".to_string(),
            submitted_at_utc: submitted_at_utc.clone(),
            account: account.clone(),
            partition: partition.clone(),
            walltime: walltime.clone(),
        },
        &selection,
    )?;
    let output = path_from_arg(workspace, output);
    bijux_dna_infra::atomic_create_json(&output, &source)?;
    Ok(ContainerCommandOutcome::success(format!(
        "{}\n",
        serde_json::to_string_pretty(&serde_json::json!({
            "run_id": source.run_id,
            "tool_id": source.tool_id,
            "source_output": output,
        }))?
    )))
}

fn bind_frozen_repository(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let [source, governance, frozen_repository, output] = args else {
        bail!(USAGE);
    };
    let source_path = path_from_arg(workspace, source);
    let source: bijux_dna_runner::api::QualificationSpecificationSourceV1 = serde_yaml::from_str(
        &fs::read_to_string(&source_path)
            .with_context(|| format!("read {}", source_path.display()))?,
    )
    .with_context(|| format!("parse {}", source_path.display()))?;
    let governance = bijux_dna_containers::api::load_container_governance(&path_from_arg(
        workspace, governance,
    ))?;
    let bound = bijux_dna_runner::api::bind_qualification_source_to_frozen_repository(
        &source,
        &governance,
        &path_from_arg(workspace, frozen_repository),
    )?;
    let output = path_from_arg(workspace, output);
    bijux_dna_infra::atomic_create_json(&output, &bound)?;
    Ok(ContainerCommandOutcome::success(format!(
        "{}\n",
        serde_json::to_string_pretty(&serde_json::json!({
            "run_id": bound.run_id,
            "tool_id": bound.tool_id,
            "repository_path": bound.repository_path,
            "source_output": output,
        }))?
    )))
}

fn compile(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [source, registry, inventory, output] = args else {
        bail!(USAGE);
    };
    let source_path = path_from_arg(workspace, source);
    let source: bijux_dna_runner::api::QualificationSpecificationSourceV1 = serde_yaml::from_str(
        &fs::read_to_string(&source_path)
            .with_context(|| format!("read {}", source_path.display()))?,
    )
    .with_context(|| format!("parse {}", source_path.display()))?;
    let pipeline = bijux_dna_runner::api::compile_container_qualification_specification(
        &request(workspace, registry, inventory),
        &source,
    )?;
    let output = path_from_arg(workspace, output);
    write_utf8(&output, &serde_yaml::to_string(&pipeline)?)?;
    Ok(ContainerCommandOutcome::success(format!(
        "{}\n",
        serde_json::to_string_pretty(&serde_json::json!({
            "run_id": source.run_id,
            "tool_id": source.tool_id,
            "pipeline_output": output,
        }))?
    )))
}

fn coverage(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [registry, inventory, sources_root, output] = args else {
        bail!(USAGE);
    };
    let report = bijux_dna_runner::api::derive_qualification_specification_coverage(
        &request(workspace, registry, inventory),
        &path_from_arg(workspace, sources_root),
    )?;
    let output = path_from_arg(workspace, output);
    write_utf8(&output, &(serde_json::to_string_pretty(&report)? + "\n"))?;
    Ok(ContainerCommandOutcome::success(format!(
        "{}\n",
        serde_json::to_string_pretty(&report.summary)?
    )))
}

fn generate_schema(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let output = match args {
        [] => workspace.path(SCHEMA_PATH),
        [output] => path_from_arg(workspace, output),
        _ => bail!(USAGE),
    };
    bijux_dna_runner::api::write_qualification_specification_source_schema(&output)?;
    Ok(ContainerCommandOutcome::success(format!("{}\n", output.display())))
}

fn generate_selection_schema(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let output = match args {
        [] => workspace.path(SELECTION_SCHEMA_PATH),
        [output] => path_from_arg(workspace, output),
        _ => bail!(USAGE),
    };
    bijux_dna_runner::api::write_qualification_source_selection_schema(&output)?;
    Ok(ContainerCommandOutcome::success(format!("{}\n", output.display())))
}

fn check_schema(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    if !args.is_empty() {
        bail!(USAGE);
    }
    let path = workspace.path(SCHEMA_PATH);
    let tracked: serde_json::Value = serde_json::from_slice(
        &fs::read(&path).with_context(|| format!("read {}", path.display()))?,
    )?;
    let generated =
        serde_json::to_value(bijux_dna_runner::api::qualification_specification_source_schema())?;
    if tracked != generated {
        bail!("qualification specification source schema is stale");
    }
    Ok(ContainerCommandOutcome::success("qualification specification source schema is current\n"))
}

fn check_selection_schema(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    if !args.is_empty() {
        bail!(USAGE);
    }
    let path = workspace.path(SELECTION_SCHEMA_PATH);
    let tracked: serde_json::Value = serde_json::from_slice(
        &fs::read(&path).with_context(|| format!("read {}", path.display()))?,
    )?;
    let generated =
        serde_json::to_value(bijux_dna_runner::api::qualification_source_selection_schema())?;
    if tracked != generated {
        bail!("qualification source selection schema is stale");
    }
    Ok(ContainerCommandOutcome::success("qualification source selection schema is current\n"))
}

fn request(
    workspace: &Workspace,
    registry: &str,
    inventory: &str,
) -> bijux_dna_runner::api::QualificationSpecificationCompilationRequest {
    bijux_dna_runner::api::QualificationSpecificationCompilationRequest {
        repository_root: workspace.root.clone(),
        registry_path: path_from_arg(workspace, registry),
        inventory_path: path_from_arg(workspace, inventory),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn help_exposes_compilation_and_coverage() -> Result<()> {
        let output = run(&Workspace { root: PathBuf::from("/workspace") }, &["--help".into()])?;
        assert!(output.stdout.contains("compile"));
        assert!(output.stdout.contains("materialize"));
        assert!(output.stdout.contains("coverage"));
        assert!(output.stdout.contains("bind-frozen-repository"));
        assert!(output.stdout.contains("check-schema"));
        assert!(output.stdout.contains("check-selection-schema"));
        Ok(())
    }
}
