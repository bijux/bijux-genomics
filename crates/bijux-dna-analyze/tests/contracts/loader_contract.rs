use bijux_dna_analyze::load::{load_facts, load_run_index, load_run_summary, AnalyzeError};

#[test]
fn load_facts_rejects_bad_schema() -> anyhow::Result<()> {
    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let path = dir.path().join("facts.jsonl");
    bijux_dna_infra::write_bytes(
        &path,
        r#"{"schema_version":"bijux.facts.v0","run_id":"r","stage_id":"s","tool_id":"t","tool_version":"1","image_digest":null,"trace_id":"tr","span_id":"sp","params_hash":"p","input_hash":"i","output_hashes":[],"runtime_s":1.0,"memory_mb":1.0,"exit_code":0,"bank_hashes":{},"reads_in":1,"reads_out":1,"bases_in":1,"bases_out":1,"pairs_in":null,"pairs_out":null,"metrics":{},"reports":{},"artifacts":{}}"#,
    )?;
    match load_facts(&path) {
        Err(err) => match err {
            AnalyzeError::InvalidSchemaVersion { .. } => Ok(()),
            _ => anyhow::bail!("expected InvalidSchemaVersion"),
        },
        Ok(_) => anyhow::bail!("expected error"),
    }
}

#[test]
fn load_run_summary_rejects_bad_schema() -> anyhow::Result<()> {
    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let path = dir.path().join("run_summary.json");
    bijux_dna_infra::write_bytes(
        &path,
        r#"{"schema_version":"bijux.run_summary.v0","facts_path":null,"report_path":null,"telemetry_path":null,"final_outputs":[],"runs":0,"stages":0,"total_runtime_s":0.0,"avg_runtime_s":0.0,"stage_rows":[]}"#,
    )?;
    match load_run_summary(&path) {
        Err(err) => match err {
            AnalyzeError::InvalidSchemaVersion { .. } => Ok(()),
            _ => anyhow::bail!("expected InvalidSchemaVersion"),
        },
        Ok(_) => anyhow::bail!("expected error"),
    }
}

#[test]
fn load_run_index_rejects_bad_schema() -> anyhow::Result<()> {
    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let path = dir.path().join("run_index.jsonl");
    bijux_dna_infra::write_bytes(&path, r#"{"schema_version":2,"run":null,"stage":null}"#)?;
    match load_run_index(&path) {
        Err(err) => match err {
            AnalyzeError::InvalidSchemaVersion { .. } => Ok(()),
            _ => anyhow::bail!("expected InvalidSchemaVersion"),
        },
        Ok(_) => anyhow::bail!("expected error"),
    }
}
