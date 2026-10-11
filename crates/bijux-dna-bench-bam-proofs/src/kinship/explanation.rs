use bijux_dna_bench_bam_evidence::readiness::kinship::BamKinshipReadyRow;
// Each flag retains an independent scientific evidence surface.
#[derive(Clone, Copy)]
#[allow(clippy::struct_excessive_bools)]
pub(super) struct SurfaceChecks {
    pub local_smoke_ready: bool,
    pub ready_case_report_ready: bool,
    pub ready_case_summary_ready: bool,
    pub ready_case_pairwise_table_ready: bool,
    pub ready_case_stage_metrics_ready: bool,
    pub insufficient_case_report_ready: bool,
    pub insufficient_case_summary_ready: bool,
    pub insufficient_case_stage_metrics_ready: bool,
    pub parser_smoke_ready: bool,
    pub bam_pipeline_ready: bool,
    pub bam_locality_ready: bool,
    pub vcf_isolation_ready: bool,
}
pub(super) fn explain(
    readiness_row: &BamKinshipReadyRow,
    checks: SurfaceChecks,
) -> (Vec<String>, &'static str, String) {
    let SurfaceChecks {
        local_smoke_ready,
        ready_case_report_ready,
        ready_case_summary_ready,
        ready_case_pairwise_table_ready,
        ready_case_stage_metrics_ready,
        insufficient_case_report_ready,
        insufficient_case_summary_ready,
        insufficient_case_stage_metrics_ready,
        parser_smoke_ready,
        bam_pipeline_ready,
        bam_locality_ready,
        vcf_isolation_ready,
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
    if !ready_case_report_ready {
        missing_surfaces.push("ready_case_report".to_string());
    }
    if !ready_case_summary_ready {
        missing_surfaces.push("ready_case_summary".to_string());
    }
    if !ready_case_pairwise_table_ready {
        missing_surfaces.push("ready_case_pairwise_table".to_string());
    }
    if !ready_case_stage_metrics_ready {
        missing_surfaces.push("ready_case_stage_metrics".to_string());
    }
    if !insufficient_case_report_ready {
        missing_surfaces.push("insufficient_case_report".to_string());
    }
    if !insufficient_case_summary_ready {
        missing_surfaces.push("insufficient_case_summary".to_string());
    }
    if !insufficient_case_stage_metrics_ready {
        missing_surfaces.push("insufficient_case_stage_metrics".to_string());
    }
    if !parser_smoke_ready {
        missing_surfaces.push("parser_smoke".to_string());
    }
    if !bam_pipeline_ready {
        missing_surfaces.push("bam_pipeline".to_string());
    }
    if !bam_locality_ready {
        missing_surfaces.push("bam_locality".to_string());
    }
    if !vcf_isolation_ready {
        missing_surfaces.push("vcf_isolation".to_string());
    }
    let coverage_status = if missing_surfaces.is_empty() { "complete" } else { "incomplete" };
    let reason = if missing_surfaces.is_empty() {
        format!(
            "binding `bam.kinship` / `{}` keeps pairwise kinship evidence, explicit insufficient-overlap behavior, parser-smoke proof, and locality across BAM and VCF handoffs complete",
            readiness_row.tool_id
        )
    } else {
        format!(
            "binding `bam.kinship` / `{}` is missing {} required completion surface(s): {}",
            readiness_row.tool_id,
            missing_surfaces.len(),
            missing_surfaces.join(", ")
        )
    };

    (missing_surfaces, coverage_status, reason)
}
