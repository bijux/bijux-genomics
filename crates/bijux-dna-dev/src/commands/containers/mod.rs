use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use chrono::{Local, NaiveDate, Utc};
use regex::Regex;
use serde::Serialize;
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use crate::model::container::{ContainerCommandOutcome, NativeContainerCommandKey};
use crate::runtime::process::ProcessRunner;
use crate::runtime::workspace::Workspace;

mod command_support;
mod content_support;
mod dispatch;
mod metadata;
mod registry_catalog;
mod runtime;
mod validation;
mod version_state;
mod versioning;

use self::command_support::{
    append_named_outcome, failure_lines, git_is_shallow_repository, git_last_modified_timestamp,
    iso_root_path, iso_run_id, json_string_pretty, out_path_arg, path_from_arg, policy_path,
    read_json, read_utf8, run_container_runtime_check, success_line, write_utf8,
};
use self::content_support::{
    line_has_network_command, load_toml, markdown_code_value, sha256_hex, table_array_strings,
    table_bool, table_string,
};
use self::registry_catalog::{
    apptainer_def_paths, apptainer_tool_ids, canonical_container_label_keys,
    canonical_metadata_labels, docker_image_labels, docker_tool_ids, dockerfile_paths,
    governed_container_file_ids, governed_container_statuses, images_metadata,
    is_non_bijux_apptainer_source, missing_container_label_markers, registry_tool_map,
    registry_tool_rows, tool_status_manifest, toolkit_bundles,
};
use self::runtime::{
    check_apptainer_cache_policy, check_apptainer_frontend_reproducibility,
    check_apptainer_frontend_security, check_apptainer_frontend_smoke_proof,
    check_apptainer_frontend_version_output_lock, check_apptainer_hardening,
    check_apptainer_post_pins, check_apptainer_version_label_sync, check_bijux_apptainer_built,
    check_missing_images, check_non_bijux_sources, check_owners, check_registry_vs_defs,
    checked_container_type, compare_apptainer_smoke_modes, compare_frontend_local_sif_hash,
    container_artifact_dir, ensure_no_args, env_or_default, env_or_empty,
    generate_local_apptainer_digests, merge_outcomes, primary_tools_csv, required_env,
    resolve_toolkit_tools, resolved_smoke_tools, run_argv, run_argv_with_env, run_program_with_env,
    run_runtime_smoke_contract,
};
use self::version_state::{
    all_registry_paths, append_toml_table, container_version_deprecations_path, lock_items_by_tool,
    lock_json_path, parse_date, production_registry_paths, read_lock_json,
    registry_deprecations_path, set_registry_status, set_versions_status, tool_versions,
    VersionMapItem,
};

pub fn run_native_container_command(
    key: NativeContainerCommandKey,
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    dispatch::run_native_container_command(key, workspace, args)
}

fn command_hostname() -> String {
    for args in [["-f"].as_slice(), [].as_slice()] {
        let mut command = std::process::Command::new("hostname");
        command.args(args);
        let Ok(output) = command.output() else {
            continue;
        };
        if output.status.success() {
            let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !value.is_empty() {
                return value;
            }
        }
    }
    String::new()
}
