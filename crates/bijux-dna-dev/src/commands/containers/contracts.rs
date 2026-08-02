use super::{
    failure_lines, out_path_arg, read_utf8, sha256_hex, success_line, write_utf8, BTreeSet,
    ContainerCommandOutcome, Deserialize, Result, Serialize, Workspace,
};

const CATALOG_PATH: &str = "containers/contracts/catalog.toml";
const LOCK_PATH: &str = "containers/contracts/catalog.lock.json";
const CATALOG_SCHEMA: &str = "bijux.container.catalog.v1";
const EVIDENCE_SCHEMA: &str = "bijux.container.evidence.v1";

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ContainerCatalog {
    schema_version: String,
    owner: String,
    contact: String,
    repository: String,
    apptainer_architecture: String,
    docker_architecture: String,
    evidence_schema: String,
    recipe_license: String,
    required_checks: Vec<String>,
    evidence_dimensions: Vec<String>,
    tools: Vec<ContainerContract>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ContainerContract {
    id: String,
    display_name: String,
    purpose: String,
    version: String,
    support_disposition: String,
    qualification: String,
    upstream: String,
    documentation: String,
    license: String,
    source_sha256: String,
    pinned_commit: String,
    domains: Vec<String>,
    stage_ids: Vec<String>,
    expected_bin: String,
    version_command: String,
    help_command: String,
    expected_version_regex: String,
    apptainer_definition: String,
    apptainer_supported: bool,
    dockerfile: String,
    docker_arm64_supported: bool,
    build_network: bool,
    runtime_network: bool,
    real_test_stage: String,
    real_test_input_kind: String,
}

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
    qualification: &'a str,
    upstream: &'a str,
    license: &'a str,
    source_sha256: &'a str,
    pinned_commit: &'a str,
    apptainer_definition: &'a str,
    dockerfile: &'a str,
    contract_sha256: String,
}

fn load_catalog(workspace: &Workspace) -> Result<ContainerCatalog> {
    Ok(toml::from_str(&read_utf8(&workspace.path(CATALOG_PATH))?)?)
}

fn is_zero_digest(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte == b'0')
}

fn is_placeholder_digest(value: &str) -> bool {
    is_zero_digest(value)
        || value.bytes().next().is_some_and(|first| value.bytes().all(|byte| byte == first))
}

fn is_hex(value: &str, length: usize) -> bool {
    value.len() == length && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn validate_sorted_unique(values: &[String], label: &str, tool: &str, errors: &mut Vec<String>) {
    let mut expected = values.to_vec();
    expected.sort();
    expected.dedup();
    if values != expected {
        errors.push(format!("{tool}: {label} must be sorted and unique"));
    }
}

pub(super) fn check_contract_catalog(workspace: &Workspace) -> Result<ContainerCommandOutcome> {
    let catalog = load_catalog(workspace)?;
    let mut errors = Vec::new();
    if catalog.schema_version != CATALOG_SCHEMA {
        errors.push(format!("schema_version must be {CATALOG_SCHEMA}"));
    }
    if catalog.owner.trim().is_empty() {
        errors.push("owner must not be empty".to_string());
    }
    if catalog.contact != "bijan@bijux.io" {
        errors.push("contact must be bijan@bijux.io".to_string());
    }
    if catalog.repository != "https://github.com/bijux/bijux-genomics" {
        errors.push("repository must identify bijux/bijux-genomics".to_string());
    }
    if catalog.apptainer_architecture != "linux/amd64" {
        errors.push("apptainer_architecture must be linux/amd64 for LUNARC".to_string());
    }
    if catalog.docker_architecture != "linux/arm64" {
        errors.push("docker_architecture must be linux/arm64".to_string());
    }
    if catalog.evidence_schema != EVIDENCE_SCHEMA {
        errors.push(format!("evidence_schema must be {EVIDENCE_SCHEMA}"));
    }
    if catalog.recipe_license != "GPL-3.0-only" {
        errors.push("recipe_license must identify the repository recipe license".to_string());
    }
    let required_checks = ["help", "minimal", "negative", "real_sample_slurm", "version"]
        .into_iter()
        .map(ToOwned::to_owned)
        .collect::<BTreeSet<_>>();
    if catalog.required_checks.iter().cloned().collect::<BTreeSet<_>>() != required_checks {
        errors.push(
            "required_checks must contain version, help, minimal, negative, and real_sample_slurm"
                .to_string(),
        );
    }
    let evidence_dimensions = [
        "benchmark",
        "build",
        "catalog",
        "freshness",
        "real_data",
        "release",
        "runtime",
        "science",
        "security",
        "source",
        "stage",
    ]
    .into_iter()
    .map(ToOwned::to_owned)
    .collect::<BTreeSet<_>>();
    if catalog.evidence_dimensions.iter().cloned().collect::<BTreeSet<_>>() != evidence_dimensions {
        errors.push("evidence_dimensions must name every production dimension".to_string());
    }

    let mut seen = BTreeSet::new();
    let mut catalog_apptainer = BTreeSet::new();
    let mut catalog_docker = BTreeSet::new();
    for tool in &catalog.tools {
        if !seen.insert(tool.id.clone()) {
            errors.push(format!("{}: duplicate tool id", tool.id));
        }
        if tool.id.trim().is_empty()
            || !tool
                .id
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        {
            errors.push(format!("{}: tool id must use lowercase snake case", tool.id));
        }
        if tool.display_name.trim().is_empty() || tool.purpose.trim().is_empty() {
            errors.push(format!("{}: display name and purpose must not be empty", tool.id));
        }
        if tool.version.trim().is_empty() || matches!(tool.version.as_str(), "planned" | "unknown")
        {
            errors.push(format!("{}: version must be concrete", tool.id));
        }
        if !matches!(
            tool.support_disposition.as_str(),
            "retained" | "unsupported" | "retired" | "excluded"
        ) {
            errors.push(format!(
                "{}: unsupported support disposition {}",
                tool.id, tool.support_disposition
            ));
        }
        if !matches!(tool.qualification.as_str(), "unverified" | "qualified" | "not_applicable") {
            errors.push(format!(
                "{}: unsupported evidence qualification {}",
                tool.id, tool.qualification
            ));
        }
        if !tool.upstream.starts_with("https://") {
            errors.push(format!("{}: upstream must be an https URL", tool.id));
        }
        if !tool.documentation.starts_with("https://") {
            errors.push(format!("{}: documentation must be an https URL", tool.id));
        }
        if tool.license.trim().is_empty() {
            errors.push(format!("{}: license must not be empty", tool.id));
        }
        if !tool.source_sha256.is_empty() && !is_hex(&tool.source_sha256, 64) {
            errors.push(format!("{}: source_sha256 must be 64 hexadecimal characters", tool.id));
        }
        if !tool.pinned_commit.is_empty()
            && !is_zero_digest(&tool.pinned_commit)
            && !matches!(tool.pinned_commit.len(), 7 | 40)
        {
            errors.push(format!("{}: pinned_commit must be a Git commit identity", tool.id));
        }
        validate_sorted_unique(&tool.domains, "domains", &tool.id, &mut errors);
        validate_sorted_unique(&tool.stage_ids, "stage_ids", &tool.id, &mut errors);
        if tool.expected_bin.trim().is_empty()
            || tool.version_command.trim().is_empty()
            || tool.help_command.trim().is_empty()
            || tool.expected_version_regex.trim().is_empty()
        {
            errors.push(format!("{}: runtime command contract is incomplete", tool.id));
        }
        if tool.real_test_stage.trim().is_empty() || tool.real_test_input_kind.trim().is_empty() {
            errors.push(format!("{}: real-test selection is incomplete", tool.id));
        }
        if !tool.apptainer_supported {
            errors.push(format!("{}: every retained tool must support Apptainer", tool.id));
        } else {
            let expected = format!("containers/apptainer/{}.def", tool.id);
            if tool.apptainer_definition != expected {
                errors.push(format!("{}: apptainer_definition must be {expected}", tool.id));
            }
            if !workspace.path(&tool.apptainer_definition).is_file() {
                errors.push(format!("{}: missing {}", tool.id, tool.apptainer_definition));
            }
            catalog_apptainer.insert(tool.id.clone());
        }
        if tool.docker_arm64_supported {
            let expected = format!("containers/docker/Dockerfile.arm64.{}", tool.id);
            if tool.dockerfile != expected {
                errors.push(format!("{}: dockerfile must be {expected}", tool.id));
            }
            if !workspace.path(&tool.dockerfile).is_file() {
                errors.push(format!("{}: missing {}", tool.id, tool.dockerfile));
            }
            catalog_docker.insert(tool.id.clone());
        } else if !tool.dockerfile.is_empty() {
            errors.push(format!(
                "{}: unsupported Docker runtime must not name a Dockerfile",
                tool.id
            ));
        }
        let _ = (tool.build_network, tool.runtime_network);
    }

    let catalog_order = catalog.tools.iter().map(|tool| tool.id.as_str()).collect::<Vec<_>>();
    let mut expected_order = catalog_order.clone();
    expected_order.sort_unstable();
    if catalog_order != expected_order {
        errors.push("tools must be sorted by canonical id".to_string());
    }

    let recipe_apptainer = super::apptainer_tool_ids(workspace);
    let recipe_docker = super::docker_tool_ids(workspace)?;
    for missing in recipe_apptainer.difference(&catalog_apptainer) {
        errors.push(format!("{missing}: Apptainer recipe is absent from catalog"));
    }
    for missing in catalog_apptainer.difference(&recipe_apptainer) {
        errors.push(format!("{missing}: catalog Apptainer recipe is absent from repository"));
    }
    for missing in recipe_docker.difference(&catalog_docker) {
        errors.push(format!("{missing}: Docker recipe is absent from catalog"));
    }
    for missing in catalog_docker.difference(&recipe_docker) {
        errors.push(format!("{missing}: catalog Docker recipe is absent from repository"));
    }

    if errors.is_empty() {
        let unresolved = catalog
            .tools
            .iter()
            .filter(|tool| tool.support_disposition == "retained")
            .filter(|tool| {
                (tool.source_sha256.is_empty() || is_placeholder_digest(&tool.source_sha256))
                    && (tool.pinned_commit.is_empty() || is_placeholder_digest(&tool.pinned_commit))
            })
            .count();
        return success_line(format!(
            "container contract catalog: OK ({} tools, {unresolved} source identities require production resolution)",
            catalog.tools.len()
        ));
    }
    failure_lines("container contract catalog: failed", &errors)
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
                qualification: &tool.qualification,
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

#[cfg(test)]
mod tests {
    use super::{is_hex, is_placeholder_digest, is_zero_digest};

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
}
