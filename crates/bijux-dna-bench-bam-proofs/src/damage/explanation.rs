use super::declarations::EXPECTED_STAGE_ID;
use bijux_dna_bench_bam_evidence::readiness::damage_authenticity::BamDamageAuthenticityReadyRow;
// Each condition names an independent scientific evidence surface.
#[derive(Clone, Copy)]
#[allow(clippy::struct_excessive_bools)]
pub(super) struct SurfaceChecks {
    pub summary_ready: bool,
    pub unified_metrics_ready: bool,
    pub parser_output_contract_ready: bool,
    pub normalized_metrics_ready: bool,
    pub damage_metric_consistency_ready: bool,
}
pub(super) fn explain(
    damage_row: &BamDamageAuthenticityReadyRow,
    checks: SurfaceChecks,
) -> (Vec<String>, String, String) {
    let SurfaceChecks {
        summary_ready,
        unified_metrics_ready,
        parser_output_contract_ready,
        normalized_metrics_ready,
        damage_metric_consistency_ready,
    } = checks;
    let mut missing_surfaces = damage_row.missing_surfaces.clone();
    if !summary_ready {
        missing_surfaces.push("damage_summary_contract".to_string());
    }
    if !unified_metrics_ready {
        missing_surfaces.push("damage_unified_metrics".to_string());
    }
    if !parser_output_contract_ready {
        missing_surfaces.push("damage_parser_output_contract".to_string());
    }
    if !normalized_metrics_ready {
        missing_surfaces.push("normalized_damage_metrics".to_string());
    }
    if !damage_metric_consistency_ready {
        missing_surfaces.push("damage_metric_consistency".to_string());
    }

    let coverage_status = if damage_row.coverage_status == "complete" && missing_surfaces.is_empty()
    {
        "complete".to_string()
    } else {
        "incomplete".to_string()
    };
    let reason = if coverage_status == "complete" {
        format!(
            "binding `{EXPECTED_STAGE_ID}` / `{}` keeps active scope, command, output, parser, expected-result, report, schema, local-smoke, summary, unified metrics, parser-output contract, and normalized damage metrics complete",
            damage_row.tool_id
        )
    } else {
        format!(
            "binding `{EXPECTED_STAGE_ID}` / `{}` is missing readiness proof for {}",
            damage_row.tool_id,
            missing_surfaces.join(", ")
        )
    };

    (missing_surfaces, coverage_status, reason)
}
