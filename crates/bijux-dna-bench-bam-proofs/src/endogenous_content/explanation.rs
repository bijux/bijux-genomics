use bijux_dna_bench_bam_evidence::readiness::overlap_endogenous::BamOverlapEndogenousReadyRow;
// Each condition names an independent scientific evidence surface.
#[derive(Clone, Copy)]
#[allow(clippy::struct_excessive_bools)]
pub(super) struct SurfaceChecks {
    pub summary_ready: bool,
    pub normalized_metrics_ready: bool,
    pub endogenous_metric_consistency_ready: bool,
}
pub(super) fn explain(
    endogenous_row: &BamOverlapEndogenousReadyRow,
    checks: SurfaceChecks,
) -> (Vec<String>, String, String) {
    let SurfaceChecks {
        summary_ready,
        normalized_metrics_ready,
        endogenous_metric_consistency_ready,
    } = checks;
    let mut missing_surfaces = endogenous_row.missing_surfaces.clone();
    if !summary_ready {
        missing_surfaces.push("endogenous_summary_contract".to_string());
    }
    if !normalized_metrics_ready {
        missing_surfaces.push("normalized_endogenous_metrics".to_string());
    }
    if !endogenous_metric_consistency_ready {
        missing_surfaces.push("endogenous_metric_consistency".to_string());
    }

    let coverage_status =
        if endogenous_row.coverage_status == "complete" && missing_surfaces.is_empty() {
            "complete".to_string()
        } else {
            "incomplete".to_string()
        };
    let reason = if coverage_status == "complete" {
        "binding `bam.endogenous_content` / `samtools` keeps active scope, command, output, parser, expected-result, report, schema, local-smoke, summary, and normalized endogenous metrics complete".to_string()
    } else {
        format!(
            "binding `bam.endogenous_content` / `samtools` is missing readiness proof for {}",
            missing_surfaces.join(", ")
        )
    };

    (missing_surfaces, coverage_status, reason)
}
