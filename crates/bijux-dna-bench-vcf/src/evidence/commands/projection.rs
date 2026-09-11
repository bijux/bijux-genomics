//! Pure structured-command projection from validated typed family rows.
use bijux_dna_bench_vcf_evidence::{
    commands::projection::render_shell_command,
    model::commands::{VcfRenderedCommandRow, VcfRenderedCommandStep},
};
pub(super) fn append_bcftools(
    rows: &mut Vec<VcfRenderedCommandRow>,
    source_rows: Vec<crate::adapters::bcftools::VcfBcftoolsAdapterRow>,
) {
    rows.extend(source_rows.into_iter().map(|row| {
        VcfRenderedCommandRow {
            stage_id: row.stage_id,
            tool_id: row.tool_id,
            readiness_kind: "benchmark_ready".to_string(),
            benchmark_status: row.benchmark_status,
            command_source: "vcf_bcftools_adapter".to_string(),
            command_steps: row
                .command_steps
                .into_iter()
                .map(|step| VcfRenderedCommandStep {
                    step_id: step.step_id,
                    step_kind: step.step_kind,
                    consumes_previous_stdout: step.consumes_previous_stdout,
                    command: render_shell_command(&step.argv),
                    argv: step.argv,
                })
                .collect(),
            script_commands: Vec::new(),
            reason: row.reason,
        }
    }));
}

pub(super) fn append_angsd(
    rows: &mut Vec<VcfRenderedCommandRow>,
    source_rows: Vec<crate::adapters::angsd::VcfAngsdAdapterRow>,
) {
    rows.extend(source_rows.into_iter().map(|row| {
        VcfRenderedCommandRow {
            stage_id: row.stage_id,
            tool_id: row.tool_id,
            readiness_kind: "benchmark_ready".to_string(),
            benchmark_status: row.benchmark_status,
            command_source: "vcf_angsd_adapter".to_string(),
            command_steps: row
                .command_steps
                .into_iter()
                .map(|step| VcfRenderedCommandStep {
                    step_id: step.step_id,
                    step_kind: step.step_kind,
                    consumes_previous_stdout: false,
                    command: render_shell_command(&step.argv),
                    argv: step.argv,
                })
                .collect(),
            script_commands: Vec::new(),
            reason: row.reason,
        }
    }));
}

pub(super) fn append_eigensoft(
    rows: &mut Vec<VcfRenderedCommandRow>,
    source_rows: Vec<crate::adapters::eigensoft::VcfEigensoftAdapterRow>,
) {
    rows.extend(source_rows.into_iter().map(|row| {
        VcfRenderedCommandRow {
            stage_id: row.stage_id,
            tool_id: row.tool_id,
            readiness_kind: "benchmark_ready".to_string(),
            benchmark_status: row.benchmark_status,
            command_source: "vcf_eigensoft_adapter".to_string(),
            command_steps: row
                .command_steps
                .into_iter()
                .map(|step| VcfRenderedCommandStep {
                    step_id: step.step_id,
                    step_kind: step.step_kind,
                    consumes_previous_stdout: false,
                    command: render_shell_command(&step.argv),
                    argv: step.argv,
                })
                .collect(),
            script_commands: Vec::new(),
            reason: row.reason,
        }
    }));
}

pub(super) fn append_plink(
    rows: &mut Vec<VcfRenderedCommandRow>,
    source_rows: Vec<crate::adapters::plink::VcfPlinkFamilyAdapterRow>,
) {
    rows.extend(source_rows.into_iter().map(|row| {
        VcfRenderedCommandRow {
            stage_id: row.stage_id,
            tool_id: row.tool_id,
            readiness_kind: "benchmark_ready".to_string(),
            benchmark_status: row.benchmark_status,
            command_source: "vcf_plink_family_adapter".to_string(),
            command_steps: row
                .command_steps
                .into_iter()
                .map(|step| VcfRenderedCommandStep {
                    step_id: step.step_id,
                    step_kind: step.step_kind,
                    consumes_previous_stdout: false,
                    command: render_shell_command(&step.argv),
                    argv: step.argv,
                })
                .collect(),
            script_commands: Vec::new(),
            reason: row.reason,
        }
    }));
}

pub(super) fn append_phasing(
    rows: &mut Vec<VcfRenderedCommandRow>,
    source_rows: Vec<crate::adapters::phasing::VcfPhasingFamilyAdapterRow>,
) {
    rows.extend(source_rows.into_iter().filter(|row| row.stage_id == "vcf.phasing").map(|row| {
        VcfRenderedCommandRow {
            stage_id: row.stage_id,
            tool_id: row.tool_id,
            readiness_kind: "benchmark_ready".to_string(),
            benchmark_status: row.benchmark_status,
            command_source: "vcf_phasing_family_adapter".to_string(),
            command_steps: row
                .command_steps
                .into_iter()
                .map(|step| VcfRenderedCommandStep {
                    step_id: step.step_id,
                    step_kind: step.step_kind,
                    consumes_previous_stdout: false,
                    command: render_shell_command(&step.argv),
                    argv: step.argv,
                })
                .collect(),
            script_commands: Vec::new(),
            reason: row.reason,
        }
    }));
}

pub(super) fn append_imputation(
    rows: &mut Vec<VcfRenderedCommandRow>,
    source_rows: Vec<crate::adapters::imputation::VcfImputationFamilyAdapterRow>,
) {
    rows.extend(source_rows.into_iter().map(|row| {
        VcfRenderedCommandRow {
            stage_id: row.stage_id,
            tool_id: row.tool_id,
            readiness_kind: "benchmark_ready".to_string(),
            benchmark_status: row.benchmark_status,
            command_source: "vcf_imputation_family_adapter".to_string(),
            command_steps: row
                .command_steps
                .into_iter()
                .map(|step| VcfRenderedCommandStep {
                    step_id: step.step_id,
                    step_kind: step.step_kind,
                    consumes_previous_stdout: false,
                    command: render_shell_command(&step.argv),
                    argv: step.argv,
                })
                .collect(),
            script_commands: Vec::new(),
            reason: row.reason,
        }
    }));
}

pub(super) fn append_descent(
    rows: &mut Vec<VcfRenderedCommandRow>,
    source_rows: Vec<crate::adapters::descent::VcfDescentFamilyAdapterRow>,
) {
    rows.extend(source_rows.into_iter().map(|row| {
        VcfRenderedCommandRow {
            stage_id: row.stage_id,
            tool_id: row.tool_id,
            readiness_kind: "benchmark_ready".to_string(),
            benchmark_status: row.benchmark_status,
            command_source: "vcf_descent_family_adapter".to_string(),
            command_steps: row
                .command_steps
                .into_iter()
                .map(|step| VcfRenderedCommandStep {
                    step_id: step.step_id,
                    step_kind: step.step_kind,
                    consumes_previous_stdout: false,
                    command: render_shell_command(&step.argv),
                    argv: step.argv,
                })
                .collect(),
            script_commands: Vec::new(),
            reason: row.reason,
        }
    }));
}
