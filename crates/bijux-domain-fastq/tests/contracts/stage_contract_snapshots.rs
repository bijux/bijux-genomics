use bijux_domain_fastq::{stage_contract_json, STAGES};

#[test]
fn stage_contract_snapshots() {
    for stage_id in STAGES {
        let stage_str = stage_id.as_str();
        if let Some(json) = stage_contract_json(stage_str) {
            let snapshot_name = format!("stage_contract__{}", stage_str.replace('.', "_"));
            insta::assert_json_snapshot!(snapshot_name, json);
        }
    }
}
