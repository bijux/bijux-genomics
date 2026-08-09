use super::{
    path_from_arg, read_utf8, success_line, write_utf8, ContainerCommandOutcome, Result, Workspace,
};

const REGISTRY_SCHEMA_PATH: &str =
    "containers/contracts/schemas/qualification-run-registry.v2.json";
const AUDIT_SCHEMA_PATH: &str = "containers/contracts/schemas/qualification-run-id-audit.v2.json";
const USAGE: &str = "Usage:\n  bijux-dna-dev containers run qualification-registry -- generate <current-runs-root> <attempt-ledger> <supplemental-history> <registry-output> <audit-output>\n  bijux-dna-dev containers run qualification-registry -- check <current-runs-root> <attempt-ledger> <supplemental-history> <registry-output> <audit-output>\n  bijux-dna-dev containers run qualification-registry -- generate-schemas [<registry-schema> <audit-schema>]\n  bijux-dna-dev containers run qualification-registry -- check-schemas";

pub(super) fn run(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let Some((operation, options)) = args.split_first() else {
        anyhow::bail!(USAGE);
    };
    if matches!(operation.as_str(), "--help" | "-h") {
        return success_line(USAGE);
    }
    match operation.as_str() {
        "generate" => derive(workspace, options, false),
        "check" => derive(workspace, options, true),
        "generate-schemas" => generate_schemas(workspace, options),
        "check-schemas" => check_schemas(workspace, options),
        _ => anyhow::bail!(USAGE),
    }
}

fn derive(workspace: &Workspace, args: &[String], check: bool) -> Result<ContainerCommandOutcome> {
    let [runs_root, attempt_ledger, supplemental_history, registry_output, audit_output] = args
    else {
        anyhow::bail!(USAGE);
    };
    let derived = bijux_dna_containers::api::derive_qualification_run_registry(
        &bijux_dna_containers::api::QualificationRegistryDerivationRequest {
            repository_root: workspace.root.clone(),
            current_runs_root: path_from_arg(workspace, runs_root),
            attempt_ledger_path: path_from_arg(workspace, attempt_ledger),
            supplemental_history_path: path_from_arg(workspace, supplemental_history),
            existing_registry_path: Some(path_from_arg(workspace, registry_output)),
        },
    )?;
    let registry =
        bijux_dna_containers::api::render_qualification_run_registry(&derived.registry)?;
    let audit = serde_json::to_string_pretty(&derived.audit)? + "\n";
    let registry_output = path_from_arg(workspace, registry_output);
    let audit_output = path_from_arg(workspace, audit_output);
    if check {
        if read_utf8(&registry_output)? != registry || read_utf8(&audit_output)? != audit {
            return Ok(ContainerCommandOutcome::failure(
                "qualification-run registry or audit drifted from catalog and history\n",
            ));
        }
        return success_line(format!(
            "qualification registry: {} retained, {} unsupported, {} allocated",
            derived.audit.retained_mapping_count,
            derived.audit.unsupported_tool_count,
            derived.audit.allocated_mapping_count
        ));
    }
    write_utf8(&registry_output, &registry)?;
    write_utf8(&audit_output, &audit)?;
    success_line(format!(
        "generated {} retained qualification IDs and {} unsupported records at {}",
        derived.audit.retained_mapping_count,
        derived.audit.unsupported_tool_count,
        registry_output.display()
    ))
}

fn generate_schemas(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    let (registry, audit) = schema_paths(workspace, args)?;
    bijux_dna_containers::api::write_qualification_run_registry_schema(&registry)?;
    bijux_dna_containers::api::write_qualification_run_id_audit_schema(&audit)?;
    success_line(format!(
        "generated qualification registry schemas at {} and {}",
        registry.display(),
        audit.display()
    ))
}

fn check_schemas(workspace: &Workspace, args: &[String]) -> Result<ContainerCommandOutcome> {
    if !args.is_empty() {
        anyhow::bail!(USAGE);
    }
    let registry: serde_json::Value =
        serde_json::from_str(&read_utf8(&workspace.path(REGISTRY_SCHEMA_PATH))?)?;
    let audit: serde_json::Value =
        serde_json::from_str(&read_utf8(&workspace.path(AUDIT_SCHEMA_PATH))?)?;
    if registry != bijux_dna_containers::api::qualification_run_registry_schema()
        || audit != bijux_dna_containers::api::qualification_run_id_audit_schema()
    {
        return Ok(ContainerCommandOutcome::failure(
            "qualification registry schemas drifted from Rust types\n",
        ));
    }
    success_line("qualification registry schemas: OK")
}

fn schema_paths(
    workspace: &Workspace,
    args: &[String],
) -> Result<(std::path::PathBuf, std::path::PathBuf)> {
    match args {
        [] => Ok((workspace.path(REGISTRY_SCHEMA_PATH), workspace.path(AUDIT_SCHEMA_PATH))),
        [registry, audit] => {
            Ok((path_from_arg(workspace, registry), path_from_arg(workspace, audit)))
        }
        _ => anyhow::bail!(USAGE),
    }
}
