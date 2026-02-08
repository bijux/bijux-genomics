/// Snapshot intent: verifies stable, reviewed output for this contract.
use bijux_dna_domain_fastq::{stage_contract_json, STAGES};
use bijux_dna_testkit::snapshot_name;
use insta::Settings;
use std::path::PathBuf;

#[test]
fn stage_contract_snapshots() {
    let mut settings = Settings::new();
    settings.set_prepend_module_to_snapshot(false);
    settings.set_snapshot_path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots"));
    for stage_id in STAGES {
        let stage_str = stage_id.as_str();
        if let Some(json) = stage_contract_json(stage_str) {
            let name = snapshot_name(
                "contracts",
                &format!("stage_contract__{}", stage_str.replace('.', "_")),
            );
            settings.bind(|| {
                insta::assert_json_snapshot!(
                    name,
                    bijux_dna_testkit::snapshot_normalize_json(&json)
                );
            });
        }
    }
}
