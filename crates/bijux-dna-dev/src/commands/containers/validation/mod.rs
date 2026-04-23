use super::runtime::{
    artifact_env, artifact_root_path, bijux_command_prefix,
    check_apptainer_frontend_reproducibility, check_apptainer_frontend_security,
    check_apptainer_frontend_smoke_proof, check_apptainer_hardening, check_apptainer_post_pins,
    check_apptainer_version_label_sync, check_missing_images, check_owners, check_registry_vs_defs,
    checked_container_type, compare_frontend_local_sif_hash, container_artifact_dir,
    ensure_no_args, env_or_default, env_or_empty, generate_local_apptainer_digests,
    list_tools_for_stage, merge_outcomes, primary_tools_csv, require_tools_or_stage,
    resolved_smoke_tools, run_argv, run_bijux_with_env, run_environment_prep_for,
    run_environment_prep_for_with_env, run_environment_smoke_for,
    run_environment_smoke_for_with_env, run_program_with_env, run_runtime_smoke_contract,
    sampled_apptainer_defs, write_ensure_images_plan_report, write_frontend_repro_summary,
    write_frontend_security_summary, write_vuln_hook_report,
};
use super::{
    anyhow, append_named_outcome, apptainer_def_paths, apptainer_tool_ids,
    canonical_container_label_keys, canonical_metadata_labels, docker_image_labels,
    docker_tool_ids, dockerfile_paths, failure_lines, fs, images_metadata, iso_root_path,
    load_toml, lock_items_by_tool, metadata, missing_container_label_markers, path_from_arg,
    read_json, read_utf8, registry_tool_rows, sha256_hex, success_line, table_array_strings,
    table_bool, table_string, tool_status_manifest, tool_versions, toolkit_bundles, validation,
    versioning, write_utf8, BTreeMap, BTreeSet, ContainerCommandOutcome, Context, Path,
    PathBuf, ProcessRunner, Regex, Result, Utc, WalkDir, Workspace,
};

mod compliance;
mod operations;

pub(super) use self::compliance::{
    check_bijux_template_markers, check_docker_arch_policy, check_docker_arm64_completeness,
    check_docker_context, check_docker_hardening, check_docker_labels, check_docker_unpinned_apt,
    check_docker_version_sync, check_dockerfiles_built, check_hpc_image_naming, check_no_secrets,
    check_planned_actionability, check_runtime_downloads, check_sbom_artifacts,
    check_smoke_inputs_policy, check_time_locale_determinism, check_tool_container_coverage,
    check_tool_id_contract, check_tool_invocation_normalization, check_tool_name_collision,
    check_toolkit_bundles, check_vuln_allowlist, check_vuln_hook,
};
pub(super) use self::operations::{
    check_apptainer_bijux_header, check_apptainer_rebuild_repro, check_build_provenance,
    check_cross_runtime_representative, check_cross_runtime_smoke,
    check_cross_runtime_smoke_at_paths, check_digest_changes_on_version_change,
    check_digest_output_policy, check_hpc_frontend_policy_enforcement, check_image_size_regression,
    check_imputation_cross_runtime_parity, check_imputation_hardening,
    check_imputation_network_policy, check_imputation_release_smoke,
    check_imputation_runtime_constraints, check_lock_matches_built_output, check_rebuild_repro,
    check_release_checklist, check_runtime_tool_digest_recording, check_smoke_contract,
    check_smoke_contract_lock, check_smoke_failure_classification, check_toolkit_bundle_buildable,
    check_vcf_downstream_bundle_coverage, check_vcf_imputation_toolchain, current_host_name,
    run_apptainer_build_all, run_apptainer_ensure, run_apptainer_ensure_stage,
    run_apptainer_frontend_reproducibility, run_apptainer_frontend_security,
    run_apptainer_frontend_smoke, run_build_apptainer_all, run_build_apptainer_hpc_frontend,
    run_build_contract, run_container_doctor, run_container_lint, run_container_smoke,
    run_containers_smoke, run_docker_build_all, run_ensure_images, run_env_prep, run_env_smoke,
    run_image_qa, run_image_smoke_vcf, run_registry_tools, run_release_gate, run_test_images,
    run_test_images_stage, run_test_images_tool, run_vuln_scan_hook, summary,
};

pub(super) fn load_runtime_manifest_rows(
    path: &std::path::Path,
) -> Result<BTreeMap<String, serde_json::Value>> {
    let mut rows = BTreeMap::new();
    for entry in fs::read_dir(path)
        .with_context(|| format!("read {}", path.display()))?
        .filter_map(std::result::Result::ok)
    {
        let manifest_path = entry.path();
        if manifest_path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let name = manifest_path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        if matches!(
            name,
            "summary.json"
                | "report.json"
                | "lock.json"
                | "security_summary.json"
                | "sbom_index.json"
        ) {
            continue;
        }
        let Ok(row) = read_json(&manifest_path) else {
            continue;
        };
        let tool = row
            .get("tool")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        if !tool.is_empty() {
            rows.insert(tool, row);
        }
    }
    Ok(rows)
}

pub(super) fn normalized_version_output(row: &serde_json::Value) -> String {
    row.get("normalized_version_output")
        .and_then(serde_json::Value::as_str)
        .or_else(|| {
            row.get("version_output")
                .and_then(serde_json::Value::as_str)
        })
        .unwrap_or_default()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

pub(super) fn registry_tool_id(row: &toml::map::Map<String, toml::Value>) -> String {
    let id = table_string(row, "id");
    if id.is_empty() {
        table_string(row, "tool_id")
    } else {
        id
    }
}
