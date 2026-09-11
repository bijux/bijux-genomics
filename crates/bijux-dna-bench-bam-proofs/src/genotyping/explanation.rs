use bijux_dna_bench_bam_evidence::readiness::recalibration_genotyping::BamRecalibrationGenotypingReadyRow;
// Each flag retains an independent scientific evidence surface.
#[derive(Clone, Copy)]
#[allow(clippy::struct_excessive_bools)]
pub(super) struct SurfaceChecks {
    pub local_plan_ready: bool,
    pub producer_contract_ready: bool,
    pub parser_smoke_ready: bool,
    pub bam_pipeline_ready: bool,
    pub cross_pipeline_ready: bool,
    pub downstream_handoff_ready: bool,
    pub downstream_variant_metrics_ready: bool,
    pub downstream_missingness_ready: bool,
}
pub(super) fn explain(
    readiness_row: &BamRecalibrationGenotypingReadyRow,
    checks: SurfaceChecks,
) -> (Vec<String>, &'static str, String) {
    let SurfaceChecks {
        local_plan_ready,
        producer_contract_ready,
        parser_smoke_ready,
        bam_pipeline_ready,
        cross_pipeline_ready,
        downstream_handoff_ready,
        downstream_variant_metrics_ready,
        downstream_missingness_ready,
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
    if !local_plan_ready {
        missing_surfaces.push("local_plan".to_string());
    }
    if !producer_contract_ready {
        missing_surfaces.push("producer_contract".to_string());
    }
    if !parser_smoke_ready {
        missing_surfaces.push("parser_smoke".to_string());
    }
    if !bam_pipeline_ready {
        missing_surfaces.push("bam_pipeline".to_string());
    }
    if !cross_pipeline_ready {
        missing_surfaces.push("cross_pipeline".to_string());
    }
    if !downstream_handoff_ready {
        missing_surfaces.push("downstream_handoff".to_string());
    }
    if !downstream_variant_metrics_ready {
        missing_surfaces.push("downstream_variant_metrics".to_string());
    }
    if !downstream_missingness_ready {
        missing_surfaces.push("downstream_missingness".to_string());
    }
    let coverage_status = if missing_surfaces.is_empty() { "complete" } else { "incomplete" };
    let reason = if missing_surfaces.is_empty() {
        "binding `bam.genotyping` / `bcftools` keeps local-ready BCF/VCF/GL handoff, parser-smoke call-rate proof, and downstream VCF metrics compatibility complete".to_string()
    } else {
        format!(
            "binding `bam.genotyping` / `bcftools` is missing {} required completion surface(s): {}",
            missing_surfaces.len(),
            missing_surfaces.join(", ")
        )
    };

    (missing_surfaces, coverage_status, reason)
}
