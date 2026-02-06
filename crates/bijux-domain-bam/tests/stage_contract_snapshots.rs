use bijux_domain_bam::{stage_contract_json, BamStage};

#[test]
fn stage_contract_snapshots() {
    for stage in BamStage::all() {
        let stage_id = stage.as_str();
        if let Some(json) = stage_contract_json(stage_id) {
            let snapshot_name = format!("stage_contract__{}", stage_id.replace('.', "_"));
            insta::assert_json_snapshot!(snapshot_name, json);
        }
    }
}
