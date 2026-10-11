use super::*;
use serde_json::json;

fn observations(executable: &str) -> Result<AllDomainRenderedCommandsReport> {
    Ok(serde_json::from_value(json!({
        "schema_version": bijux_dna_bench_model::ALL_DOMAIN_RENDERED_COMMANDS_SCHEMA_VERSION,
        "output_path":"commands.sh", "argv_output_path":"commands.jsonl",
        "row_count":1, "result_id_count":1, "domain_counts":{},
        "benchmark_status_counts":{}, "command_source_counts":{},
        "rows":[{
            "result_id":"record", "domain":"vcf", "stage_id":"vcf.postprocess",
            "tool_id":"observed-tool", "corpus_id":"truth", "asset_profile_id":"diploid",
            "readiness_kind":"declared", "benchmark_status":"declared", "command_source":"adapter",
            "command_steps":[{"step_id":"execute", "step_kind":"tool", "consumes_previous_stdout":false,
                "argv":[executable], "command":executable}], "script_commands":[executable]
        }]
    }))?)
}

#[test]
fn recorded_command_audit_publishes_failures_and_preserves_prior_output_on_admission_error(
) -> Result<()> {
    let root = bijux_dna_infra::temp_dir("recorded-command-audit-")?;
    let output = Path::new("reports/audit.json");
    let valid = observations("bcftools")?;
    let report = write_recorded_command_audit(root.path(), output, &valid)?;
    assert_eq!(report.output_path, "reports/audit.json");
    assert_eq!(std::fs::read(root.path().join(output))?, serde_json::to_vec_pretty(&report)?);
    assert!(write_recorded_command_audit(root.path(), output, &observations("true")?).is_err());
    let failed = std::fs::read(root.path().join(output))?;
    let observed: AllDomainNoPlaceholderCommandCheckReport = serde_json::from_slice(&failed)?;
    assert!(!observed.ok);
    assert_eq!(observed.invalid_row_count, 1);
    let mut malformed = valid.clone();
    malformed.rows[0].command_steps.clear();
    assert!(write_recorded_command_audit(root.path(), output, &malformed).is_err());
    assert_eq!(std::fs::read(root.path().join(output))?, failed);
    let blocked = root.path().join("existing-file");
    std::fs::write(&blocked, b"preserve")?;
    assert!(write_recorded_command_audit(root.path(), &blocked.join("audit.json"), &valid).is_err());
    assert_eq!(std::fs::read(&blocked)?, b"preserve");
    Ok(())
}
