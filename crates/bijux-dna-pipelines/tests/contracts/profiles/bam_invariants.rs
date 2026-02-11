use bijux_dna_core::ids::StageId;
use bijux_dna_core::prelude::id_catalog;
use bijux_dna_pipelines::bam::{
    bam_adna_shotgun_profile, bam_default_profile, validate_bam_profile,
};

#[test]
fn bam_default_and_adna_profiles_satisfy_invariants() {
    for profile in [bam_default_profile(), bam_adna_shotgun_profile()] {
        let report = validate_bam_profile(&profile);
        assert!(
            report.valid,
            "profile {} failed BAM invariants: {:?}",
            profile.id.as_str(),
            report.violations
        );
    }
}

#[test]
fn bam_adna_requires_damage_stage() {
    let mut profile = bam_adna_shotgun_profile();
    profile
        .capabilities
        .required_stages
        .retain(|stage| *stage != id_catalog::BAM_DAMAGE);
    profile
        .defaults
        .params
        .remove(&StageId::from_static(id_catalog::BAM_DAMAGE));

    let report = validate_bam_profile(&profile);
    assert!(!report.valid);
    assert!(report
        .violations
        .iter()
        .any(|v| v.code == "adna_damage_stage_missing"));
}
