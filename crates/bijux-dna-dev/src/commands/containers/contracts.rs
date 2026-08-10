use super::{
    failure_lines, out_path_arg, path_from_arg, read_utf8, success_line, write_utf8,
    ContainerCommandOutcome, Result, Workspace,
};

const CATALOG_PATH: &str = "containers/contracts/catalog.toml";
const LOCK_PATH: &str = "containers/contracts/catalog.lock.json";
const QUALIFICATION_SCHEMA_PATH: &str =
    "containers/contracts/schemas/qualification-assessment.v3.json";
const QUALIFICATION_REPORT_SCHEMA_PATH: &str =
    "containers/contracts/schemas/qualification-report.v4.json";

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
    let report = qualification_report(workspace, None)?;
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
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    let usage = "Usage: cargo run -p bijux-dna-dev -- containers run generate-evidence-qualification -- [<release-publication> <output-json> <output-markdown>]";
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line(usage);
    }
    let (release_path, json_path, markdown_path) = match args {
        [] => (
            None,
            workspace.path("containers/reports/qualification.json"),
            workspace.path("containers/reports/qualification.md"),
        ),
        [release, json, markdown] => {
            let release_path = path_from_arg(workspace, release);
            let json_path = path_from_arg(workspace, json);
            let markdown_path = path_from_arg(workspace, markdown);
            if json_path == workspace.path("containers/reports/qualification.json")
                || markdown_path == workspace.path("containers/reports/qualification.md")
            {
                return Ok(ContainerCommandOutcome::failure(
                    "release-qualified output is immutable evidence and cannot overwrite the historical repository report\n",
                ));
            }
            if json_path == markdown_path
                || json_path == release_path
                || markdown_path == release_path
            {
                return Ok(ContainerCommandOutcome::failure(
                    "release publication, qualification JSON, and qualification Markdown paths must be distinct\n",
                ));
            }
            (Some(release_path), json_path, markdown_path)
        }
        _ => {
            return Ok(ContainerCommandOutcome {
                exit_code: 2,
                stdout: String::new(),
                stderr: format!("{usage}\n"),
            })
        }
    };
    let report = qualification_report(workspace, release_path.as_deref())?;
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
    release_publication_path: Option<&std::path::Path>,
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
    match release_publication_path {
        Some(path) => bijux_dna_api::v1::api::qualification::derive_container_qualification_report_with_release(
            &workspace.root,
            &environment_evidence,
            &admitted_reports,
            path,
        ),
        None => bijux_dna_api::v1::api::qualification::derive_container_qualification_report(
            &workspace.root,
            &environment_evidence,
            &admitted_reports,
        ),
    }
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

#[cfg(test)]
mod tests {
    use super::generate_evidence_qualification;
    use crate::runtime::workspace::Workspace;

    #[test]
    fn final_qualification_refuses_historical_report_overwrite() -> anyhow::Result<()> {
        let workspace = Workspace { root: std::path::PathBuf::from("/governed/repository") };
        let outcome = generate_evidence_qualification(
            &workspace,
            &[
                "/governed/STUDY/containers/release/publication.json".to_string(),
                "containers/reports/qualification.json".to_string(),
                "/governed/STUDY/containers/release/qualification.md".to_string(),
            ],
        )?;

        assert!(!outcome.is_success());
        assert!(outcome.stderr.contains("cannot overwrite the historical repository report"));
        Ok(())
    }

    #[test]
    fn final_qualification_never_overwrites_its_release_publication() -> anyhow::Result<()> {
        let workspace = Workspace { root: std::path::PathBuf::from("/governed/repository") };
        let publication = "/governed/STUDY/containers/release/publication.json";
        let outcome = generate_evidence_qualification(
            &workspace,
            &[
                publication.to_string(),
                publication.to_string(),
                "/governed/STUDY/containers/release/qualification.md".to_string(),
            ],
        )?;

        assert!(!outcome.is_success());
        assert!(outcome.stderr.contains("paths must be distinct"));
        Ok(())
    }
}
