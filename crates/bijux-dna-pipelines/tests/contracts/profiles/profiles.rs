/// Snapshot intent: verifies stable, reviewed output for this contract.
use std::path::PathBuf;

use bijux_dna_pipelines::bam::{
    bam_adna_capture_profile, bam_adna_shotgun_profile, bam_default_profile,
};
use bijux_dna_pipelines::cross::{fastq_to_bam_adna_shotgun_profile, fastq_to_bam_default_profile};
use bijux_dna_pipelines::fastq::{fastq_default_profile, fastq_minimal_profile};
use bijux_dna_testkit::snapshot_name;
use insta::assert_json_snapshot;

fn prune_bam_downstream(value: &mut serde_json::Value) {
    let banned = ["bam.genotyping", "bam.haplogroups", "bam.kinship"];
    match value {
        serde_json::Value::Array(items) => {
            items.retain(|item| {
                !item
                    .as_str()
                    .is_some_and(|entry| banned.contains(&entry))
            });
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
