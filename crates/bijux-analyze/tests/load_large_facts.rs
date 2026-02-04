use std::fmt::Write;
use bijux_analyze::load::load_facts;

#[test]
fn load_facts_handles_large_jsonl() -> anyhow::Result<()> {
    let dir = bijux_infra::temp_dir("bijux")?;
    let path = dir.path().join("facts.jsonl");
    let mut payload = String::new();
    for i in 0..500 {
        writeln!(
            payload,
            "{{\"schema_version\":\"bijux.facts.v1\",\"run_id\":\"run\",\"stage_id\":\"fastq.trim\",\"tool_id\":\"fastp\",\"tool_version\":\"1\",\"image_digest\":null,\"trace_id\":\"t\",\"span_id\":\"s\",\"params_hash\":\"p{i}\",\"input_hash\":\"i\",\"output_hashes\":[],\"runtime_s\":1.0,\"memory_mb\":1.0,\"exit_code\":0,\"bank_hashes\":{{}},\"reads_in\":1,\"reads_out\":1,\"bases_in\":1,\"bases_out\":1,\"pairs_in\":null,\"pairs_out\":null,\"metrics\":{{}},\"reports\":{{}},\"artifacts\":{{}}}}"
        )?;
    }
    bijux_infra::write_bytes(&path, payload)?;
    let rows = load_facts(&path)?;
    assert_eq!(rows.len(), 500);
    Ok(())
}
