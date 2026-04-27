#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use sha2::Digest;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use bijux_dna_pipelines::registry::{bam_profiles, cross_profiles, fastq_profiles};
use bijux_dna_pipelines::StabilityTier;
use support::workspace_root;

fn parse_registry(path: &std::path::Path) -> toml::Value {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    raw.parse::<toml::Value>().unwrap_or_else(|err| panic!("parse {}: {err}", path.display()))
}

fn tools_by_id(parsed: &toml::Value) -> BTreeMap<String, toml::Value> {
    parsed
        .get("tools")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|entry| {
            let id = entry.get("id").and_then(toml::Value::as_str)?.to_string();
            Some((id, entry))
        })
        .collect()
}

fn str_field<'a>(table: &'a toml::Value, key: &str) -> &'a str {
    table.get(key).and_then(toml::Value::as_str).unwrap_or("")
}

#[test]
fn policy__contracts__tool_registry_reproducibility_policy__production_registry_is_pinned_and_non_floating(
) {
    let root = workspace_root();
    let mut offenders = Vec::new();

    for rel in
        ["configs/ci/registry/tool_registry.toml", "configs/ci/registry/tool_registry_vcf.toml"]
    {
        let registry = parse_registry(&root.join(rel));
        let tools = tools_by_id(&registry);

        for (id, tool) in tools {
            let status = str_field(&tool, "status");
            if !support::registry_status_is_production(status) {
                continue;
            }

            let upstream = str_field(&tool, "upstream");
            let version = str_field(&tool, "default_version");
            let pin = str_field(&tool, "pinned_commit");
            let metrics_schema = str_field(&tool, "metrics_schema");
            let citation = str_field(&tool, "citation");
            let license = str_field(&tool, "license");
            let version_rule = str_field(&tool, "version_rule");
            let container_ref = str_field(&tool, "container_ref");
            let expected_bin = str_field(&tool, "expected_bin");
            let version_cmd = str_field(&tool, "version_cmd");
            let help_cmd = str_field(&tool, "help_cmd");
            let dockerfile = str_field(&tool, "dockerfile");

            if upstream.eq_ignore_ascii_case("unknown") {
                offenders.push(format!(
                    "{rel}: tool={id}: upstream cannot be unknown in production registry"
                ));
            }
            if version == "latest-pinned" {
                offenders.push(format!(
                    "{rel}: tool={id}: latest-pinned is forbidden in production registry"
                ));
            }
            if pin.is_empty() || pin == "domain-managed" || pin == "unresolved" {
                offenders.push(format!("{rel}: tool={id}: immutable pin is required"));
            }
            if metrics_schema == "bijux.unknown.v1" {
                offenders.push(format!(
                    "{rel}: tool={id}: unknown metrics schema is forbidden in production registry"
                ));
            }
            if citation.is_empty() || citation.starts_with("pending:") {
                offenders.push(format!("{rel}: tool={id}: citation must be concrete"));
            }
            if license.is_empty() {
                offenders.push(format!("{rel}: tool={id}: license is required"));
            }
            if version_rule.is_empty() {
                offenders.push(format!("{rel}: tool={id}: version_rule is required"));
            }
            if container_ref.contains(":latest") {
                offenders.push(format!(
                    "{rel}: tool={id}: floating container tag is forbidden ({container_ref})"
                ));
            }
            if expected_bin.is_empty() {
                offenders.push(format!("{rel}: tool={id}: expected_bin is required"));
            }
            if !version_cmd.contains("--version") {
                offenders.push(format!("{rel}: tool={id}: version_cmd must run --version"));
            }
            if !(help_cmd.contains("--help") || help_cmd.contains(" -h")) {
                offenders.push(format!("{rel}: tool={id}: help_cmd must run --help/-h"));
            }
            if !version_cmd.contains(expected_bin) {
                offenders.push(format!("{rel}: tool={id}: version_cmd must invoke expected_bin"));
            }
            if !help_cmd.contains(expected_bin) {
                offenders.push(format!("{rel}: tool={id}: help_cmd must invoke expected_bin"));
            }
            let dockerfile_path = root.join(dockerfile);
            if !dockerfile.is_empty() && dockerfile_path.exists() {
                let content = std::fs::read_to_string(&dockerfile_path).unwrap_or_default();
                if !(content.contains("ENTRYPOINT") || content.contains("CMD [")) {
                    offenders.push(format!("{rel}: tool={id}: dockerfile missing ENTRYPOINT/CMD"));
                }
                if !content.contains(expected_bin) {
                    offenders.push(format!(
                        "{rel}: tool={id}: dockerfile must reference expected_bin `{expected_bin}`"
                    ));
                }
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "production tool registry reproducibility violations:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__tool_registry_reproducibility_policy__required_tools_are_present_in_production_registry(
) {
    let root = workspace_root();
    let registry = parse_registry(&root.join("configs/ci/registry/tool_registry.toml"));
    let tool_ids = tools_by_id(&registry).keys().cloned().collect::<BTreeSet<_>>();
    let required = parse_registry(&root.join("configs/ci/tools/required_tools.toml"));
    let required_tools = required
        .get("required_tools")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| value.as_str().map(str::to_string))
        .collect::<BTreeSet<_>>();

    let missing = required_tools
        .difference(&tool_ids)
        .filter(|tool_id| tool_id.as_str() != "planner")
        .cloned()
        .collect::<Vec<_>>();
    bijux_dna_policies::policy_assert!(
        missing.is_empty(),
        "required tools missing from production registry: {}",
        missing.join(", ")
    );
}

#[test]
fn policy__contracts__tool_registry_reproducibility_policy__profiles_only_use_valid_production_tools(
) {
    let root = workspace_root();
    let production =
        tools_by_id(&parse_registry(&root.join("configs/ci/registry/tool_registry.toml")));
    let experimental = tools_by_id(&parse_registry(
        &root.join("configs/ci/registry/tool_registry_experimental.toml"),
    ));

    let mut profiles = Vec::new();
    profiles.extend(fastq_profiles());
    profiles.extend(bam_profiles());
    profiles.extend(cross_profiles());

    let mut offenders = Vec::new();
    for profile in profiles {
        if profile.stability != StabilityTier::Stable {
            continue;
        }
        for (stage_id, tool_id) in &profile.defaults.tools {
            let tool_key = tool_id.as_str().to_string();
            if tool_key == "planner" {
                continue;
            }
            if let Some(tool) = production.get(&tool_key) {
                if str_field(tool, "metrics_schema") == "bijux.unknown.v1" {
                    offenders.push(format!(
                        "profile={} stage={} tool={} has unknown metrics schema in production",
                        profile.id, stage_id, tool_key
                    ));
                }
                continue;
            }
            if experimental.contains_key(&tool_key) {
                offenders.push(format!(
                    "profile={} stage={} tool={} is experimental but used by stable profile",
                    profile.id, stage_id, tool_key
                ));
            } else {
                offenders.push(format!(
                    "profile={} stage={} tool={} missing from production registry",
                    profile.id, stage_id, tool_key
                ));
            }
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "profile to production tool registry violations:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__tool_registry_reproducibility_policy__profiles_release_readiness_gate() {
    let root = workspace_root();
    let production =
        tools_by_id(&parse_registry(&root.join("configs/ci/registry/tool_registry.toml")));
    let experimental = tools_by_id(&parse_registry(
        &root.join("configs/ci/registry/tool_registry_experimental.toml"),
    ));

    let mut profiles = Vec::new();
    profiles.extend(fastq_profiles());
    profiles.extend(bam_profiles());
    profiles.extend(cross_profiles());

    let mut offenders = Vec::new();
    for profile in profiles {
        if profile.stability != StabilityTier::Stable {
            continue;
        }
        for (stage_id, tool_id) in &profile.defaults.tools {
            let tool_key = tool_id.as_str().to_string();
            if tool_key == "planner" {
                continue;
            }
            if experimental.contains_key(&tool_key) {
                offenders.push(format!(
                    "profile={} stage={} tool={} is experimental",
                    profile.id, stage_id, tool_key
                ));
                continue;
            }
            let Some(tool) = production.get(&tool_key) else {
                offenders.push(format!(
                    "profile={} stage={} tool={} missing from production registry",
                    profile.id, stage_id, tool_key
                ));
                continue;
            };
            let metrics_schema = str_field(tool, "metrics_schema");
            let default_version = str_field(tool, "default_version");
            let container_ref = str_field(tool, "container_ref");
            let pin = str_field(tool, "pinned_commit");
            if metrics_schema == "bijux.unknown.v1" {
                offenders.push(format!(
                    "profile={} stage={} tool={} uses unknown metrics schema",
                    profile.id, stage_id, tool_key
                ));
            }
            if default_version == "latest-pinned" || container_ref.contains(":latest") {
                offenders.push(format!(
                    "profile={} stage={} tool={} uses floating pin",
                    profile.id, stage_id, tool_key
                ));
            }
            if pin.is_empty() || pin == "domain-managed" || pin == "unresolved" {
                offenders.push(format!(
                    "profile={} stage={} tool={} missing immutable pin",
                    profile.id, stage_id, tool_key
                ));
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "release readiness violations:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__tool_registry_reproducibility_policy__reference_adna_profile_uses_production_tools_only(
) {
    let root = workspace_root();
    let production =
        tools_by_id(&parse_registry(&root.join("configs/ci/registry/tool_registry.toml")));
    let experimental = tools_by_id(&parse_registry(
        &root.join("configs/ci/registry/tool_registry_experimental.toml"),
    ));
    let profile = bijux_dna_pipelines::fastq::fastq_reference_adna_profile();
    let mut offenders = Vec::new();

    for (stage_id, tool_id) in &profile.defaults.tools {
        let tool_key = tool_id.as_str().to_string();
        if tool_key == "planner" {
            continue;
        }
        if experimental.contains_key(&tool_key) {
            offenders.push(format!(
                "stage={} tool={} is experimental",
                stage_id.as_str(),
                tool_key
            ));
            continue;
        }
        if !production.contains_key(&tool_key) {
            offenders.push(format!(
                "stage={} tool={} missing from production registry",
                stage_id.as_str(),
                tool_key
            ));
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "reference aDNA profile must not use experimental tools:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__tool_registry_reproducibility_policy__tool_digest_contract_lock_matches_registry(
) {
    let root = workspace_root();
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
        let path = root.join(rel);
        let raw =
            std::fs::read(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
        let mut file_hasher = sha2::Sha256::new();
        file_hasher.update(raw);
        payload.push_str(rel);
        payload.push(' ');
        payload.push_str(&sha256_hex(file_hasher.finalize()));
        payload.push('\n');
    }
    let mut hasher = sha2::Sha256::new();
    hasher.update(payload.as_bytes());
    let expected_hash = sha256_hex(hasher.finalize());
    let lock_path = root.join("configs/ci/registry/tool_registry_lock.sha256");
    let actual_hash = std::fs::read_to_string(&lock_path)
        .unwrap_or_else(|err| panic!("read lockfile: {err}"))
        .trim()
        .to_string();
    bijux_dna_policies::policy_assert!(
        actual_hash == expected_hash,
        "tool digest contract violated: configs/ci/registry/tool_registry_lock.sha256 is stale; update it after tool pin changes"
    );
}

fn sha256_hex(digest: impl AsRef<[u8]>) -> String {
    let bytes = digest.as_ref();
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(&mut hex, "{byte:02x}");
    }
    hex
}

#[test]
fn policy__contracts__tool_registry_reproducibility_policy__production_registries_forbid_latest_pinned(
) {
    let root = workspace_root();
    let mut offenders = Vec::new();
    for rel in
        ["configs/ci/registry/tool_registry.toml", "configs/ci/registry/tool_registry_vcf.toml"]
    {
        let registry = parse_registry(&root.join(rel));
        for (id, tool) in tools_by_id(&registry) {
            let version = str_field(&tool, "version");
            let default_version = str_field(&tool, "default_version");
            if version == "latest-pinned" || default_version == "latest-pinned" {
                offenders
                    .push(format!("{rel}: tool={id} uses forbidden latest-pinned version marker"));
            }
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "production registry versions must be immutable:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__tool_registry_reproducibility_policy__production_registries_forbid_unknown_upstream_and_domain_managed_pins(
) {
    let root = workspace_root();
    let mut offenders = Vec::new();
    for rel in
        ["configs/ci/registry/tool_registry.toml", "configs/ci/registry/tool_registry_vcf.toml"]
    {
        let registry = parse_registry(&root.join(rel));
        for (id, tool) in tools_by_id(&registry) {
            let upstream = str_field(&tool, "upstream");
            let pin = str_field(&tool, "pinned_commit");
            if upstream.eq_ignore_ascii_case("unknown") {
                offenders.push(format!("{rel}: tool={id} has upstream=unknown"));
            }
            if pin == "domain-managed" {
                offenders.push(format!("{rel}: tool={id} has pinned_commit=domain-managed"));
            }
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "production registries must avoid unknown upstreams and domain-managed pins:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__tool_registry_reproducibility_policy__stable_profiles_must_not_use_unknown_declared_versions(
) {
    let root = workspace_root();
    let mut tools =
        tools_by_id(&parse_registry(&root.join("configs/ci/registry/tool_registry.toml")));
    for (id, value) in
        tools_by_id(&parse_registry(&root.join("configs/ci/registry/tool_registry_vcf.toml")))
    {
        tools.entry(id).or_insert(value);
    }

    let mut profiles = Vec::new();
    profiles.extend(fastq_profiles());
    profiles.extend(bam_profiles());
    profiles.extend(cross_profiles());

    let mut offenders = Vec::new();
    for profile in profiles {
        if profile.stability != StabilityTier::Stable {
            continue;
        }
        for (stage_id, tool_id) in &profile.defaults.tools {
            let tool_key = tool_id.as_str().to_string();
            if tool_key == "planner" {
                continue;
            }
            let Some(tool) = tools.get(&tool_key) else {
                offenders.push(format!(
                    "profile={} stage={} tool={} missing from production registries",
                    profile.id, stage_id, tool_key
                ));
                continue;
            };
            let declared_version = str_field(tool, "version");
            if declared_version.trim().is_empty()
                || declared_version.eq_ignore_ascii_case("unknown")
            {
                offenders.push(format!(
                    "profile={} stage={} tool={} has invalid declared version `{declared_version}`",
                    profile.id, stage_id, tool_key
                ));
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "stable profiles cannot use tools with unknown declared_version:\n{}",
        offenders.join("\n")
    );
}
