use bijux_pipelines_bam::{
    bam_adna_capture_profile, bam_adna_shotgun_profile, bam_default_profile,
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
