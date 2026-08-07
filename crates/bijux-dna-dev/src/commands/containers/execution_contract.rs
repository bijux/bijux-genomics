use super::{path_from_arg, read_utf8, success_line, ContainerCommandOutcome, Result, Workspace};

const SCHEMA_PATH: &str = "containers/contracts/config-credential.schema.json";
const USAGE: &str = "Usage:\n  bijux-dna-dev containers run credential-reference -- validate <credential-reference>\n  bijux-dna-dev containers run credential-reference -- generate-schema [<output>]\n  bijux-dna-dev containers run credential-reference -- check-schema";

pub(super) fn run(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let Some((operation, options)) = args.split_first() else {
        anyhow::bail!(USAGE);
    };
    if matches!(operation.as_str(), "--help" | "-h") {
        return success_line(USAGE);
    }
    match operation.as_str() {
        "validate" => validate(workspace, options),
        "generate-schema" => generate_schema(workspace, options),
        "check-schema" => check_schema(workspace, options),
        _ => anyhow::bail!(USAGE),
    }
}

fn validate(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [path] = args else {
        anyhow::bail!(USAGE);
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
        _ => anyhow::bail!(USAGE),
    };
    bijux_dna_containers::api::write_container_credential_reference_schema(&output)?;
    success_line(format!("generated credential-reference schema at {}", output.display()))
}

fn check_schema(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    if !args.is_empty() {
        anyhow::bail!(USAGE);
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
}
