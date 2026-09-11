use super::observations::evidence;
use anyhow::Result;
use bijux_dna_bench_vcf_evidence::{
    commands::{
        admission::validate_vcf_rendered_command_row,
        projection::{
            render_shell_command, render_vcf_command_argv_jsonl, render_vcf_commands_shell_script,
        },
    },
    snapshot::VcfEvidenceSnapshot,
};

#[test]
fn command_projection_preserves_empty_and_shell_sensitive_arguments() {
    assert_eq!(
        render_shell_command(&[
            "tool".into(),
            String::new(),
            "two words".into(),
            "a'b".into(),
            "$(touch bad)".into()
        ]),
        "tool '' 'two words' 'a'\"'\"'b' '$(touch bad)'"
    );
}

#[test]
fn argv_shell_disagreement_and_invalid_pipeline_are_rejected() {
    let mut row = evidence().commands.remove(0);
    row.command_steps[0].command.push_str("; unwanted");
    assert!(validate_vcf_rendered_command_row(&row).is_err());
    let mut row = evidence().commands.remove(0);
    row.command_steps[0].consumes_previous_stdout = true;
    assert!(validate_vcf_rendered_command_row(&row).is_err());
    let mut row = evidence().commands.remove(0);
    row.script_commands.push("unwanted".into());
    assert!(validate_vcf_rendered_command_row(&row).is_err());
}

#[test]
fn duplicate_steps_and_nul_arguments_are_rejected() {
    let mut row = evidence().commands.remove(0);
    row.command_steps.push(row.command_steps[0].clone());
    assert!(validate_vcf_rendered_command_row(&row).is_err());
    let mut row = evidence().commands.remove(0);
    row.command_steps[0].argv.push("nul\0arg".into());
    assert!(validate_vcf_rendered_command_row(&row).is_err());
}

#[test]
fn wire_projection_keeps_ordered_argv_and_existing_shell_layout() -> Result<()> {
    let rows = evidence().commands;
    let json = render_vcf_command_argv_jsonl(&rows)?;
    let value: serde_json::Value = serde_json::from_str(&json)?;
    assert_eq!(value["command_steps"][0]["argv"][2], "input with spaces.bcf");
    assert_eq!(value["command_steps"][0]["consumes_previous_stdout"], false);
    assert_eq!(render_vcf_commands_shell_script(&rows,"../.."),"#!/usr/bin/env bash\nset -euo pipefail\nrepo_root=\"$(cd \"$(dirname \"${BASH_SOURCE[0]}\")/../..\" && pwd)\"\ncd \"$repo_root\"\n\n# vcf.call / bcftools\nbcftools call 'input with spaces.bcf'\n");
    assert!(VcfEvidenceSnapshot::admit(evidence()).is_ok());
    Ok(())
}
