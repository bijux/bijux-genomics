use super::ExpectedBenchmarkResultRow;
use crate::publication::sanitize_tsv;
use std::fmt::Write;
pub(super) fn render_expected_benchmark_results_tsv(rows: &[ExpectedBenchmarkResultRow]) -> String {
    let mut rendered = String::from(
        "result_row_id\tdomain\tstage_id\ttool_id\treadiness_kind\tcorpus_family_id\tfixture_id\tsample_scope\texpected_output_artifact_ids\traw_output_artifact_ids\tnormalized_metrics_output_id\tresult_root\tstage_result_manifest_path\tstdout_path\tstderr_path\treason\n",
    );
    for row in rows {
        let _ = writeln!(
            rendered,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            sanitize_tsv(&row.result_row_id),
            sanitize_tsv(&row.domain),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.readiness_kind),
            sanitize_tsv(&row.corpus_family_id),
            sanitize_tsv(&row.fixture_id),
            sanitize_tsv(&row.sample_scope),
            sanitize_tsv(&row.expected_output_artifact_ids.join(",")),
            sanitize_tsv(&row.raw_output_artifact_ids.join(",")),
            sanitize_tsv(row.normalized_metrics_output_id.as_deref().unwrap_or("")),
            sanitize_tsv(&row.result_root),
            sanitize_tsv(&row.stage_result_manifest_path),
            sanitize_tsv(&row.stdout_path),
            sanitize_tsv(&row.stderr_path),
            sanitize_tsv(&row.reason),
        );
    }
    rendered
}
