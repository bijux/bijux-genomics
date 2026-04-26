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
fn load_run_summary_reports_unreadable_path() -> anyhow::Result<()> {
    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let path = dir.path().join("run_summary.json");
    std::fs::create_dir(&path)?;

    match load_run_summary(&path) {
        Err(AnalyzeError::InvalidJson { message }) => {
            assert!(message.contains(&path.display().to_string()));
            Ok(())
        }
        Err(err) => anyhow::bail!("expected InvalidJson, got {err}"),
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

#[test]
fn load_run_index_rejects_empty_row_kind() -> anyhow::Result<()> {
    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let path = dir.path().join("run_index.jsonl");
    bijux_dna_infra::write_bytes(&path, r#"{"schema_version":1,"run":null,"stage":null}"#)?;

    match load_run_index(&path) {
        Err(AnalyzeError::InvalidJsonlRow { line, message }) => {
            assert_eq!(line, 1);
            assert!(message.contains("exactly one of run or stage"));
            Ok(())
        }
        Err(err) => anyhow::bail!("expected InvalidJsonlRow, got {err}"),
        Ok(_) => anyhow::bail!("expected error"),
    }
}

#[test]
fn load_run_index_rejects_ambiguous_row_kind() -> anyhow::Result<()> {
    let dir = bijux_dna_infra::temp_dir("bijux")?;
    let path = dir.path().join("run_index.jsonl");
    bijux_dna_infra::write_bytes(
        &path,
        r#"{"schema_version":1,"run":{"run_id":"run-1","domain":"fastq","pipeline":"fastq-to-fastq","stages":["fastq.validate_reads"],"tools":["fastqvalidator"],"objective":"qc","platform":"local","success":true},"stage":{"run_id":"run-1","stage_id":"fastq.validate_reads","tool_id":"fastqvalidator","params_hash":"params","input_hash":"input","output_hashes":["output"],"artifacts":{}}}"#,
    )?;

    match load_run_index(&path) {
        Err(AnalyzeError::InvalidJsonlRow { line, message }) => {
            assert_eq!(line, 1);
            assert!(message.contains("exactly one of run or stage"));
            Ok(())
        }
        Err(err) => anyhow::bail!("expected InvalidJsonlRow, got {err}"),
        Ok(_) => anyhow::bail!("expected error"),
    }
}
