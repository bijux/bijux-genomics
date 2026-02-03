use bijux_pipelines::bam::{
    bam_adna_capture_profile, bam_adna_shotgun_profile, bam_default_profile,
};
use bijux_pipelines::cross::{fastq_to_bam_adna_shotgun_profile, fastq_to_bam_default_profile};
use bijux_pipelines::fastq::{
    fastq_default_profile, fastq_minimal_profile, DefaultPipelineOptions,
};
use insta::assert_json_snapshot;

#[test]
fn bam_default_profile_snapshot() {
    assert_json_snapshot!("bam_default_profile", bam_default_profile());
}

#[test]
fn bam_adna_shotgun_profile_snapshot() {
    assert_json_snapshot!("bam_adna_shotgun_profile", bam_adna_shotgun_profile());
}

#[test]
fn bam_adna_capture_profile_snapshot() {
    assert_json_snapshot!("bam_adna_capture_profile", bam_adna_capture_profile());
}

#[test]
fn fastq_default_profile_snapshot() {
    assert_json_snapshot!(
        "fastq_default_profile",
        fastq_default_profile(DefaultPipelineOptions::default())
    );
}

#[test]
fn fastq_minimal_profile_snapshot() {
    assert_json_snapshot!("fastq_minimal_profile", fastq_minimal_profile());
}

#[test]
fn cross_fastq_to_bam_adna_profile_snapshot() {
    assert_json_snapshot!(
        "fastq_to_bam_adna_shotgun_profile",
        fastq_to_bam_adna_shotgun_profile()
    );
}

#[test]
fn cross_fastq_to_bam_default_profile_snapshot() {
    assert_json_snapshot!(
        "fastq_to_bam_default_profile",
        fastq_to_bam_default_profile()
    );
}
