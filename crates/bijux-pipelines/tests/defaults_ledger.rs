use bijux_pipelines::registry::{bam_profiles, cross_profiles, fastq_profiles};

#[test]
fn defaults_ledger_snapshots_are_stable() {
    let mut profiles = Vec::new();
    profiles.extend(fastq_profiles());
    profiles.extend(bam_profiles());
    profiles.extend(cross_profiles());

    for profile in profiles {
        let name = format!("defaults__{}", profile.id.as_str().replace([':', '.'], "_"));
        let ledger = profile.defaults_ledger();
        insta::assert_json_snapshot!(name, ledger);
    }
}
