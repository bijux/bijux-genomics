use std::collections::BTreeSet;

use anyhow::{Context, Result};
use regex::Regex;
use sha2::{Digest, Sha256};

use crate::commands::command_support::{fail, pass, read, run_command};
use crate::model::check::{CheckDefinition, CheckOutcome};
use crate::runtime::workspace::Workspace;

mod artifacts;
mod governance;
mod workspace_contracts;

pub(crate) use artifacts::{
    check_artifact_env_contract, check_artifacts_layout, check_artifacts_tracked,
    check_assets_reference_schema, check_no_fake_artifacts, check_output_roots,
};
pub(crate) use governance::{
    check_audit_allowlist, check_bench_knob_discipline_downstream, check_bench_knobs,
    check_benchmark_integrity_policy, check_certification_schema_docs,
    check_clippy_allowlist_expiry, check_clippy_allowlist_growth, check_deny_policy_deviations,
};
pub(crate) use workspace_contracts::{
    check_cargo_config_policy, check_config_schema, check_docs_build_contract,
    check_docs_requirements_lock, check_examples_runner_contract,
    check_frontend_mini_domain_validation, check_generated_configs, check_gitignore_contract,
    check_hidden_tmp_usage, check_hpc_rsync_docs_parity, check_hpc_safety, check_logging_contract,
    check_make_help_sync, check_no_target_paths_in_tests, check_no_user_path_literals,
    check_readme_links, check_root_layout, check_runtime_execution_kernel_config,
    check_rustflags_consistency,
};

pub(crate) fn check_ssot_guardrails(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let output = run_command(
        workspace,
        "git",
        &["show", "--name-only", "--pretty=", "HEAD"],
    )?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let changed = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>();
    let registry_changed = changed.contains(&"configs/ci/registry/tool_registry.toml");
    let lock_changed = changed.contains(&"configs/ci/registry/tool_registry_lock.sha256");
    if registry_changed && !lock_changed {
        return fail(
            check,
            "partial registry edit detected without tool_registry_lock.sha256",
        );
    }
    let stages_changed = changed
        .iter()
        .any(|path| path.starts_with("configs/ci/stages/") && path.ends_with(".toml"));
    let params_changed = changed.iter().any(|path| {
        path.starts_with("configs/ci/params/param_registry") && path.ends_with(".toml")
    });
    if stages_changed && !params_changed {
        return fail(
            check,
            "partial stage edit detected without param registry update",
        );
    }
    pass(check, "last commit preserves SSOT guardrails")
}

pub(crate) fn check_species_aliases(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let aliases_cfg: toml::Value = toml::from_str(&read(
        &workspace.path("configs/runtime/species_aliases.toml"),
    )?)?;
    let species_cfg: toml::Value =
        toml::from_str(&read(&workspace.path("configs/runtime/species.toml"))?)?;
    let aliases = aliases_cfg
        .get("aliases")
        .and_then(toml::Value::as_table)
        .cloned()
        .unwrap_or_default();
    let default_builds = aliases_cfg
        .get("default_builds")
        .and_then(toml::Value::as_table)
        .cloned()
        .unwrap_or_default();
    let species_rows = species_cfg
        .get("species")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let canonical = Regex::new(r"^[A-Z][a-z]+ [a-z]+$").expect("regex");
    let mut authority_default_build = std::collections::BTreeMap::new();
    let mut authority_species = BTreeSet::new();
    let mut errors = Vec::new();
    for row in species_rows {
        let species_id = row
            .get("species_id")
            .and_then(toml::Value::as_str)
            .unwrap_or("");
        let build_id = row
            .get("default_build_id")
            .and_then(toml::Value::as_str)
            .unwrap_or("");
        if species_id.is_empty() || build_id.is_empty() {
            errors.push("species.toml row missing species_id/default_build_id".to_string());
            continue;
        }
        authority_default_build.insert(species_id.to_string(), build_id.to_string());
        authority_species.insert(species_id.to_string());
    }
    for (alias, species) in aliases {
        let alias = alias.clone();
        let species = species.as_str().unwrap_or("").to_string();
        if alias != alias.to_lowercase() {
            errors.push(format!("alias `{alias}` must be lowercase"));
        }
        if !canonical.is_match(&species) {
            errors.push(format!(
                "alias `{alias}` has non-canonical species id `{species}`"
            ));
        }
        if !authority_species.contains(&species) {
            errors.push(format!(
                "alias `{alias}` points to undeclared species `{species}`"
            ));
        }
    }
    for (species, build) in default_builds {
        let species = species.clone();
        let build = build.as_str().unwrap_or("");
        match authority_default_build.get(&species) {
            Some(expected) if expected == build => {}
            Some(expected) => errors.push(format!(
                "default_builds mismatch for `{species}`: aliases=`{build}`, species.toml=`{expected}`"
            )),
            None => errors.push(format!(
                "default_builds species `{species}` missing in species.toml authority"
            )),
        }
    }
    if errors.is_empty() {
        return pass(check, "species aliases stay canonical and authority-backed");
    }
    fail(check, errors.join("\n"))
}

pub(crate) fn check_tool_registry_lock(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let inputs = [
        "configs/ci/registry/tool_registry.toml",
        "configs/ci/registry/tool_registry_experimental.toml",
        "configs/ci/registry/tool_registry_vcf.toml",
        "configs/ci/registry/tool_registry_vcf_downstream.toml",
        "configs/ci/registry/domains.toml",
        "configs/ci/registry/deprecations.toml",
    ];
    let mut payload = String::new();
    for rel in inputs {
        let bytes = std::fs::read(workspace.path(rel))
            .with_context(|| format!("read {}", workspace.path(rel).display()))?;
        let file_sha = format!("{:x}", Sha256::digest(&bytes));
        payload.push_str(rel);
        payload.push(' ');
        payload.push_str(&file_sha);
        payload.push('\n');
    }
    let expected = format!("{:x}", Sha256::digest(payload.as_bytes()));
    let actual = read(&workspace.path("configs/ci/registry/tool_registry_lock.sha256"))?
        .trim()
        .to_string();
    if expected != actual {
        return fail(
            check,
            "tool registry lock hash does not match registry inputs",
        );
    }
    let marker = workspace.path("artifacts/configs/tool_registry_lock.marker");
    if !marker.is_file() {
        return fail(
            check,
            format!("missing {}", workspace.rel(&marker).display()),
        );
    }
    let marker_text = read(&marker)?;
    if !marker_text.contains("generated_by=bijux-dna-dev domain run lock-registry")
        || !marker_text.contains(&format!("lock_sha256={actual}"))
    {
        return fail(check, "tool registry lock marker is stale or invalid");
    }
    pass(check, "tool registry lock and marker stay synchronized")
}

pub(crate) fn check_vcf_compatibility_matrix(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let panels: toml::Value =
        toml::from_str(&read(&workspace.path("configs/vcf/panels/panels.toml"))?)?;
    let registry: toml::Value = toml::from_str(&read(
        &workspace.path("configs/ci/registry/tool_registry_vcf_downstream.toml"),
    )?)?;
    let panel_rows = panels
        .get("panel")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let tool_rows = registry
        .get("tools")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut rows = Vec::new();
    for panel in panel_rows {
        let species = panel
            .get("species_id")
            .and_then(toml::Value::as_str)
            .unwrap_or("");
        let build = panel
            .get("build_id")
            .and_then(toml::Value::as_str)
            .unwrap_or("");
        let panel_id = panel.get("id").and_then(toml::Value::as_str).unwrap_or("");
        let tags = panel
            .get("compatibility")
            .and_then(toml::Value::as_table)
            .and_then(|table| table.get("tool_tags"))
            .and_then(toml::Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|value| value.as_str().map(ToOwned::to_owned))
            .collect::<BTreeSet<_>>();
        for tool in &tool_rows {
            let tool_id = tool.get("id").and_then(toml::Value::as_str).unwrap_or("");
            if !tags.contains(tool_id) {
                continue;
            }
            let stages = tool
                .get("stage_ids")
                .and_then(toml::Value::as_array)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter_map(|value| value.as_str().map(ToOwned::to_owned))
                .collect::<Vec<_>>()
                .join(", ");
            rows.push(format!(
                "| {species} | {build} | {panel_id} | {tool_id} | {stages} |"
            ));
        }
    }
    rows.sort();

    let doc = read(&workspace.path("docs/50-reference/VCF_DOWNSTREAM_COMPATIBILITY_MATRIX.md"))?;
    let present_rows = doc
        .lines()
        .filter(|line| line.starts_with("| ") && !line.starts_with("|---"))
        .skip(1)
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if present_rows == rows {
        return pass(
            check,
            "VCF downstream compatibility matrix matches SSOT inputs",
        );
    }
    fail(check, "VCF downstream compatibility matrix is stale")
}
