use super::models::FastqReportMapRow;
use super::models::RequiredReportMapping;
use crate::readiness::declarations::observations::ExpectedResultBinding as ExpectedBenchmarkResultRow;
use anyhow::anyhow;
use anyhow::Result;
use bijux_dna_core::id_catalog::{
    FASTQ_ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN, FASTQ_INDEX_REFERENCE, FASTQ_INFER_ASVS,
    FASTQ_SCREEN, FASTQ_STATS_NEUTRAL, FASTQ_TRIM, FASTQ_VALIDATE_READS,
};
use std::collections::BTreeSet;
pub(super) fn ensure_fastq_report_map_contract(
    rows: &[FastqReportMapRow],
    expected_rows: &[ExpectedBenchmarkResultRow],
) -> Result<()> {
    let report_result_ids =
        rows.iter().map(|row| row.result_row_id.as_str()).collect::<BTreeSet<_>>();
    if report_result_ids.len() != rows.len() {
        return Err(anyhow!("FASTQ report map must keep one row per expected FASTQ result row"));
    }

    let expected_result_ids =
        expected_rows.iter().map(|row| row.result_row_id.as_str()).collect::<BTreeSet<_>>();
    if report_result_ids != expected_result_ids {
        return Err(anyhow!(
            "FASTQ report map must retain every FASTQ expected-result row; expected {} rows and found {} report rows",
            expected_rows.len(),
            rows.len()
        ));
    }

    let report_keys = rows.iter().map(report_result_key).collect::<BTreeSet<_>>();
    let expected_keys = expected_rows.iter().map(expected_result_key).collect::<BTreeSet<_>>();
    if report_keys != expected_keys {
        return Err(anyhow!(
            "FASTQ report map drifted from the governed FASTQ expected-result identity slice"
        ));
    }

    for row in rows {
        if row.result_row_id.trim().is_empty()
            || row.tool_id.trim().is_empty()
            || row.corpus_family_id.trim().is_empty()
            || row.fixture_id.trim().is_empty()
            || row.sample_scope.trim().is_empty()
            || row.support_status.trim().is_empty()
            || row.report_section_id.trim().is_empty()
            || row.summary_table_id.trim().is_empty()
        {
            return Err(anyhow!(
                "FASTQ report map row `{}` / `{}` is missing required report binding fields",
                row.stage_id,
                row.tool_id
            ));
        }
    }

    let section_count =
        rows.iter().map(|row| row.report_section_id.as_str()).collect::<BTreeSet<_>>().len();
    let summary_table_count =
        rows.iter().map(|row| row.summary_table_id.as_str()).collect::<BTreeSet<_>>().len();
    if section_count != 6 || summary_table_count != 7 {
        return Err(anyhow!(
            "FASTQ report map must retain 6 sections and 7 summary tables, found {section_count} sections and {summary_table_count} tables"
        ));
    }

    ensure_required_row_mappings(rows)?;

    Ok(())
}

pub(super) fn report_result_key(row: &FastqReportMapRow) -> String {
    format!(
        "{}:{}:{}:{}:{}",
        row.result_row_id, row.stage_id, row.tool_id, row.corpus_family_id, row.fixture_id
    )
}

pub(super) fn expected_result_key(row: &ExpectedBenchmarkResultRow) -> String {
    format!(
        "{}:{}:{}:{}:{}",
        row.result_row_id, row.stage_id, row.tool_id, row.corpus_family_id, row.fixture_id
    )
}

pub(super) fn require_row_mapping(
    rows: &[FastqReportMapRow],
    RequiredReportMapping {
        stage_id,
        tool_id,
        section_id,
        summary_table_id,
        stage_kind,
        criticality,
        support_status,
    }: RequiredReportMapping<'_>,
) -> Result<()> {
    let row = rows
        .iter()
        .find(|row| row.stage_id == stage_id && row.tool_id == tool_id)
        .ok_or_else(|| anyhow!("FASTQ report map is missing `{stage_id}` / `{tool_id}`"))?;
    if row.report_section_id != section_id
        || row.summary_table_id != summary_table_id
        || row.stage_kind != stage_kind
        || row.criticality != criticality
        || row.support_status != support_status
    {
        return Err(anyhow!(
            "FASTQ report map row `{stage_id}` / `{tool_id}` drifted from its governed reporting contract"
        ));
    }
    Ok(())
}

fn ensure_required_row_mappings(rows: &[FastqReportMapRow]) -> Result<()> {
    require_row_mapping(
        rows,
        RequiredReportMapping {
            stage_id: FASTQ_INDEX_REFERENCE,
            tool_id: "bowtie2_build",
            section_id: "reference_preparation",
            summary_table_id: "reference_index_assets",
            stage_kind: "meta",
            criticality: "optional",
            support_status: "supported",
        },
    )?;
    require_row_mapping(
        rows,
        RequiredReportMapping {
            stage_id: FASTQ_VALIDATE_READS,
            tool_id: "fastqvalidator",
            section_id: "input_readiness",
            summary_table_id: "validation_intake",
            stage_kind: "core",
            criticality: "essential",
            support_status: "supported",
        },
    )?;
    require_row_mapping(
        rows,
        RequiredReportMapping {
            stage_id: FASTQ_STATS_NEUTRAL,
            tool_id: "seqkit_stats",
            section_id: "quality_profiling",
            summary_table_id: "qc_signal_profiles",
            stage_kind: "core",
            criticality: "essential",
            support_status: "supported",
        },
    )?;
    require_row_mapping(
        rows,
        RequiredReportMapping {
            stage_id: FASTQ_ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN,
            tool_id: "bijux_dna",
            section_id: "quality_profiling",
            summary_table_id: "premerge_complexity",
            stage_kind: "optional",
            criticality: "optional",
            support_status: "supported",
        },
    )?;
    require_row_mapping(
        rows,
        RequiredReportMapping {
            stage_id: FASTQ_TRIM,
            tool_id: "fastp",
            section_id: "read_cleanup",
            summary_table_id: "cleanup_retention",
            stage_kind: "core",
            criticality: "essential",
            support_status: "supported",
        },
    )?;
    require_row_mapping(
        rows,
        RequiredReportMapping {
            stage_id: FASTQ_SCREEN,
            tool_id: "kraken2",
            section_id: "contamination_screening",
            summary_table_id: "screening_contamination",
            stage_kind: "optional",
            criticality: "optional",
            support_status: "supported",
        },
    )?;
    require_row_mapping(
        rows,
        RequiredReportMapping {
            stage_id: FASTQ_INFER_ASVS,
            tool_id: "dada2",
            section_id: "amplicon_interpretation",
            summary_table_id: "amplicon_features",
            stage_kind: "amplicon",
            criticality: "experimental",
            support_status: "supported",
        },
    )?;

    Ok(())
}
