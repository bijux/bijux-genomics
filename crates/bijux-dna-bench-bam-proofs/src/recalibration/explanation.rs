use bijux_dna_bench_bam_evidence::readiness::recalibration_genotyping::BamRecalibrationGenotypingReadyRow;
// Each condition names an independent scientific evidence surface.
#[derive(Clone, Copy)]
#[allow(clippy::struct_excessive_bools)]
pub(super) struct SurfaceChecks {
    pub local_smoke_ready: bool,
    pub summary_ready: bool,
    pub stage_metrics_ready: bool,
    pub skip_behavior_ready: bool,
    pub known_sites_identity_ready: bool,
    pub recalibration_report_ready: bool,
}
pub(super) fn explain(
    readiness_row: &BamRecalibrationGenotypingReadyRow,
    checks: SurfaceChecks,
) -> (Vec<String>, String, String) {
    let SurfaceChecks {
        local_smoke_ready,
        summary_ready,
        stage_metrics_ready,
        skip_behavior_ready,
        known_sites_identity_ready,
        recalibration_report_ready,
    } = checks;
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
    if !summary_ready {
        missing_surfaces.push("summary".to_string());
    }
    if !stage_metrics_ready {
        missing_surfaces.push("stage_metrics".to_string());
    }
    if !skip_behavior_ready {
        missing_surfaces.push("skip_behavior".to_string());
    }
    if !known_sites_identity_ready {
        missing_surfaces.push("known_sites_identity".to_string());
    }
    if !recalibration_report_ready {
        missing_surfaces.push("recalibration_report".to_string());
    }

    let coverage_status =
        if missing_surfaces.is_empty() { "complete".to_string() } else { "incomplete".to_string() };
    let reason = if missing_surfaces.is_empty() {
        format!(
            "binding `{}` / `{}` keeps retained readiness plus deterministic recalibration skip proof",
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
