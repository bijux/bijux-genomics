/// Snapshot intent: verifies stable, reviewed output for this contract.
use std::path::PathBuf;

use bijux_pipelines::bam::{
    bam_adna_capture_profile, bam_adna_shotgun_profile, bam_default_profile,
};
use bijux_pipelines::cross::{fastq_to_bam_adna_shotgun_profile, fastq_to_bam_default_profile};
use bijux_pipelines::fastq::{fastq_default_profile, fastq_minimal_profile};
use bijux_testkit::snapshot_name;
use insta::assert_json_snapshot;

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
    let json = serde_json::to_value(bam_default_profile()).expect("serialize profile");
    assert_json_snapshot!(name, bijux_testkit::snapshot_normalize_json(&json));
}

#[test]
fn bam_adna_shotgun_profile_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "bam_adna_shotgun_profile");
    let json = serde_json::to_value(bam_adna_shotgun_profile()).expect("serialize profile");
    assert_json_snapshot!(name, bijux_testkit::snapshot_normalize_json(&json));
}

#[test]
fn bam_adna_capture_profile_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "bam_adna_capture_profile");
    let json = serde_json::to_value(bam_adna_capture_profile()).expect("serialize profile");
    assert_json_snapshot!(name, bijux_testkit::snapshot_normalize_json(&json));
}

#[test]
fn fastq_default_profile_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "fastq_default_profile");
    let json = serde_json::to_value(fastq_default_profile()).expect("serialize profile");
    assert_json_snapshot!(name, bijux_testkit::snapshot_normalize_json(&json));
}

#[test]
fn fastq_minimal_profile_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "fastq_minimal_profile");
    let json = serde_json::to_value(fastq_minimal_profile()).expect("serialize profile");
    assert_json_snapshot!(name, bijux_testkit::snapshot_normalize_json(&json));
}

#[test]
fn cross_fastq_to_bam_adna_profile_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "fastq_to_bam_adna_shotgun_profile");
    let json =
        serde_json::to_value(fastq_to_bam_adna_shotgun_profile()).expect("serialize profile");
    assert_json_snapshot!(name, bijux_testkit::snapshot_normalize_json(&json));
}

#[test]
fn cross_fastq_to_bam_default_profile_snapshot() {
    let _guard = snapshot_settings().bind_to_scope();
    let name = snapshot_name("contracts", "fastq_to_bam_default_profile");
    let json = serde_json::to_value(fastq_to_bam_default_profile()).expect("serialize profile");
    assert_json_snapshot!(name, bijux_testkit::snapshot_normalize_json(&json));
}
