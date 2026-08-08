use super::{path_from_arg, read_utf8, success_line, ContainerCommandOutcome, Result, Workspace};

const PUBLICATION_SCHEMA_PATH: &str =
    "containers/contracts/schemas/runtime-verification-publication.v2.json";
const IDENTITY_SCHEMA_PATH: &str =
    "containers/contracts/schemas/tool-image-identity-evidence.v2.json";
const USAGE: &str = "Usage:\n  bijux-dna-dev containers run runtime-verification -- check-report <json-output> <markdown-output>\n  bijux-dna-dev containers run runtime-verification -- generate-schema [<publication-schema> <identity-schema>]\n  bijux-dna-dev containers run runtime-verification -- check-schema\n\nEffectful execution is owned by the bijux-dna-containers-qa runtime_verification binary.";

pub(super) fn run(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let Some((operation, options)) = args.split_first() else {
        anyhow::bail!(USAGE);
    };
    if matches!(operation.as_str(), "--help" | "-h") {
        return success_line(USAGE);
    }
    match operation.as_str() {
        "check-report" => check_report(workspace, options),
        "generate-schema" => generate_schema(workspace, options),
        "check-schema" => check_schema(workspace, options),
        _ => anyhow::bail!(USAGE),
    }
}

fn check_report(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let [json_output, markdown_output] = args else {
        anyhow::bail!(USAGE);
    };
    bijux_dna_containers::api::verify_runtime_verification_report(
        &path_from_arg(workspace, json_output),
        &path_from_arg(workspace, markdown_output),
    )?;
    success_line("runtime-verification report: OK")
}

fn generate_schema(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let (publication, identity) = match args {
        [] => (workspace.path(PUBLICATION_SCHEMA_PATH), workspace.path(IDENTITY_SCHEMA_PATH)),
        [publication, identity] => {
            (path_from_arg(workspace, publication), path_from_arg(workspace, identity))
        }
        _ => anyhow::bail!(USAGE),
    };
    bijux_dna_containers::api::write_runtime_verification_publication_schema(&publication)?;
    bijux_dna_containers::api::write_tool_image_identity_evidence_schema(&identity)?;
    success_line(format!("generated {} and {}", publication.display(), identity.display()))
}

fn check_schema(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    if !args.is_empty() {
        anyhow::bail!(USAGE);
    }
    let publication: serde_json::Value =
        serde_json::from_str(&read_utf8(&workspace.path(PUBLICATION_SCHEMA_PATH))?)?;
    let identity: serde_json::Value =
        serde_json::from_str(&read_utf8(&workspace.path(IDENTITY_SCHEMA_PATH))?)?;
    if publication != bijux_dna_containers::api::runtime_verification_publication_schema()
        || identity != bijux_dna_containers::api::tool_image_identity_evidence_schema()
    {
        return Ok(ContainerCommandOutcome::failure(
            "runtime-verification schema drifted from Rust environment types\n",
        ));
    }
    success_line("runtime-verification schema: OK")
}
