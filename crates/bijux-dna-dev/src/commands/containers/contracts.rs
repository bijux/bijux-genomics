use super::{
    failure_lines, out_path_arg, read_utf8, success_line, write_utf8, ContainerCommandOutcome,
    Result, Workspace,
};

const CATALOG_PATH: &str = "containers/contracts/catalog.toml";
const LOCK_PATH: &str = "containers/contracts/catalog.lock.json";
const QUALIFICATION_SCHEMA_PATH: &str =
    "containers/contracts/schemas/qualification-assessment.v1.json";
const QUALIFICATION_REPORT_SCHEMA_PATH: &str =
    "containers/contracts/schemas/qualification-report.v2.json";

pub(super) fn check_contract_catalog(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let validation = bijux_dna_containers::evidence::validate_container_catalog(&workspace.root)?;
    if validation.is_valid() {
        return success_line(format!(
            "container contract catalog: OK ({} tools, {} source identities require production resolution)",
            validation.tool_count, validation.unresolved_source_identities
        ));
    }
    failure_lines("container contract catalog: failed", &validation.errors)
}

fn contract_lock_content(workspace: &Workspace) -> Result<String> {
    bijux_dna_containers::evidence::render_container_catalog_lock(&workspace.root)
}

pub(super) fn generate_contract_lock(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage = "Usage: cargo run -p bijux-dna-dev -- containers run generate-contract-lock -- [<output-path>]";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let output = out_path_arg(workspace, args, LOCK_PATH, usage)?;
    write_utf8(&output, &contract_lock_content(workspace)?)?;
    success_line(format!("generated {}", output.display()))
}

pub(super) fn check_contract_lock(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let lock_path = workspace.path(LOCK_PATH);
    if !lock_path.is_file() {
        return Ok(ContainerCommandOutcome::failure(format!(
            "missing {LOCK_PATH}; generate it with the governed container command\n"
        )));
    }
    if read_utf8(&lock_path)? != contract_lock_content(workspace)? {
        return Ok(ContainerCommandOutcome::failure(format!(
            "{LOCK_PATH} drifted from {CATALOG_PATH}\n"
        )));
    }
    success_line("container contract lock: OK")
}

pub(super) fn check_evidence_qualification(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let report = qualification_report(workspace)?;
    let expected_json =
        bijux_dna_api::v1::api::qualification::render_qualification_report_json(&report)?;
    let expected_markdown =
        bijux_dna_api::v1::api::qualification::render_qualification_report_markdown(&report)?;
    let mut errors = Vec::new();
    for (path, expected) in [
        ("containers/reports/qualification.json", expected_json.as_str()),
        ("containers/reports/qualification.md", expected_markdown.as_str()),
    ] {
        if read_utf8(&workspace.path(path))? != expected {
            errors.push(format!("generated qualification report is stale: {path}"));
        }
    }
    if errors.is_empty() {
        success_line("container evidence qualification: OK")
    } else {
        failure_lines("container evidence qualification: failed", &errors)
    }
}

pub(super) fn generate_evidence_qualification(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let report = qualification_report(workspace)?;
    let json_path = workspace.path("containers/reports/qualification.json");
    let markdown_path = workspace.path("containers/reports/qualification.md");
    let json = bijux_dna_api::v1::api::qualification::render_qualification_report_json(&report)?;
    let markdown =
        bijux_dna_api::v1::api::qualification::render_qualification_report_markdown(&report)?;
    bijux_dna_infra::atomic_write_bytes(&json_path, json.as_bytes())?;
    bijux_dna_infra::atomic_write_bytes(&markdown_path, markdown.as_bytes())?;
    success_line(format!(
        "generated typed qualification for {} container tools",
        report.summary.tool_count
    ))
}

fn qualification_report(
    workspace: &Workspace,
) -> Result<bijux_dna_api::v1::api::qualification::QualificationReport> {
    let environment_evidence =
        bijux_dna_containers::evidence::load_container_evidence_chain(&workspace.root)?;
    let registry = bijux_dna_bench::load_stage_campaign_registry(
        &workspace.path("benchmarks/contracts/stage-campaigns.toml"),
    )?;
    let interfaces = bijux_dna_bench::load_stage_interface_catalog_from_repo(
        &workspace.path("benchmarks/contracts/stage-interfaces.toml"),
        &workspace.root,
    )?;
    let admitted_reports = bijux_dna_bench::load_stage_reports_from_dir(
        &workspace.path("benchmarks/reports"),
        &registry,
        &interfaces,
    )?;
    if admitted_reports.is_empty() {
        anyhow::bail!(
            "container qualification requires at least one Rust-admitted benchmark report"
        );
    }
    bijux_dna_api::v1::api::qualification::derive_container_qualification_report(
        &workspace.root,
        &environment_evidence,
        &admitted_reports,
    )
}

pub(super) fn generate_qualification_schema(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage = "Usage: cargo run -p bijux-dna-dev -- containers run generate-qualification-schema -- [<output-path>]";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let output = out_path_arg(workspace, args, QUALIFICATION_SCHEMA_PATH, usage)?;
    bijux_dna_api::v1::api::qualification::write_qualification_assessment_schema(&output)?;
    success_line(format!("generated {}", output.display()))
}

pub(super) fn check_qualification_schema(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let path = workspace.path(QUALIFICATION_SCHEMA_PATH);
    if !path.is_file() {
        return Ok(ContainerCommandOutcome::failure(format!(
            "missing {QUALIFICATION_SCHEMA_PATH}; generate it with the governed container command\n"
        )));
    }
    let checked: serde_json::Value = serde_json::from_str(&read_utf8(&path)?)?;
    let generated = bijux_dna_api::v1::api::qualification::render_qualification_assessment_schema();
    if checked != generated {
        return Ok(ContainerCommandOutcome::failure(format!(
            "{QUALIFICATION_SCHEMA_PATH} drifted from Rust policy types\n"
        )));
    }
    success_line("qualification assessment schema: OK")
}

pub(super) fn generate_qualification_report_schema(
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage = "Usage: cargo run -p bijux-dna-dev -- containers run generate-qualification-report-schema -- [<output-path>]";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let output = out_path_arg(workspace, args, QUALIFICATION_REPORT_SCHEMA_PATH, usage)?;
    bijux_dna_api::v1::api::qualification::write_qualification_report_schema(&output)?;
    success_line(format!("generated {}", output.display()))
}

pub(super) fn check_qualification_report_schema(
    workspace: &Workspace,
) -> Result<ContainerCommandOutcome> {
    let path = workspace.path(QUALIFICATION_REPORT_SCHEMA_PATH);
    if !path.is_file() {
        return Ok(ContainerCommandOutcome::failure(format!(
            "missing {QUALIFICATION_REPORT_SCHEMA_PATH}; generate it with the governed container command\n"
        )));
    }
    let checked: serde_json::Value = serde_json::from_str(&read_utf8(&path)?)?;
    let generated = bijux_dna_api::v1::api::qualification::render_qualification_report_schema();
    if checked != generated {
        return Ok(ContainerCommandOutcome::failure(format!(
            "{QUALIFICATION_REPORT_SCHEMA_PATH} drifted from Rust policy types\n"
        )));
    }
    success_line("qualification report schema: OK")
}
