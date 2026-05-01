#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use std::collections::{BTreeMap, BTreeSet};

fn list(table: &toml::Value, key: &str) -> Vec<String> {
    table
        .get(key)
        .and_then(toml::Value::as_array)
        .map(|values| {
            values.iter().filter_map(toml::Value::as_str).map(str::to_string).collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

#[test]
fn policy__contracts__default_profile_binding_policy__default_profiles_use_registry_bound_tools() {
    let root = support::workspace_root();
    let mut tools = Vec::new();
    for rel in [
        "configs/ci/registry/tool_registry.toml",
        "configs/ci/registry/tool_registry_experimental.toml",
        "configs/ci/registry/tool_registry_vcf.toml",
    ] {
        let raw = std::fs::read_to_string(root.join(rel)).expect("read registry");
        let parsed: toml::Value = raw.parse().expect("parse registry");
        tools.extend(
            parsed.get("tools").and_then(toml::Value::as_array).cloned().unwrap_or_default(),
        );
    }
    let tool_bindings = tools
        .iter()
        .filter_map(|tool| {
            let id = tool.get("id").and_then(toml::Value::as_str)?;
            let mut bindings = list(tool, "bindings");
            if bindings.is_empty() {
                bindings = list(tool, "stage_ids");
            }
            Some((id.to_string(), bindings.into_iter().collect::<BTreeSet<_>>()))
        })
        .collect::<BTreeMap<_, _>>();

    let mut profiles = Vec::new();
    profiles.extend(bijux_dna_pipelines::registry::fastq_profiles());
    profiles.extend(bijux_dna_pipelines::registry::bam_profiles());
    profiles.extend(bijux_dna_pipelines::registry::cross_profiles());

    let mut offenders = Vec::new();
    for profile in profiles {
        if !profile.id.as_str().contains("__default__") {
            continue;
        }
        for (stage_id, tool_id) in &profile.defaults.tools {
            let stage = stage_id.as_str().to_string();
            let tool = tool_id.as_str().to_string();
            if tool == "planner" {
                continue;
            }
            if !stage_requires_registry_binding(&stage) {
                continue;
            }
            let Some(bindings) = tool_bindings.get(&tool) else {
                offenders.push(format!(
                    "profile={} stage={} default tool {} missing from registry",
                    profile.id.as_str(),
                    stage,
                    tool
                ));
                continue;
            };
            let stage_aliases = stage_aliases(&stage);
            if !bindings.is_empty()
                && !bindings.contains(&stage)
                && !stage_aliases.iter().any(|alias| bindings.contains(alias))
            {
                offenders.push(format!(
                    "profile={} stage={} default tool {} not bound in registry (bindings={:?})",
                    profile.id.as_str(),
                    stage,
                    tool,
                    bindings
                ));
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "default profile binding drift:\n{}",
        offenders.join("\n")
    );
}

fn stage_aliases(stage: &str) -> Vec<String> {
    match stage {
        "bam.contamination" => vec!["bam.authenticity".to_string()],
        "bam.genotyping" => vec![
            "vcf.call_gl".to_string(),
            "vcf.call_pseudohaploid".to_string(),
            "vcf.damage_filter".to_string(),
            "vcf.gl_propagation".to_string(),
        ],
        _ => Vec::new(),
    }
}

fn stage_requires_registry_binding(stage: &str) -> bool {
    !(stage.starts_with("core.") || stage.starts_with("cross."))
}
