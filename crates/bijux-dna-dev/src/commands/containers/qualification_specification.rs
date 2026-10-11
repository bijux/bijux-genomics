//! Thin operator composition for generic container qualification specifications.

use std::fs;

use anyhow::{bail, Context};

use super::{path_from_arg, write_utf8, ContainerCommandOutcome, Result, Workspace};

const SCHEMA_PATH: &str = "containers/contracts/schemas/qualification-specification-source.v1.json";
const USAGE: &str = "Usage:\n  bijux-dna-dev containers run qualification-specification -- compile <source> <registry> <inventory> <pipeline-output>\n  bijux-dna-dev containers run qualification-specification -- coverage <registry> <inventory> <sources-root> <json-output>\n  bijux-dna-dev containers run qualification-specification -- generate-schema [<output>]\n  bijux-dna-dev containers run qualification-specification -- check-schema";

pub(super) fn run(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let Some((operation, options)) = args.split_first() else {
        bail!(USAGE);
    };
    match operation.as_str() {
        "compile" => compile(workspace, options),
        "coverage" => coverage(workspace, options),
        "generate-schema" => generate_schema(workspace, options),
        "check-schema" => check_schema(workspace, options),
        "--help" | "-h" => Ok(ContainerCommandOutcome::success(format!("{USAGE}\n"))),
        _ => bail!(USAGE),
    }
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
        assert!(output.stdout.contains("coverage"));
        assert!(output.stdout.contains("check-schema"));
        Ok(())
    }
}
