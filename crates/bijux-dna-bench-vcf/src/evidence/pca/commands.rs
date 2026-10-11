use crate::adapters::{eigensoft::VcfEigensoftAdapterRow, plink::VcfPlinkFamilyAdapterRow};
use bijux_dna_bench_vcf_evidence::{
    commands::projection::render_shell_command,
    model::commands::{VcfRenderedCommandRow, VcfRenderedCommandStep},
};
pub(super) fn vcf_pca_plink_command_row(row: VcfPlinkFamilyAdapterRow) -> VcfRenderedCommandRow {
    let command_steps = row
        .command_steps
        .into_iter()
        .map(|step| VcfRenderedCommandStep {
            step_id: step.step_id,
            step_kind: step.step_kind,
            consumes_previous_stdout: false,
            command: render_shell_command(&step.argv),
            argv: step.argv,
        })
        .collect::<Vec<_>>();
    VcfRenderedCommandRow {
        stage_id: row.stage_id,
        tool_id: row.tool_id,
        readiness_kind: "benchmark_ready".to_string(),
        benchmark_status: row.benchmark_status,
        command_source: "vcf_plink_family_adapter".to_string(),
        script_commands: command_steps.iter().map(|step| step.command.clone()).collect(),
        command_steps,
        reason: row.reason,
    }
}
pub(super) fn vcf_pca_eigensoft_command_row(row: VcfEigensoftAdapterRow) -> VcfRenderedCommandRow {
    let command_steps = row
        .command_steps
        .into_iter()
        .map(|step| VcfRenderedCommandStep {
            step_id: step.step_id,
            step_kind: step.step_kind,
            consumes_previous_stdout: false,
            command: render_shell_command(&step.argv),
            argv: step.argv,
        })
        .collect::<Vec<_>>();
    VcfRenderedCommandRow {
        stage_id: row.stage_id,
        tool_id: row.tool_id,
        readiness_kind: "benchmark_ready".to_string(),
        benchmark_status: row.benchmark_status,
        command_source: "vcf_eigensoft_adapter".to_string(),
        script_commands: command_steps.iter().map(|step| step.command.clone()).collect(),
        command_steps,
        reason: row.reason,
    }
}
