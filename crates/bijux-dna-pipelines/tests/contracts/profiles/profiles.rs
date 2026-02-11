/// Snapshot intent: verifies stable, reviewed output for this contract.
use std::path::PathBuf;

use bijux_dna_pipelines::bam::{
    bam_adna_capture_profile, bam_adna_shotgun_profile, bam_default_profile,
};
use bijux_dna_pipelines::cross::{fastq_to_bam_adna_shotgun_profile, fastq_to_bam_default_profile};
use bijux_dna_pipelines::fastq::{
    fastq_default_profile, fastq_minimal_profile, fastq_reference_adna_profile,
};
use bijux_dna_testkit::snapshot_name;
use insta::assert_json_snapshot;

fn profile_drift_components(profile: &bijux_dna_pipelines::PipelineProfile) -> serde_json::Value {
    use bijux_dna_core::prelude::hashing::params_hash;
    use std::collections::BTreeMap;

    let tool_map = profile
        .defaults
        .tools
        .iter()
        .map(|(stage, tool)| (stage.as_str().to_string(), tool.as_str().to_string()))
        .collect::<BTreeMap<_, _>>();
    let param_map = profile
        .defaults
        .params
        .iter()
        .map(|(stage, params)| {
            (
                stage.as_str().to_string(),
                params_hash(&params.to_json()).expect("hash profile params"),
            )
        })
        .collect::<BTreeMap<_, _>>();

    let tool_hash = params_hash(&serde_json::to_value(&tool_map).expect("serialize tools map"))
        .expect("hash tools map");
    let params_hash_value =
        params_hash(&serde_json::to_value(&param_map).expect("serialize params hash map"))
            .expect("hash params map");

    serde_json::json!({
        "tool_bindings_hash": tool_hash,
        "param_bindings_hash": params_hash_value,
        "tool_bindings": tool_map,
        "param_bindings": param_map,
    })
}

fn prune_bam_downstream(value: &mut serde_json::Value) {
    let banned = ["bam.genotyping", "bam.haplogroups", "bam.kinship"];
    match value {
        serde_json::Value::Array(items) => {
            items.retain(|item| !item.as_str().is_some_and(|entry| banned.contains(&entry)));
            for item in items {
                prune_bam_downstream(item);
            }
        }
        serde_json::Value::Object(map) => {
            for key in banned {
                map.remove(key);
            }
            for value in map.values_mut() {
                prune_bam_downstream(value);
            }
        }
        _ => {}
    }
}

fn snapshot_settings() -> insta::Settings {
    let mut settings = insta::Settings::clone_current();
    settings.set_snapshot_path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots"));
    settings.set_prepend_module_to_snapshot(false);
    settings
}

#[test]
fn bam_default_profile_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "bam_default_profile");
    let mut json = serde_json::to_value(bam_default_profile()).expect("serialize profile");
    prune_bam_downstream(&mut json);
    assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
}

#[test]
fn bam_adna_shotgun_profile_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "bam_adna_shotgun_profile");
    let mut json = serde_json::to_value(bam_adna_shotgun_profile()).expect("serialize profile");
    prune_bam_downstream(&mut json);
    assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
}

#[test]
fn bam_adna_capture_profile_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "bam_adna_capture_profile");
    let mut json = serde_json::to_value(bam_adna_capture_profile()).expect("serialize profile");
    prune_bam_downstream(&mut json);
    assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
}

#[test]
fn fastq_default_profile_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "fastq_default_profile");
    let json = serde_json::to_value(fastq_default_profile()).expect("serialize profile");
    assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
}

#[test]
fn fastq_minimal_profile_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "fastq_minimal_profile");
    let json = serde_json::to_value(fastq_minimal_profile()).expect("serialize profile");
    assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
}

#[test]
fn fastq_reference_adna_profile_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "fastq_reference_adna_profile");
    let json = serde_json::to_value(fastq_reference_adna_profile()).expect("serialize profile");
    assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
}

#[test]
fn fastq_reference_adna_profile_golden_stage_tool_and_param_hashes_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "fastq_reference_adna_profile_golden");
    let profile = fastq_reference_adna_profile();
    let stage_list = profile
        .capabilities
        .required_stages
        .iter()
        .map(|stage| (*stage).to_string())
        .collect::<Vec<_>>();
    let mut tool_ids = std::collections::BTreeMap::new();
    let mut param_hashes = std::collections::BTreeMap::new();
    for (stage, tool) in &profile.defaults.tools {
        tool_ids.insert(stage.as_str().to_string(), tool.as_str().to_string());
    }
    for (stage, params) in &profile.defaults.params {
        let hash = bijux_dna_core::prelude::hashing::params_hash(&params.to_json())
            .expect("hash default params");
        param_hashes.insert(stage.as_str().to_string(), hash);
    }
    let json = serde_json::json!({
        "profile_id": profile.id,
        "required_stages": stage_list,
        "tool_ids": tool_ids,
        "param_hashes": param_hashes,
    });
    assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
}

#[test]
fn cross_fastq_to_bam_adna_profile_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "fastq_to_bam_adna_shotgun_profile");
    let mut json =
        serde_json::to_value(fastq_to_bam_adna_shotgun_profile()).expect("serialize profile");
    prune_bam_downstream(&mut json);
    assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
}

#[test]
fn cross_fastq_to_bam_default_profile_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "fastq_to_bam_default_profile");
    let mut json = serde_json::to_value(fastq_to_bam_default_profile()).expect("serialize profile");
    prune_bam_downstream(&mut json);
    assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
}

#[test]
fn profile_hash_contract_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "profile_hash_contract");
    let mut hashes = std::collections::BTreeMap::new();
    let mut profiles = Vec::new();
    profiles.extend(bijux_dna_pipelines::registry::fastq_profiles());
    profiles.extend(bijux_dna_pipelines::registry::bam_profiles());
    profiles.extend(bijux_dna_pipelines::registry::cross_profiles());
    profiles.extend(bijux_dna_pipelines::registry::vcf_profiles());
    for profile in profiles {
        hashes.insert(
            profile.id.as_str().to_string(),
            serde_json::json!({
                "profile_hash": profile.profile_hash(),
                "manifest": profile.profile_manifest(),
                "drift_components": profile_drift_components(&profile),
            }),
        );
    }
    assert_json_snapshot!(
        name,
        bijux_dna_testkit::snapshot_normalize_json(&serde_json::json!(hashes))
    );
}
