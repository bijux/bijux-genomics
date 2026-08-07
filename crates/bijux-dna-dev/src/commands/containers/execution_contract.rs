use super::{path_from_arg, read_utf8, success_line, ContainerCommandOutcome, Result, Workspace};

const SCHEMA_PATH: &str = "containers/contracts/config-credential.schema.json";
const CREDENTIAL_USAGE: &str = "Usage:\n  bijux-dna-dev containers run credential-reference -- validate <credential-reference>\n  bijux-dna-dev containers run credential-reference -- generate-schema [<output>]\n  bijux-dna-dev containers run credential-reference -- check-schema";
const PIPELINE_USAGE: &str =
    "Usage: bijux-dna-dev containers run pipeline-contract -- validate <pipeline>";

pub(super) fn run_credential_reference(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let Some((operation, options)) = args.split_first() else {
        anyhow::bail!(CREDENTIAL_USAGE);
    };
    if matches!(operation.as_str(), "--help" | "-h") {
        return success_line(CREDENTIAL_USAGE);
    }
    match operation.as_str() {
        "validate" => validate(workspace, options),
        "generate-schema" => generate_schema(workspace, options),
        "check-schema" => check_schema(workspace, options),
        _ => anyhow::bail!(CREDENTIAL_USAGE),
    }
}

pub(super) fn run_pipeline_contract(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    if args.iter().any(|argument| matches!(argument.as_str(), "--help" | "-h")) {
        return success_line(PIPELINE_USAGE);
    }
    let [operation, path] = args else {
        anyhow::bail!(PIPELINE_USAGE);
    };
    if operation != "validate" {
        anyhow::bail!(PIPELINE_USAGE);
    }
    let path = path_from_arg(workspace, path);
    let contract = bijux_dna_containers::api::load_container_pipeline(&path)?;
    let (schema, run_id, tool) = match contract {
        bijux_dna_containers::api::ContainerPipelineContract::Legacy(pipeline) => {
            (pipeline.schema_version, pipeline.run.id, pipeline.run.tool)
        }
        bijux_dna_containers::api::ContainerPipelineContract::Current(pipeline) => {
            (pipeline.schema_version, pipeline.run.id, pipeline.run.tool)
        }
    };
    success_line(format!("validated {schema} pipeline {run_id} for {tool} at {}", path.display()))
}

fn validate(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [path] = args else {
        anyhow::bail!(CREDENTIAL_USAGE);
    };
    let path = path_from_arg(workspace, path);
    let contract = bijux_dna_containers::api::load_container_credential_reference(&path)?;
    success_line(format!(
        "validated {} credential references at {}",
        contract.references.len(),
        path.display()
    ))
}

fn generate_schema(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let output = match args {
        [] => workspace.path(SCHEMA_PATH),
        [output] => path_from_arg(workspace, output),
        _ => anyhow::bail!(CREDENTIAL_USAGE),
    };
    bijux_dna_containers::api::write_container_credential_reference_schema(&output)?;
    success_line(format!("generated credential-reference schema at {}", output.display()))
}

fn check_schema(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    if !args.is_empty() {
        anyhow::bail!(CREDENTIAL_USAGE);
    }
    let checked: serde_json::Value =
        serde_json::from_str(&read_utf8(&workspace.path(SCHEMA_PATH))?)?;
    if checked != bijux_dna_containers::api::container_credential_reference_schema() {
        return Ok(ContainerCommandOutcome::failure(
            "credential-reference schema drifted from Rust container types\n",
        ));
    }
    success_line("credential-reference schema: OK")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_requires_exactly_one_contract_path() {
        let workspace = Workspace { root: "/workspace".into() };
        assert!(validate(&workspace, &[]).is_err());
        assert!(validate(&workspace, &["one".into(), "two".into()]).is_err());
    }

    #[test]
    fn pipeline_validation_requires_its_operation_and_path() {
        let workspace = Workspace { root: "/workspace".into() };
        assert!(run_pipeline_contract(&workspace, &[]).is_err());
        assert!(run_pipeline_contract(&workspace, &["validate".into()]).is_err());
    }
}
