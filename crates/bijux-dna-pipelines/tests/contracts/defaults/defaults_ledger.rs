/// Snapshot intent: verifies stable, reviewed output for this contract.
use std::path::PathBuf;

use bijux_dna_pipelines::registry::{bam_profiles, cross_profiles, fastq_profiles};
use bijux_dna_testkit::snapshot_name;

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
        let mut json = serde_json::to_value(&ledger).expect("serialize ledger");
        prune_bam_downstream(&mut json);
        insta::assert_json_snapshot!(name, bijux_dna_testkit::snapshot_normalize_json(&json));
    }
}
