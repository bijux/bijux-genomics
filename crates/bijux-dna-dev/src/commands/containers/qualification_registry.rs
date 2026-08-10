use super::{
    path_from_arg, read_utf8, success_line, write_utf8, ContainerCommandOutcome, Result, Workspace,
};

const REGISTRY_SCHEMA_PATH: &str =
    "containers/contracts/schemas/qualification-run-registry.v3.json";
const AUDIT_SCHEMA_PATH: &str = "containers/contracts/schemas/qualification-run-id-audit.v3.json";
const RUN_REGISTRY_PATH: &str = "containers/contracts/qualification-runs.toml";
const RUN_INDEX_JSON_PATH: &str = "containers/runs/RUN_INDEX.json";
const RUN_INDEX_MARKDOWN_PATH: &str = "containers/runs/RUN_INDEX.md";
const RUN_INDEX_SCHEMA_PATH: &str = "containers/contracts/schemas/run-index.v1.json";
const REPORT_INDEX_JSON_PATH: &str = "containers/reports/REPORT_INDEX.json";
const REPORT_INDEX_MARKDOWN_PATH: &str = "containers/reports/REPORT_INDEX.md";
const REPORT_INDEX_SCHEMA_PATH: &str =
    "containers/contracts/schemas/report-portfolio-index.v1.json";
const USAGE: &str = "Usage:\n  bijux-dna-dev containers run qualification-registry -- generate <current-runs-root> <attempt-ledger> <supplemental-history> <registry-output> <audit-output>\n  bijux-dna-dev containers run qualification-registry -- check <current-runs-root> <attempt-ledger> <supplemental-history> <registry-output> <audit-output>\n  bijux-dna-dev containers run qualification-registry -- generate-index\n  bijux-dna-dev containers run qualification-registry -- check-index\n  bijux-dna-dev containers run qualification-registry -- generate-report-index\n  bijux-dna-dev containers run qualification-registry -- check-report-index\n  bijux-dna-dev containers run qualification-registry -- generate-schemas [<registry-schema> <audit-schema>]\n  bijux-dna-dev containers run qualification-registry -- check-schemas\n  bijux-dna-dev containers run qualification-registry -- generate-index-schema\n  bijux-dna-dev containers run qualification-registry -- check-index-schema\n  bijux-dna-dev containers run qualification-registry -- generate-report-index-schema\n  bijux-dna-dev containers run qualification-registry -- check-report-index-schema";

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
        "generate-index" if options.is_empty() => index(workspace, false),
        "check-index" if options.is_empty() => index(workspace, true),
        "generate-report-index" if options.is_empty() => report_index(workspace, false),
        "check-report-index" if options.is_empty() => report_index(workspace, true),
        "generate-schemas" => generate_schemas(workspace, options),
        "check-schemas" => check_schemas(workspace, options),
        "generate-index-schema" if options.is_empty() => index_schema(workspace, false),
        "check-index-schema" if options.is_empty() => index_schema(workspace, true),
        "generate-report-index-schema" if options.is_empty() => {
            report_index_schema(workspace, false)
        }
        "check-report-index-schema" if options.is_empty() => report_index_schema(workspace, true),
        _ => anyhow::bail!(USAGE),
    }
}

fn report_index(workspace: &Workspace, check: bool) -> Result<ContainerCommandOutcome> {
    let index =
        bijux_dna_containers::api::derive_container_report_portfolio_index(&workspace.root)?;
    let json = bijux_dna_containers::api::render_container_report_portfolio_index_json(&index)?;
    let markdown =
        bijux_dna_containers::api::render_container_report_portfolio_index_markdown(&index);
    let json_path = workspace.path(REPORT_INDEX_JSON_PATH);
    let markdown_path = workspace.path(REPORT_INDEX_MARKDOWN_PATH);
    if check {
        if read_utf8(&json_path)? != json || read_utf8(&markdown_path)? != markdown {
            return Ok(ContainerCommandOutcome::failure(
                "container report portfolio index drifted from report files\n",
            ));
        }
    } else {
        write_utf8(&json_path, &json)?;
        write_utf8(&markdown_path, &markdown)?;
    }
    success_line(format!(
        "container report index: {} current, {} historical or unvalidated",
        index.summary.current_count, index.summary.historical_unvalidated_count
    ))
}

fn report_index_schema(workspace: &Workspace, check: bool) -> Result<ContainerCommandOutcome> {
    let path = workspace.path(REPORT_INDEX_SCHEMA_PATH);
    let expected = serde_json::to_string_pretty(
        &bijux_dna_containers::api::container_report_portfolio_index_schema(),
    )? + "\n";
    if check {
        if read_utf8(&path)? != expected {
            return Ok(ContainerCommandOutcome::failure(
                "container report-index schema drifted from Rust model authority\n",
            ));
        }
    } else {
        write_utf8(&path, &expected)?;
    }
    success_line("container report-index schema: OK")
}

fn index(workspace: &Workspace, check: bool) -> Result<ContainerCommandOutcome> {
    let index = bijux_dna_containers::api::derive_container_run_index(
        &workspace.root,
        &workspace.path(RUN_REGISTRY_PATH),
    )?;
    let json = bijux_dna_containers::api::render_container_run_index_json(&index)?;
    let markdown = bijux_dna_containers::api::render_container_run_index_markdown(&index);
    let json_path = workspace.path(RUN_INDEX_JSON_PATH);
    let markdown_path = workspace.path(RUN_INDEX_MARKDOWN_PATH);
    if check {
        if read_utf8(&json_path)? != json || read_utf8(&markdown_path)? != markdown {
            return Ok(ContainerCommandOutcome::failure(
                "container run index drifted from the canonical qualification registry\n",
            ));
        }
    } else {
        write_utf8(&json_path, &json)?;
        write_utf8(&markdown_path, &markdown)?;
    }
    success_line(format!(
        "container run index: {} protocols through run-{}",
        index.summary.run_count,
        index.namespace.next_run_number - 1
    ))
}

fn index_schema(workspace: &Workspace, check: bool) -> Result<ContainerCommandOutcome> {
    let path = workspace.path(RUN_INDEX_SCHEMA_PATH);
    let expected =
        serde_json::to_string_pretty(&bijux_dna_containers::api::container_run_index_schema())?
            + "\n";
    if check {
        if read_utf8(&path)? != expected {
            return Ok(ContainerCommandOutcome::failure(
                "container run-index schema drifted from Rust model authority\n",
            ));
        }
    } else {
        write_utf8(&path, &expected)?;
    }
    success_line("container run-index schema: OK")
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
    let registry = bijux_dna_containers::api::render_qualification_run_registry(&derived.registry)?;
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
            "qualification registry: {} supported, {} redistributable, {} external-artifact, {} allocated",
            derived.audit.supported_tool_count,
            derived.audit.redistributable_tool_count,
            derived.audit.external_artifact_tool_count,
            derived.audit.allocated_mapping_count
        ));
    }
    write_utf8(&registry_output, &registry)?;
    write_utf8(&audit_output, &audit)?;
    success_line(format!(
        "generated {} supported qualification IDs ({} redistributable, {} external-artifact) at {}",
        derived.audit.supported_tool_count,
        derived.audit.redistributable_tool_count,
        derived.audit.external_artifact_tool_count,
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
