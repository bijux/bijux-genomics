#[must_use]
pub fn artifact_kind_schema(role: &str) -> (&'static str, &'static str) {
    match role.trim() {
        "reads" | "trimmed_reads" => ("fastq", "bijux.artifact.fastq.v1"),
        "bam" | "dedup_bam" => ("bam", "bijux.artifact.bam.v1"),
        "report_json" | "metrics_json" | "summary_json" => {
            ("json", "bijux.artifact.report_json.v1")
        }
        "summary_tsv" => ("tsv", "bijux.artifact.summary_tsv.v1"),
        "report_html" => ("html", "bijux.artifact.report_html.v1"),
        "log" => ("log", "bijux.artifact.log.v1"),
        "index" => ("index", "bijux.artifact.index.v1"),
        "metrics_envelope" => ("json", "bijux.metrics.envelope.v1"),
        "stage_report" => ("json", "bijux.stage_report.v1"),
        _ => ("file", "bijux.artifact.file.v1"),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn artifact_kind_schema_trims_role_before_lookup() {
        assert_eq!(
            super::artifact_kind_schema(" trimmed_reads "),
            ("fastq", "bijux.artifact.fastq.v1")
        );
    }
}
