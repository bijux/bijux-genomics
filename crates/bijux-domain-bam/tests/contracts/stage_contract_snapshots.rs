/// Snapshot intent: verifies stable, reviewed output for this contract.

use bijux_domain_bam::{stage_contract_json, BamStage};
use bijux_testkit::snapshot_name;
use insta::Settings;
use std::path::PathBuf;

#[test]
fn stage_contract_snapshots() {
    let mut settings = Settings::new();
    settings.set_prepend_module_to_snapshot(false);
    settings.set_snapshot_path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots"));
    for stage in BamStage::all() {
        let stage_id = stage.as_str();
        if let Some(json) = stage_contract_json(stage_id) {
            let name = snapshot_name(
                "contracts",
                &format!("stage_contract__{}", stage_id.replace('.', "_")),
            );
            settings.bind(|| {
                insta::assert_json_snapshot!(name, json);
            });
        }
    }
}
