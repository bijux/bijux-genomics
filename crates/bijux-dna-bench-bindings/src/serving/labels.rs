use crate::observations::{BamStageAssessment, FastqAssessment};
pub(super) fn adapter_status_label(capability: &FastqAssessment) -> &'static str {
    if capability.runnable {
        "runnable"
    } else if capability.plannable {
        "plannable"
    } else if capability.declared {
        "declared_only"
    } else {
        "missing"
    }
}
pub(super) fn parser_status_label(capability: &FastqAssessment) -> &'static str {
    if capability.comparable {
        "comparable"
    } else if capability.benchmark_normalized {
        "benchmark_normalized"
    } else if capability.parse_normalized {
        "parse_normalized"
    } else {
        "not_normalized"
    }
}
pub(super) fn bam_parser_status_label(stage: &BamStageAssessment) -> &'static str {
    if stage.has_parser_fixtures {
        "parser_fixture_validated"
    } else if stage.has_scientific_report_contract {
        "scientific_report_contract"
    } else if stage.has_artifact_contract {
        "artifact_contract_only"
    } else {
        "missing"
    }
}
