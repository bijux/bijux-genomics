use super::models::FastqAdapterOutputContractRow;
use super::models::FastqAdapterOutputContractStatus;
use std::fmt::Write as _;
pub(super) fn render_fastq_adapter_output_contract_tsv(
    rows: &[FastqAdapterOutputContractRow],
) -> String {
    let mut rendered = String::from(
        "tool_id\tstage_id\tadapter_status\toutput_contract_status\tstage_output_ids\tstage_expected_artifact_ids\tdeclared_output_ids\texecution_expected_output_ids\traw_output_artifact_ids\tnormalized_metrics_output_id\tstdout_path_template\tstderr_path_template\tstage_result_manifest_path_template\tmissing_declarations\treason\n",
    );
    for row in rows {
        let _ = writeln!(
            rendered,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.adapter_status),
            sanitize_tsv(output_contract_status_label(row.output_contract_status)),
            sanitize_tsv(&row.stage_output_ids.join(",")),
            sanitize_tsv(&row.stage_expected_artifact_ids.join(",")),
            sanitize_tsv(&row.declared_output_ids.join(",")),
            sanitize_tsv(&row.execution_expected_output_ids.join(",")),
            sanitize_tsv(&row.raw_output_artifact_ids.join(",")),
            sanitize_tsv(row.normalized_metrics_output_id.as_deref().unwrap_or("")),
            sanitize_tsv(row.stdout_path_template.as_deref().unwrap_or("")),
            sanitize_tsv(row.stderr_path_template.as_deref().unwrap_or("")),
            sanitize_tsv(row.stage_result_manifest_path_template.as_deref().unwrap_or("")),
            sanitize_tsv(&row.missing_declarations.join(",")),
            sanitize_tsv(&row.reason)
        );
    }
    rendered
}

/// Label canonical FASTQ declaration evidence.
#[must_use]
pub fn output_contract_status_label(status: FastqAdapterOutputContractStatus) -> &'static str {
    match status {
        FastqAdapterOutputContractStatus::Complete => "complete",
        FastqAdapterOutputContractStatus::Incomplete => "incomplete",
        FastqAdapterOutputContractStatus::MissingAdapter => "missing_adapter",
    }
}

pub(super) fn sanitize_tsv(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}
