use bijux_dna_bench_bam_evidence::readiness::contamination_sex_haplogroups::BamContaminationSexHaplogroupsReadyRow;
// Each condition names an independent scientific evidence surface.
#[derive(Clone, Copy)]
#[allow(clippy::struct_excessive_bools)]
pub(super) struct SurfaceChecks {
    pub local_smoke_ready: bool,
    pub ready_case_ready: bool,
    pub parser_contract_ready: bool,
    pub ready_summary_ready: bool,
    pub ready_stage_metrics_ready: bool,
    pub insufficient_summary_ready: bool,
    pub insufficient_stage_metrics_ready: bool,
    pub insufficiency_behavior_ready: bool,
    pub tool_specific_artifact_ready: bool,
}
pub(super) fn explain(
    readiness_row: &BamContaminationSexHaplogroupsReadyRow,
    checks: &SurfaceChecks,
) -> (Vec<String>, String, String) {
    let SurfaceChecks {
        local_smoke_ready,
        ready_case_ready,
        parser_contract_ready,
        ready_summary_ready,
        ready_stage_metrics_ready,
        insufficient_summary_ready,
        insufficient_stage_metrics_ready,
        insufficiency_behavior_ready,
        tool_specific_artifact_ready,
    } = *checks;
    let mut missing_surfaces = Vec::new();
    if !readiness_row.active_scope_ready {
        missing_surfaces.push("active_scope".to_string());
    }
    if !readiness_row.command_ready {
        missing_surfaces.push("command".to_string());
    }
    if !readiness_row.output_ready {
        missing_surfaces.push("output".to_string());
    }
    if !readiness_row.parser_ready {
        missing_surfaces.push("parser".to_string());
    }
    if !readiness_row.expected_result_ready {
        missing_surfaces.push("expected_result".to_string());
    }
    if !readiness_row.report_ready {
        missing_surfaces.push("report".to_string());
    }
    if !readiness_row.schema_ready {
        missing_surfaces.push("schema".to_string());
    }
    if !local_smoke_ready {
        missing_surfaces.push("local_smoke".to_string());
    }
    if !ready_case_ready {
        missing_surfaces.push("ready_case".to_string());
    }
    if !parser_contract_ready {
        missing_surfaces.push("parser_contract".to_string());
    }
    if !ready_summary_ready {
        missing_surfaces.push("ready_summary".to_string());
    }
    if !ready_stage_metrics_ready {
        missing_surfaces.push("ready_stage_metrics".to_string());
    }
    if !insufficient_summary_ready {
        missing_surfaces.push("insufficient_summary".to_string());
    }
    if !insufficient_stage_metrics_ready {
        missing_surfaces.push("insufficient_stage_metrics".to_string());
    }
    if !insufficiency_behavior_ready {
        missing_surfaces.push("insufficiency_behavior".to_string());
    }
    if !tool_specific_artifact_ready {
        missing_surfaces.push("tool_specific_artifact".to_string());
    }

    let coverage_status =
        if missing_surfaces.is_empty() { "complete".to_string() } else { "incomplete".to_string() };
    let reason = if missing_surfaces.is_empty() {
        format!(
            "binding `{}` / `{}` keeps retained readiness plus ready and insufficient sex smoke proofs",
            readiness_row.stage_id, readiness_row.tool_id
        )
    } else {
        format!(
            "binding `{}` / `{}` is missing completion proof for {}",
            readiness_row.stage_id,
            readiness_row.tool_id,
            missing_surfaces.join(", ")
        )
    };

    (missing_surfaces, coverage_status, reason)
}
