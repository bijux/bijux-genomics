use super::models::FastqReportMapRow;
use crate::readiness::declarations::observations::InventoryReadinessKind as LocalStageReadinessKind;
use bijux_dna_domain_fastq::stage_semantics::FastqStageKind;
use bijux_dna_domain_fastq::StageCriticality;
use std::fmt::Write as _;
pub(super) fn render_fastq_report_map_tsv(rows: &[FastqReportMapRow]) -> String {
    let mut rendered = String::from(
        "result_row_id\tstage_id\ttool_id\tcorpus_family_id\tfixture_id\tsample_scope\tcanonical_stage_rank\treadiness_kind\tstage_kind\tcriticality\tsupport_status\treport_section_id\treport_section_title\tsummary_table_id\tsummary_table_title\tmetric_classes\tmutates_fastq\tproduces_reports_only\treport_focus\treason\n",
    );
    for row in rows {
        let _ = writeln!(
            rendered,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            sanitize_tsv(&row.result_row_id),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.corpus_family_id),
            sanitize_tsv(&row.fixture_id),
            sanitize_tsv(&row.sample_scope),
            row.canonical_stage_rank,
            sanitize_tsv(&row.readiness_kind),
            sanitize_tsv(&row.stage_kind),
            sanitize_tsv(&row.criticality),
            sanitize_tsv(&row.support_status),
            sanitize_tsv(&row.report_section_id),
            sanitize_tsv(&row.report_section_title),
            sanitize_tsv(&row.summary_table_id),
            sanitize_tsv(&row.summary_table_title),
            sanitize_tsv(&row.metric_classes.join(",")),
            row.mutates_fastq,
            row.produces_reports_only,
            sanitize_tsv(&row.report_focus),
            sanitize_tsv(&row.reason)
        );
    }
    rendered
}

pub(super) fn sanitize_tsv(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

pub(super) fn readiness_kind_label(kind: LocalStageReadinessKind) -> &'static str {
    match kind {
        LocalStageReadinessKind::DryRun => "dry_run",
        LocalStageReadinessKind::Smoke => "smoke",
        LocalStageReadinessKind::DryOrSmoke => "dry_or_smoke",
    }
}

pub(super) fn stage_kind_label(kind: FastqStageKind) -> &'static str {
    match kind {
        FastqStageKind::Core => "core",
        FastqStageKind::Optional => "optional",
        FastqStageKind::Meta => "meta",
        FastqStageKind::Amplicon => "amplicon",
    }
}

pub(super) fn criticality_label(criticality: StageCriticality) -> &'static str {
    match criticality {
        StageCriticality::Essential => "essential",
        StageCriticality::Optional => "optional",
        StageCriticality::Experimental => "experimental",
    }
}

pub(super) fn metric_class_label(
    metric_class: bijux_dna_domain_fastq::metrics::spec::MetricClass,
) -> &'static str {
    match metric_class {
        bijux_dna_domain_fastq::metrics::spec::MetricClass::Integrity => "integrity",
        bijux_dna_domain_fastq::metrics::spec::MetricClass::Retention => "retention",
        bijux_dna_domain_fastq::metrics::spec::MetricClass::QualityShift => "quality_shift",
        bijux_dna_domain_fastq::metrics::spec::MetricClass::Contamination => "contamination",
        bijux_dna_domain_fastq::metrics::spec::MetricClass::Composition => "composition",
    }
}
