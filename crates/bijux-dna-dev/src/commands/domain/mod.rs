use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use toml::Value as TomlValue;
use walkdir::WalkDir;

mod index;
mod inventory;
mod locking;
mod parity;
mod planning;
mod schema_policy;
mod support;
mod tool_governance;

use self::index::{check_domain_index, generate_index};
use self::inventory::{check_inventory, check_orphan_files, generate_inventory, inventory_drift};
use self::locking::{check_reference_bundle_lock, lock_registry};
use self::parity::{check_rust_stage_catalog_parity, check_ssot_authority};
use self::planning::{check_planner_fixture_coverage, check_planner_stage_coverage};
use self::schema_policy::{
    check_domain_schema, check_domain_tool_metadata, check_external_tool_policy,
    check_fixture_contracts,
};
use self::support::*;
use self::tool_governance::{check_shared_tools, check_tool_container_parity};
use crate::model::domain::{DomainCommandOutcome, NativeDomainCommandKey};
use crate::runtime::process::ProcessRunner;
use crate::runtime::workspace::Workspace;

const DOMAIN_INDEX_REGENERATE_PREFIX: &str =
    "# Regenerate with: cargo run -p bijux-dna-dev -- domain run generate-index -- ";
const REGISTRY_LOCK_GENERATED_BY: &str = "generated_by=bijux-dna-dev domain run lock-registry";

pub fn run_native_domain_command(
    key: &NativeDomainCommandKey,
    workspace: &Workspace,
    args: &[String],
) -> Result<DomainCommandOutcome> {
    match key {
        NativeDomainCommandKey::CheckDefaultSettingsDocs => {
            ensure_no_args("check-default-settings-docs", args)?;
            check_default_settings_docs(workspace)
        }
        NativeDomainCommandKey::CheckDocLinks => {
            ensure_no_args("check-doc-links", args)?;
            check_doc_links(workspace)
        }
        NativeDomainCommandKey::CheckDomainIndex => {
            ensure_no_args("check-domain-index", args)?;
            check_domain_index(workspace)
        }
        NativeDomainCommandKey::CheckDomainLayout => {
            ensure_no_args("check-domain-layout", args)?;
            check_domain_layout(workspace)
        }
        NativeDomainCommandKey::CheckDomainSchema => {
            ensure_no_args("check-domain-schema", args)?;
            check_domain_schema(workspace)
        }
        NativeDomainCommandKey::CheckDomainToolMetadata => {
            ensure_no_args("check-domain-tool-metadata", args)?;
            check_domain_tool_metadata(workspace)
        }
        NativeDomainCommandKey::CheckExternalToolPolicy => {
            ensure_no_args("check-external-tool-policy", args)?;
            check_external_tool_policy(workspace)
        }
        NativeDomainCommandKey::CheckFixtureContracts => {
            ensure_no_args("check-fixture-contracts", args)?;
            check_fixture_contracts(workspace)
        }
        NativeDomainCommandKey::CheckInventory => {
            ensure_no_args("check-inventory", args)?;
            check_inventory(workspace)
        }
        NativeDomainCommandKey::CheckOrphanFiles => {
            ensure_no_args("check-orphan-files", args)?;
            check_orphan_files(workspace)
        }
        NativeDomainCommandKey::CheckPlannerFixtureCoverage => {
            ensure_no_args("check-planner-fixture-coverage", args)?;
            check_planner_fixture_coverage(workspace)
        }
        NativeDomainCommandKey::CheckPlannerStageCoverage => {
            ensure_no_args("check-planner-stage-coverage", args)?;
            check_planner_stage_coverage(workspace)
        }
        NativeDomainCommandKey::CheckReferenceBundleLock => {
            ensure_no_args("check-reference-bundle-lock", args)?;
            check_reference_bundle_lock(workspace)
        }
        NativeDomainCommandKey::CheckRustStageCatalogParity => {
            ensure_no_args("check-rust-stage-catalog-parity", args)?;
            check_rust_stage_catalog_parity(workspace)
        }
        NativeDomainCommandKey::CheckSharedTools => {
            ensure_no_args("check-shared-tools", args)?;
            check_shared_tools(workspace)
        }
        NativeDomainCommandKey::CheckSsotAuthority => {
            ensure_no_args("check-ssot-authority", args)?;
            check_ssot_authority(workspace)
        }
        NativeDomainCommandKey::CheckToolContainerParity => {
            ensure_no_args("check-tool-container-parity", args)?;
            check_tool_container_parity(workspace)
        }
        NativeDomainCommandKey::GenerateIndex => generate_index(workspace, args),
        NativeDomainCommandKey::GenerateInventory => generate_inventory(workspace, args),
        NativeDomainCommandKey::InventoryDrift => {
            ensure_no_args("inventory-drift", args)?;
            inventory_drift(workspace)
        }
        NativeDomainCommandKey::LockRegistry => lock_registry(workspace, args),
        NativeDomainCommandKey::Validate => validate(workspace, args),
    }
}

fn command_runner(workspace: &Workspace) -> ProcessRunner<'_> {
    ProcessRunner::new(workspace)
}

fn artifact_env(workspace: &Workspace) -> Result<Vec<(String, String)>> {
    let artifact_root = workspace.path("artifacts");
    bijux_dna_infra::ensure_dir(&artifact_root)
        .with_context(|| format!("create {}", artifact_root.display()))?;
    let cargo_target_dir = artifact_root.join("target");
    let cargo_home = artifact_root.join("cargo/home");
    let tmpdir = artifact_root.join("tmp");
    bijux_dna_infra::ensure_dir(&cargo_target_dir)
        .with_context(|| format!("create {}", cargo_target_dir.display()))?;
    bijux_dna_infra::ensure_dir(&cargo_home)
        .with_context(|| format!("create {}", cargo_home.display()))?;
    bijux_dna_infra::ensure_dir(&tmpdir).with_context(|| format!("create {}", tmpdir.display()))?;
    Ok(vec![
        (
            "ARTIFACT_ROOT".to_string(),
            artifact_root.display().to_string(),
        ),
        ("ISO_ROOT".to_string(), artifact_root.display().to_string()),
        (
            "CARGO_TARGET_DIR".to_string(),
            cargo_target_dir.display().to_string(),
        ),
        ("CARGO_HOME".to_string(), cargo_home.display().to_string()),
        ("TMPDIR".to_string(), tmpdir.display().to_string()),
        ("TMP".to_string(), tmpdir.display().to_string()),
        ("TEMP".to_string(), tmpdir.display().to_string()),
        ("TZ".to_string(), "UTC".to_string()),
        ("LC_ALL".to_string(), "C".to_string()),
    ])
}

fn load_toml(path: &Path) -> Result<TomlValue> {
    toml::from_str(&read_utf8(path)?).with_context(|| format!("parse TOML {}", path.display()))
}

fn toml_tools(path: &Path) -> Result<Vec<TomlValue>> {
    Ok(load_toml(path)?
        .get("tools")
        .and_then(TomlValue::as_array)
        .cloned()
        .unwrap_or_default())
}

fn toml_stages(path: &Path) -> Result<Vec<TomlValue>> {
    Ok(load_toml(path)?
        .get("stages")
        .and_then(TomlValue::as_array)
        .cloned()
        .unwrap_or_default())
}

fn tool_registry_files(workspace: &Workspace) -> Vec<PathBuf> {
    vec![
        workspace.path("configs/ci/registry/tool_registry.toml"),
        workspace.path("configs/ci/registry/tool_registry_experimental.toml"),
        workspace.path("configs/ci/registry/tool_registry_vcf.toml"),
        workspace.path("configs/ci/registry/tool_registry_vcf_downstream.toml"),
    ]
}

fn cargo_registry_list_tools(workspace: &Workspace) -> Result<BTreeSet<String>> {
    let output = command_runner(workspace).run_owned_with_env(
        "cargo",
        &[
            "run".to_string(),
            "--quiet".to_string(),
            "--bin".to_string(),
            "bijux-dna".to_string(),
            "--".to_string(),
            "registry".to_string(),
            "list-tools".to_string(),
        ],
        &artifact_env(workspace)?,
    )?;
    if !output.status.success() {
        bail!("cargo registry list-tools failed");
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect())
}

fn cargo_registry_list_stages(workspace: &Workspace) -> Result<BTreeSet<String>> {
    let output = command_runner(workspace).run_owned_with_env(
        "cargo",
        &[
            "run".to_string(),
            "--quiet".to_string(),
            "--bin".to_string(),
            "bijux-dna".to_string(),
            "--".to_string(),
            "registry".to_string(),
            "list-stages".to_string(),
        ],
        &artifact_env(workspace)?,
    )?;
    if !output.status.success() {
        bail!("cargo registry list-stages failed");
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .split(',')
        .flat_map(|chunk| chunk.lines())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect())
}

fn cargo_registry_stage_tools(workspace: &Workspace, stage_id: &str) -> Result<BTreeSet<String>> {
    let output = command_runner(workspace).run_owned_with_env(
        "cargo",
        &[
            "run".to_string(),
            "--quiet".to_string(),
            "--bin".to_string(),
            "bijux-dna".to_string(),
            "--".to_string(),
            "registry".to_string(),
            "list-tools".to_string(),
            "--stage".to_string(),
            stage_id.to_string(),
            "--kind".to_string(),
            "all".to_string(),
        ],
        &artifact_env(workspace)?,
    )?;
    if !output.status.success() {
        return Ok(BTreeSet::new());
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect())
}

fn check_default_settings_docs(workspace: &Workspace) -> Result<DomainCommandOutcome> {
    let mut errors = Vec::new();
    let required_sections = ["inputs", "outputs", "key parameters", "validity limits"];
    let stage_re = regex(r#"(?m)^stage_id:\s*"?([^"\n#]+)"?\s*$"#)?;
    let stage_line_re = regex(r"(?m)^\s{2}([a-z0-9._-]+):\s*(.*)$")?;
    let nested_item_re = regex(r"^\s{4}-\s*([a-z0-9._-]+)\s*$")?;
    let top_level_re = regex(r"^[A-Za-z0-9_]+:\s*")?;

    for dom_dir in domain_directories(workspace)? {
        let dom = dom_dir
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow!("invalid domain directory {}", dom_dir.display()))?;
        let doc = dom_dir.join("docs/DEFAULT_SETTINGS.md");
        if !doc.is_file() {
            errors.push(format!("domain/{dom}/docs/DEFAULT_SETTINGS.md missing"));
            continue;
        }
        let text = read_utf8(&doc)?.to_lowercase();
        for section in required_sections {
            if !text.contains(section) {
                errors.push(format!(
                    "{}: missing required section phrase '{section}'",
                    workspace.rel(&doc).display()
                ));
            }
        }

        let mut stage_ids = Vec::new();
        for stage_file in yaml_files(&dom_dir.join("stages"))? {
            if stage_file.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
                continue;
            }
            let stage_text = read_utf8(&stage_file)?;
            if let Some(captures) = stage_re.captures(&stage_text) {
                if let Some(stage_id) = captures.get(1) {
                    stage_ids.push(stage_id.as_str().trim().to_string());
                }
            }
        }

        let idx = dom_dir.join("index.yaml");
        let idx_text = if idx.is_file() {
            read_utf8(&idx)?
        } else {
            String::new()
        };
        let active_default_start = idx_text
            .lines()
            .position(|line| line.starts_with("active_default_rationale:"));

        for stage in stage_ids {
            let stage_lower = stage.to_lowercase();
            if !text.contains(&stage_lower) {
                errors.push(format!(
                    "{}: missing stage coverage for '{stage}'",
                    workspace.rel(&doc).display()
                ));
            }
            if !regex(&format!(r"{}.*default", regex::escape(&stage_lower)))?.is_match(&text) {
                errors.push(format!(
                    "{}: missing blessed default description for '{stage}'",
                    workspace.rel(&doc).display()
                ));
            }
            let has_doc_rationale =
                regex(&format!(r"{}.*rationale", regex::escape(&stage_lower)))?.is_match(&text);
            let has_idx_default = regex(&format!(r"(?m)^\s{{2}}{}:\s*.+$", regex::escape(&stage)))?
                .is_match(&idx_text);

            let mut has_idx_rationale = false;
            if let Some(start_index) = active_default_start {
                for line in idx_text.lines().skip(start_index + 1) {
                    if top_level_re.is_match(line) {
                        break;
                    }
                    if regex(&format!(r"^\s{{2}}{}:\s*.+$", regex::escape(&stage)))?.is_match(line)
                    {
                        has_idx_rationale = true;
                        break;
                    }
                }
            }

            if !(has_doc_rationale || has_idx_rationale) {
                errors.push(format!(
                    "{}: missing blessed default rationale for '{stage}'",
                    workspace.rel(&doc).display()
                ));
            }
            if !(has_idx_default
                || regex(&format!(r"{}.*default", regex::escape(&stage_lower)))?.is_match(&text))
            {
                errors.push(format!(
                    "{}: missing blessed default mapping for '{stage}'",
                    workspace.rel(&doc).display()
                ));
            }
        }

        if idx.is_file() {
            let mut in_block = false;
            let mut mapping = BTreeMap::<String, Vec<String>>::new();
            let mut current_stage = None::<String>;
            for line in idx_text.lines() {
                if line.starts_with("stage_tool_compatibility:") {
                    in_block = true;
                    continue;
                }
                if in_block && top_level_re.is_match(line) {
                    break;
                }
                if !in_block {
                    continue;
                }
                if let Some(captures) = stage_line_re.captures(line) {
                    let stage = captures
                        .get(1)
                        .map(|value| value.as_str().to_string())
                        .ok_or_else(|| anyhow!("missing stage_tool_compatibility key"))?;
                    let rest = captures
                        .get(2)
                        .map(|value| value.as_str().trim().to_string())
                        .unwrap_or_default();
                    let tools = if rest.starts_with('[') && rest.ends_with(']') {
                        rest.trim_matches(|ch| ch == '[' || ch == ']')
                            .split(',')
                            .map(str::trim)
                            .filter(|value| !value.is_empty())
                            .map(|value| value.trim_matches('"').to_string())
                            .collect::<Vec<_>>()
                    } else {
                        Vec::new()
                    };
                    mapping.insert(stage.clone(), tools);
                    current_stage = Some(stage);
                    continue;
                }
                if let Some(captures) = nested_item_re.captures(line) {
                    if let (Some(stage), Some(tool)) = (current_stage.clone(), captures.get(1)) {
                        mapping
                            .entry(stage)
                            .or_default()
                            .push(tool.as_str().to_string());
                    }
                }
            }
            for (stage, tools) in mapping {
                if tools.len() == 1 {
                    let marker = format!("single_tool_justification: {stage}").to_lowercase();
                    if !text.contains(&marker) {
                        errors.push(format!(
                            "{}: missing '{marker}' for single-tool stage",
                            workspace.rel(&doc).display()
                        ));
                    }
                }
            }
        }
    }

    if errors.is_empty() {
        return success_line("default-settings docs: OK");
    }
    failure_block("default-settings docs check failed", errors)
}

fn check_doc_links(workspace: &Workspace) -> Result<DomainCommandOutcome> {
    let mut errors = Vec::new();
    let link_re = regex(r"\[[^\]]*\]\(([^)]+)\)")?;
    for dom_dir in domain_directories(workspace)? {
        let docs_dir = dom_dir.join("docs");
        if !docs_dir.is_dir() {
            continue;
        }
        for md in markdown_files(&docs_dir)? {
            let text = read_utf8(&md)?;
            for captures in link_re.captures_iter(&text) {
                let Some(target_match) = captures.get(1) else {
                    continue;
                };
                let target = target_match.as_str().trim();
                if target.is_empty()
                    || target.starts_with("http://")
                    || target.starts_with("https://")
                    || target.starts_with("mailto:")
                    || target.starts_with('#')
                {
                    continue;
                }
                let target_path = target.split('#').next().unwrap_or_default();
                let candidate = md
                    .parent()
                    .ok_or_else(|| anyhow!("missing parent for {}", md.display()))?
                    .join(target_path);
                if !candidate.exists() {
                    errors.push(format!("{} -> {target}", workspace.rel(&md).display()));
                }
            }
        }
    }
    if errors.is_empty() {
        return success_line("domain docs links: OK");
    }
    failure_block("domain docs link check failed", errors)
}

fn check_domain_layout(workspace: &Workspace) -> Result<DomainCommandOutcome> {
    let domain_dir = workspace.path("domain");
    if !domain_dir.is_dir() {
        return Ok(DomainCommandOutcome {
            exit_code: 1,
            stdout: String::new(),
            stderr: format!("domain layout: missing {}\n", domain_dir.display()),
        });
    }

    let allowed = [
        regex(
            r"^domain/[^/]+/(index\.yaml|artifacts\.yaml|metrics\.yaml|execution_support\.yaml)$",
        )?,
        regex(r"^domain/[^/]+/(stages|tools)/[^/]+\.yaml$")?,
        regex(r"^domain/[^/]+/(metrics|artifacts)/_schema\.yaml$")?,
        regex(r"^domain/[^/]+/fixtures/[^/]+(?:/[^/]+){0,2}$")?,
        regex(r"^domain/[^/]+/docs/[^/]+(?:/[^/]+)?$")?,
    ];

    let mut errors = Vec::new();
    for entry in WalkDir::new(&domain_dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let rel = workspace.rel(entry.path()).to_string_lossy().to_string();
        if rel.ends_with(".tmp") {
            errors.push(format!(
                "domain layout: forbidden *.tmp files under domain/\n{rel}"
            ));
            continue;
        }
        if allowed.iter().all(|pattern| !pattern.is_match(&rel)) {
            errors.push(format!(
                "domain layout: unknown file not in allowlist: {rel}"
            ));
        }
    }

    if errors.is_empty() {
        return success_line("domain layout: OK");
    }
    Ok(DomainCommandOutcome::failure(format!(
        "{}\n",
        errors.join("\n")
    )))
}

fn validate(workspace: &Workspace, args: &[String]) -> Result<DomainCommandOutcome> {
    let allow_non_artifacts = match args {
        [] => false,
        [single] if single == "--allow-non-artifacts" => true,
        [single] if single == "--help" || single == "-h" => {
            return Ok(DomainCommandOutcome::success(
                "Usage: cargo run -p bijux-dna-dev -- domain run validate -- [--allow-non-artifacts]\n",
            ));
        }
        _ => {
            return Ok(DomainCommandOutcome {
                exit_code: 2,
                stdout: String::new(),
                stderr: "Usage: cargo run -p bijux-dna-dev -- domain run validate -- [--allow-non-artifacts]\n".to_string(),
            });
        }
    };

    let checks = [
        check_domain_layout(workspace)?,
        check_domain_schema(workspace)?,
        check_domain_index(workspace)?,
        check_ssot_authority(workspace)?,
        check_rust_stage_catalog_parity(workspace)?,
        check_shared_tools(workspace)?,
        check_tool_container_parity(workspace)?,
        check_domain_tool_metadata(workspace)?,
        check_planner_stage_coverage(workspace)?,
        check_planner_fixture_coverage(workspace)?,
        check_default_settings_docs(workspace)?,
        check_fixture_contracts(workspace)?,
        check_orphan_files(workspace)?,
        check_doc_links(workspace)?,
        check_external_tool_policy(workspace)?,
        check_reference_bundle_lock(workspace)?,
        check_inventory(workspace)?,
    ];
    let mut stdout = String::new();
    let mut stderr = String::new();
    for outcome in checks {
        stdout.push_str(&outcome.stdout);
        stderr.push_str(&outcome.stderr);
        if !outcome.is_success() {
            return Ok(DomainCommandOutcome {
                exit_code: outcome.exit_code,
                stdout,
                stderr,
            });
        }
    }

    let env = if allow_non_artifacts {
        vec![
            ("TZ".to_string(), "UTC".to_string()),
            ("LC_ALL".to_string(), "C".to_string()),
        ]
    } else {
        artifact_env(workspace)?
    };
    let compiler = command_runner(workspace).run_owned_with_env(
        "cargo",
        &[
            "run".to_string(),
            "-p".to_string(),
            "bijux-dna-domain-compiler".to_string(),
            "--bin".to_string(),
            "domain_validate".to_string(),
            "--".to_string(),
            "--domain-dir".to_string(),
            workspace.path("domain").display().to_string(),
        ],
        &env,
    )?;
    let compiler_outcome = DomainCommandOutcome::from_output(compiler);
    stdout.push_str(&compiler_outcome.stdout);
    stderr.push_str(&compiler_outcome.stderr);
    Ok(DomainCommandOutcome {
        exit_code: compiler_outcome.exit_code,
        stdout,
        stderr,
    })
}
