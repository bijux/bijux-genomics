use super::*;
use crate::{AllDomainRenderedCommandRow, AllDomainRenderedCommandStep};

fn observed(argv: &[&str]) -> AllDomainRenderedCommandsReport {
    let row = AllDomainRenderedCommandRow {
        result_id: "observed-result".to_string(),
        domain: "vcf".to_string(),
        stage_id: "vcf.postprocess".to_string(),
        tool_id: "observed-tool".to_string(),
        corpus_id: "truth".to_string(),
        asset_profile_id: "diploid".to_string(),
        readiness_kind: "declared".to_string(),
        benchmark_status: "declared".to_string(),
        command_source: "adapter".to_string(),
        command_steps: vec![AllDomainRenderedCommandStep {
            step_id: "execute".to_string(),
            step_kind: "tool".to_string(),
            consumes_previous_stdout: false,
            argv: argv.iter().map(|s| (*s).to_string()).collect(),
            command: argv.join(" "),
        }],
        script_commands: vec![argv.join(" ")],
    };
    AllDomainRenderedCommandsReport {
        schema_version: crate::ALL_DOMAIN_RENDERED_COMMANDS_SCHEMA_VERSION.to_string(),
        output_path: "commands.sh".to_string(),
        argv_output_path: "commands.jsonl".to_string(),
        row_count: 1,
        result_id_count: 1,
        domain_counts: BTreeMap::new(),
        benchmark_status_counts: BTreeMap::new(),
        command_source_counts: BTreeMap::new(),
        rows: vec![row],
    }
}

#[test]
fn recorded_command_audit_preserves_diagnostics_and_observed_paths() -> Result<()> {
    for argv in [
        vec!["/opt/tools/bcftools", "view", "input.vcf"],
        vec!["bash", "-c", "MODE=quiet bcftools view 'input with spaces.vcf' | bgzip"],
    ] {
        let input = observed(&argv);
        let report = evaluate_recorded_command_audit(&input, "audit.json".to_string())?;
        assert!(report.ok);
        assert_eq!(report.output_path, "audit.json");
        assert_eq!(report.script_output_path, input.output_path);
        assert_eq!(report.argv_output_path, input.argv_output_path);
        assert_eq!(report.command_step_count, 1);
        assert_eq!(report.valid_row_count, 1);
        assert!(report.violations.is_empty());
        assert_eq!(report.rows[0].result_id, input.rows[0].result_id);
    }
    for (argv, finding) in [
        (vec![""], "empty_executable"),
        (vec!["echo", "done"], "echo_only_execution"),
        (vec!["true"], "unconditional_success"),
        (vec!["bash", "-c", "rc=0\nexit $rc"], "unconditional_success"),
        (vec!["tool", "TODO"], "contains_todo_marker"),
        (vec!["tool", "placeholder"], "contains_placeholder"),
    ] {
        let report = evaluate_recorded_command_audit(&observed(&argv), "audit.json".to_string())?;
        assert!(!report.ok);
        assert_eq!(report.invalid_row_count, 1);
        assert_eq!(report.violation_count, 1);
        assert!(report.rows[0].finding_types.iter().any(|v| v == finding));
    }
    Ok(())
}

#[test]
fn recorded_command_audit_rejects_missing_and_duplicate_observations() {
    let original = observed(&["bcftools", "view"]);
    for mutation in 0..6 {
        let mut input = original.clone();
        match mutation {
            0 => input.schema_version = "unsupported".to_string(),
            1 => input.row_count = 2,
            2 => {
                input.rows.clear();
                input.row_count = 0;
            }
            3 => {
                input.rows.push(input.rows[0].clone());
                input.row_count = 2;
            }
            4 => input.rows[0].command_steps.clear(),
            _ => input.rows[0].command_steps[0].step_id.clear(),
        }
        assert!(evaluate_recorded_command_audit(&input, "audit.json".to_string()).is_err());
    }
}
