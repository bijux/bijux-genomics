use super::{
    failure_lines, out_path_arg, read_utf8, run_argv, sha256_hex, success_line, write_utf8,
    ContainerCommandOutcome, Result, Serialize, Workspace,
};
use bijux_dna_environment::evidence::ContainerCatalog;

const CATALOG_PATH: &str = "containers/contracts/catalog.toml";
const LOCK_PATH: &str = "containers/contracts/catalog.lock.json";
const QUALIFICATION_SCHEMA_PATH: &str =
    "containers/contracts/schemas/qualification-assessment.v1.json";

#[derive(Serialize)]
struct ContractLock<'a> {
    schema_version: &'static str,
    source: &'static str,
    source_sha256: String,
    owner: &'a str,
    contact: &'a str,
    repository: &'a str,
    apptainer_architecture: &'a str,
    docker_architecture: &'a str,
    evidence_schema: &'a str,
    recipe_license: &'a str,
    required_checks: &'a [String],
    evidence_dimensions: &'a [String],
    tools: Vec<ContractLockTool<'a>>,
}

#[derive(Serialize)]
struct ContractLockTool<'a> {
    id: &'a str,
    display_name: &'a str,
    version: &'a str,
    support_disposition: &'a str,
    support_reason: &'a str,
    upstream: &'a str,
    license: &'a str,
    source_sha256: &'a str,
    pinned_commit: &'a str,
    apptainer_definition: &'a str,
    dockerfile: &'a str,
    contract_sha256: String,
}

fn is_zero_digest(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte == b'0')
}

fn is_placeholder_digest(value: &str) -> bool {
    is_zero_digest(value)
        || value.bytes().next().is_some_and(|first| value.bytes().all(|byte| byte == first))
}

pub(super) fn check_contract_catalog(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let validation = bijux_dna_environment::evidence::validate_container_catalog(&workspace.root)?;
    if validation.is_valid() {
        return success_line(format!(
            "container contract catalog: OK ({} tools, {} source identities require production resolution)",
            validation.tool_count, validation.unresolved_source_identities
        ));
    }
    failure_lines("container contract catalog: failed", &validation.errors)
}

fn contract_lock_content(workspace: &Workspace) -> Result<String> {
    let source = read_utf8(&workspace.path(CATALOG_PATH))?;
    let catalog: ContainerCatalog = toml::from_str(&source)?;
    let unresolved = catalog
        .tools
        .iter()
        .filter(|tool| tool.support_disposition == "retained")
        .filter(|tool| {
            (tool.source_sha256.is_empty() || is_placeholder_digest(&tool.source_sha256))
                && (tool.pinned_commit.is_empty() || is_placeholder_digest(&tool.pinned_commit))
        })
        .map(|tool| tool.id.as_str())
        .collect::<Vec<_>>();
    if !unresolved.is_empty() {
        anyhow::bail!(
            "refusing release lock with unresolved source identities: {}",
            unresolved.join(", ")
        );
    }
    let mut tools = catalog.tools.iter().collect::<Vec<_>>();
    tools.sort_by(|left, right| left.id.cmp(&right.id));
    let locked_tools = tools
        .into_iter()
        .map(|tool| {
            let serialized = serde_json::to_vec(tool)?;
            Ok(ContractLockTool {
                id: &tool.id,
                display_name: &tool.display_name,
                version: &tool.version,
                support_disposition: &tool.support_disposition,
                support_reason: &tool.support_reason,
                upstream: &tool.upstream,
                license: &tool.license,
                source_sha256: &tool.source_sha256,
                pinned_commit: &tool.pinned_commit,
                apptainer_definition: &tool.apptainer_definition,
                dockerfile: &tool.dockerfile,
                contract_sha256: sha256_hex(&serialized),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let lock = ContractLock {
        schema_version: "bijux.container.catalog_lock.v1",
        source: CATALOG_PATH,
        source_sha256: sha256_hex(source.as_bytes()),
        owner: &catalog.owner,
        contact: &catalog.contact,
        repository: &catalog.repository,
        apptainer_architecture: &catalog.apptainer_architecture,
        docker_architecture: &catalog.docker_architecture,
        evidence_schema: &catalog.evidence_schema,
        recipe_license: &catalog.recipe_license,
        required_checks: &catalog.required_checks,
        evidence_dimensions: &catalog.evidence_dimensions,
        tools: locked_tools,
    };
    Ok(format!("{}\n", serde_json::to_string_pretty(&lock)?))
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
    bijux_dna_environment::evidence::load_container_evidence_chain(&workspace.root)?;
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
        return Ok(ContainerCommandOutcome::failure(
            "container qualification requires at least one Rust-admitted benchmark report\n",
        ));
    }
    let mut args = [
        "python3",
        "-B",
        "containers/contracts/qualification_report.py",
        "--catalog",
        "containers/contracts/catalog.toml",
        "--inventory",
        "containers/reports/image-inventory.json",
        "--runtime",
        "containers/reports/runtime-verification.json",
        "--runs",
        "containers/reports/run-evidence.json",
        "--repository",
    ]
    .into_iter()
    .map(ToOwned::to_owned)
    .collect::<Vec<_>>();
    args.push(workspace.root.to_string_lossy().into_owned());
    for admission in admitted_reports {
        args.push("--benchmark-report".to_string());
        args.push(
            workspace
                .path("benchmarks/reports")
                .join(admission.report_file_name)
                .to_string_lossy()
                .into_owned(),
        );
    }
    args.extend(
        [
            "--json-output",
            "containers/reports/qualification.json",
            "--markdown-output",
            "containers/reports/qualification.md",
            "--check",
        ]
        .into_iter()
        .map(ToOwned::to_owned),
    );
    let outcome = run_argv(workspace, &args)?;
    if outcome.is_success() {
        return success_line("container evidence qualification: OK");
    }
    Ok(outcome)
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

#[cfg(test)]
mod tests {
    use super::{has_explicit_upstream_license, is_hex, is_placeholder_digest, is_zero_digest};

    #[test]
    fn digest_helpers_distinguish_resolved_and_placeholder_identities() {
        assert!(is_zero_digest(&"0".repeat(40)));
        assert!(!is_zero_digest("0123456789abcdef"));
        assert!(is_placeholder_digest(&"5".repeat(64)));
        assert!(!is_placeholder_digest(
            "220b8f1687f32f6f04cb4e85b0d6ab4ecd2e98f6f5147064c4c2420ddfdd5b3f"
        ));
        assert!(is_hex(&"a".repeat(64), 64));
        assert!(!is_hex("not-a-digest", 64));
    }

    #[test]
    fn redistribution_requires_an_explicit_upstream_license() {
        assert!(has_explicit_upstream_license("MIT"));
        assert!(has_explicit_upstream_license("GPL-3.0-only"));
        assert!(!has_explicit_upstream_license("NOASSERTION"));
        assert!(!has_explicit_upstream_license(""));
    }
}
