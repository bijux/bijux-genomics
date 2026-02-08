use bijux_pipelines::bam::{
    bam_adna_capture_profile, bam_adna_shotgun_profile, bam_default_profile,
};
use bijux_pipelines::cross::{fastq_to_bam_adna_shotgun_profile, fastq_to_bam_default_profile};
use bijux_pipelines::fastq::{fastq_default_profile, fastq_minimal_profile};
use bijux_testkit::snapshot_name;
use insta::assert_json_snapshot;

#[test]
fn bam_default_profile_snapshot() {
    let name = snapshot_name("contracts", "bam_default_profile");
    assert_json_snapshot!(name, bam_default_profile());
}

#[test]
fn bam_adna_shotgun_profile_snapshot() {
    let name = snapshot_name("contracts", "bam_adna_shotgun_profile");
    assert_json_snapshot!(name, bam_adna_shotgun_profile());
}

#[test]
fn bam_adna_capture_profile_snapshot() {
    let name = snapshot_name("contracts", "bam_adna_capture_profile");
    assert_json_snapshot!(name, bam_adna_capture_profile());
}

#[test]
fn fastq_default_profile_snapshot() {
    let name = snapshot_name("contracts", "fastq_default_profile");
    assert_json_snapshot!(name, fastq_default_profile());
}

#[test]
fn fastq_minimal_profile_snapshot() {
    let name = snapshot_name("contracts", "fastq_minimal_profile");
    assert_json_snapshot!(name, fastq_minimal_profile());
}

#[test]
fn cross_fastq_to_bam_adna_profile_snapshot() {
    let name = snapshot_name("contracts", "fastq_to_bam_adna_shotgun_profile");
    assert_json_snapshot!(name, fastq_to_bam_adna_shotgun_profile());
}

#[test]
fn cross_fastq_to_bam_default_profile_snapshot() {
    let name = snapshot_name("contracts", "fastq_to_bam_default_profile");
    assert_json_snapshot!(name, fastq_to_bam_default_profile());
}
