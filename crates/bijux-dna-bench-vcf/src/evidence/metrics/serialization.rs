use bijux_dna_bench_model::VcfComparableMetricsRow;
use std::fmt::Write;
pub(super) fn render_vcf_comparable_metrics_tsv(rows: &[VcfComparableMetricsRow]) -> String {
    let mut rendered = String::from(
        "stage_id\tmetric_id\tmetric_name\tunit\tdirection\trequired\ttools_covered\n",
    );
    for row in rows {
        let _ = writeln!(
            rendered,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}",
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.metric_id),
            sanitize_tsv(&row.metric_name),
            sanitize_tsv(&row.unit),
            sanitize_tsv(row.direction.as_str()),
            row.required,
            sanitize_tsv(&row.tools_covered.join(",")),
        );
    }
    rendered
}
fn sanitize_tsv(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}
