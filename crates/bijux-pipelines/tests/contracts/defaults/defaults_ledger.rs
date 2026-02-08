/// Snapshot intent: verifies stable, reviewed output for this contract.
use std::path::PathBuf;

use bijux_pipelines::registry::{bam_profiles, cross_profiles, fastq_profiles};
use bijux_testkit::snapshot_name;

#[test]
fn defaults_ledger_snapshots_are_stable() {
    let mut settings = insta::Settings::clone_current();
    settings.set_snapshot_path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots"));
    settings.set_prepend_module_to_snapshot(false);
    let _guard = settings.bind_to_scope();

    let mut profiles = Vec::new();
    profiles.extend(fastq_profiles());
    profiles.extend(bam_profiles());
    profiles.extend(cross_profiles());

    for profile in profiles {
        let base = format!("defaults__{}", profile.id.as_str().replace([':', '.'], "_"));
        let name = snapshot_name("contracts", &base);
        let ledger = profile.defaults_ledger();
        insta::assert_json_snapshot!(name, ledger);
    }
}
