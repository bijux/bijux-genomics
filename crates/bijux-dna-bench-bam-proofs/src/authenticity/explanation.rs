use super::declarations::EXPECTED_STAGE_ID;
use bijux_dna_bench_bam_evidence::readiness::damage_authenticity::BamDamageAuthenticityReadyRow;
// Each condition names an independent scientific evidence surface.
#[derive(Clone, Copy)]
#[allow(clippy::struct_excessive_bools)]
pub(super) struct SurfaceChecks {
    pub authenticity_report_contract_ready: bool,
    pub summary_ready: bool,
    pub composition_ready: bool,
    pub normalized_metrics_ready: bool,
    pub evidence_consumption_ready: bool,
}
pub(super) fn explain(
    authenticity_row: &BamDamageAuthenticityReadyRow,
    checks: SurfaceChecks,
) -> (Vec<String>, String, String) {
    let SurfaceChecks {
        authenticity_report_contract_ready,
        summary_ready,
        composition_ready,
        normalized_metrics_ready,
        evidence_consumption_ready,
    } = checks;
    let mut missing_surfaces = authenticity_row.missing_surfaces.clone();
    if !authenticity_report_contract_ready {
        missing_surfaces.push("authenticity_report_contract".to_string());
    }
    if !summary_ready {
        missing_surfaces.push("authenticity_summary_contract".to_string());
    }
    if !composition_ready {
        missing_surfaces.push("authenticity_evidence_composition".to_string());
    }
    if !normalized_metrics_ready {
        missing_surfaces.push("normalized_authenticity_metrics".to_string());
    }
    if !evidence_consumption_ready {
        missing_surfaces.push("authenticity_evidence_paths".to_string());
    }

    let coverage_status =
        if authenticity_row.coverage_status == "complete" && missing_surfaces.is_empty() {
            "complete".to_string()
        } else {
            "incomplete".to_string()
        };
    let reason = if coverage_status == "complete" {
        format!(
            "binding `{EXPECTED_STAGE_ID}` / `{}` keeps active scope, command, output, parser, expected-result, report, schema, local-smoke, advisory summary, evidence composition, normalized metrics, and upstream evidence-path consumption complete",
            authenticity_row.tool_id
        )
    } else {
        format!(
            "binding `{EXPECTED_STAGE_ID}` / `{}` is missing readiness proof for {}",
            authenticity_row.tool_id,
            missing_surfaces.join(", ")
        )
    };

    (missing_surfaces, coverage_status, reason)
}
