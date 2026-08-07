use super::{
    command_hostname, path_from_arg, read_utf8, success_line, ContainerCommandOutcome, Result,
    Workspace,
};

const SCHEMA_PATH: &str = "containers/contracts/config-credential.schema.json";
const GOVERNANCE_SCHEMA_PATH: &str = "containers/contracts/config-governance.schema.json";
const PIPELINE_V1_SCHEMA_PATH: &str = "containers/contracts/config-pipeline.schema.json";
const PIPELINE_V2_SCHEMA_PATH: &str = "containers/contracts/config-pipeline-v2.schema.json";
const CREDENTIAL_USAGE: &str = "Usage:\n  bijux-dna-dev containers run credential-reference -- validate <credential-reference>\n  bijux-dna-dev containers run credential-reference -- generate-schema [<output>]\n  bijux-dna-dev containers run credential-reference -- check-schema";
const PIPELINE_USAGE: &str = "Usage:\n  bijux-dna-dev containers run pipeline-contract -- validate <pipeline>\n  bijux-dna-dev containers run pipeline-contract -- admit --governance <governance> <pipeline>\n  bijux-dna-dev containers run pipeline-contract -- currentize --governance <governance> --inventory <inventory> --commit <sha> [--apply] <pipelines-root>\n  bijux-dna-dev containers run pipeline-contract -- generate-schemas [<governance-schema> <pipeline-v1-schema> <pipeline-v2-schema>]\n  bijux-dna-dev containers run pipeline-contract -- check-schemas";

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
    match args {
        [operation, path] if operation == "validate" => validate_pipeline(workspace, path),
        [operation, governance_option, governance, pipeline]
            if operation == "admit" && governance_option == "--governance" =>
        {
            admit_pipeline(workspace, governance, pipeline)
        }
        [operation, options @ ..] if operation == "currentize" => {
            currentize_pipelines(workspace, options)
        }
        [operation, options @ ..] if operation == "generate-schemas" => {
            generate_pipeline_schemas(workspace, options)
        }
        [operation] if operation == "check-schemas" => check_pipeline_schemas(workspace),
        _ => anyhow::bail!(PIPELINE_USAGE),
    }
}

fn pipeline_schema_paths(workspace: &Workspace) -> [std::path::PathBuf; 3] {
    [
        workspace.path(GOVERNANCE_SCHEMA_PATH),
        workspace.path(PIPELINE_V1_SCHEMA_PATH),
        workspace.path(PIPELINE_V2_SCHEMA_PATH),
    ]
}

fn generate_pipeline_schemas(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let paths = match args {
        [] => pipeline_schema_paths(workspace),
        [governance, pipeline_v1, pipeline_v2] => [
            path_from_arg(workspace, governance),
            path_from_arg(workspace, pipeline_v1),
            path_from_arg(workspace, pipeline_v2),
        ],
        _ => anyhow::bail!(PIPELINE_USAGE),
    };
    bijux_dna_containers::api::write_container_governance_schema(&paths[0])?;
    bijux_dna_containers::api::write_container_pipeline_v1_schema(&paths[1])?;
    bijux_dna_containers::api::write_container_pipeline_v2_schema(&paths[2])?;
    success_line(format!(
        "generated governance and pipeline schemas at {}, {}, and {}",
        paths[0].display(),
        paths[1].display(),
        paths[2].display()
    ))
}

fn check_pipeline_schemas(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let paths = pipeline_schema_paths(workspace);
    let generated = [
        bijux_dna_containers::api::container_governance_schema(),
        bijux_dna_containers::api::container_pipeline_v1_schema(),
        bijux_dna_containers::api::container_pipeline_v2_schema(),
    ];
    for (path, expected) in paths.iter().zip(generated) {
        let checked: serde_json::Value = serde_json::from_str(&read_utf8(path)?)?;
        if checked != expected {
            return Ok(ContainerCommandOutcome::failure(format!(
                "{} drifted from Rust container contract authority\n",
                path.display()
            )));
        }
    }
    success_line("container governance and pipeline schemas: OK")
}

fn validate_pipeline(workspace: &Workspace, path: &str) -> Result<ContainerCommandOutcome> {
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

fn admit_pipeline(
    workspace: &Workspace,
    governance: &str,
    pipeline: &str,
) -> Result<ContainerCommandOutcome> {
    let governance_path = path_from_arg(workspace, governance);
    let pipeline_path = path_from_arg(workspace, pipeline);
    let governance = bijux_dna_containers::api::load_container_governance(&governance_path)?;
    let admission = bijux_dna_containers::api::admit_container_pipeline_site(
        &bijux_dna_containers::api::ContainerPipelineSiteAdmissionRequest {
            governance,
            hostname: command_hostname(),
            pipeline_path: pipeline_path.clone(),
        },
    )?;
    success_line(format!(
        "admitted {} pipeline {} at {} for account {} partition {}",
        admission.schema_version,
        admission.run_id,
        pipeline_path.display(),
        admission.account,
        admission.partition
    ))
}

fn currentize_pipelines(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let mut governance = None;
    let mut inventory = None;
    let mut commit = None;
    let mut pipelines_root = None;
    let mut apply = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--apply" if !apply => {
                apply = true;
                index += 1;
            }
            option @ ("--governance" | "--inventory" | "--commit") => {
                let value = args.get(index + 1).ok_or_else(|| anyhow::anyhow!(PIPELINE_USAGE))?;
                let target = match option {
                    "--governance" => &mut governance,
                    "--inventory" => &mut inventory,
                    "--commit" => &mut commit,
                    _ => unreachable!(),
                };
                if target.replace(value.clone()).is_some() {
                    anyhow::bail!("{option} may be supplied only once");
                }
                index += 2;
            }
            value if !value.starts_with('-') && pipelines_root.is_none() => {
                pipelines_root = Some(value.to_owned());
                index += 1;
            }
            _ => anyhow::bail!(PIPELINE_USAGE),
        }
    }
    let request = bijux_dna_containers::api::ContainerPipelineCurrentizationRequest {
        governance: bijux_dna_containers::api::load_container_governance(&path_from_arg(
            workspace,
            governance.as_deref().ok_or_else(|| anyhow::anyhow!(PIPELINE_USAGE))?,
        ))?,
        hostname: command_hostname(),
        pipelines_root: path_from_arg(
            workspace,
            pipelines_root.as_deref().ok_or_else(|| anyhow::anyhow!(PIPELINE_USAGE))?,
        ),
        inventory_path: path_from_arg(
            workspace,
            inventory.as_deref().ok_or_else(|| anyhow::anyhow!(PIPELINE_USAGE))?,
        ),
        repository_commit: commit.ok_or_else(|| anyhow::anyhow!(PIPELINE_USAGE))?,
    };
    let plan = bijux_dna_containers::api::derive_container_pipeline_currentization(&request)?;
    let changed = plan.entries.iter().filter(|entry| entry.changed).count();
    let external_artifacts = plan
        .entries
        .iter()
        .filter(|entry| {
            entry.disposition
                == bijux_dna_containers::api::ContainerPipelineCurrentizationDisposition::ExternalArtifactRequired
        })
        .count();
    let published_evidence = plan
        .entries
        .iter()
        .filter(|entry| {
            entry.disposition
                == bijux_dna_containers::api::ContainerPipelineCurrentizationDisposition::PublishedEvidence
        })
        .count();
    if apply {
        bijux_dna_containers::api::apply_container_pipeline_currentization(&request, &plan)?;
    }
    success_line(format!(
        "{} {changed} changes across {} pipeline declarations ({external_artifacts} external-artifact required, {published_evidence} published evidence immutable) at inventory {}",
        if apply { "currentized" } else { "planned" },
        plan.entries.len(),
        plan.inventory_sha256
    ))
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
        assert!(run_pipeline_contract(
            &workspace,
            &["admit".into(), "--governance".into(), "site.yml".into()]
        )
        .is_err());
        assert!(run_pipeline_contract(
            &workspace,
            &["admit".into(), "--site".into(), "site.yml".into(), "pipeline.yml".into(),]
        )
        .is_err());
        assert!(run_pipeline_contract(
            &workspace,
            &["currentize".into(), "--governance".into(), "site.yml".into(), "pipelines".into(),]
        )
        .is_err());
    }
}
