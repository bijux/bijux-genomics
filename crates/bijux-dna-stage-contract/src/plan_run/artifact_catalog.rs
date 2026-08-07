#[must_use]
pub fn artifact_kind_schema(role: &str) -> (&'static str, &'static str) {
    match role.trim() {
        "reads" | "trimmed_reads" => ("fastq", "bijux.fastq.artifact.reads.v1"),
        "bam" | "dedup_bam" | "alignment" => {
            ("bam", "bijux.bam.artifact.coordinate_sorted_indexed.v1")
        }
        "variant" => ("vcf", "bijux.vcf.artifact.coordinate_sorted_indexed.v1"),
        "reference" => ("reference", "bijux.reference.artifact.fasta.v1"),
        "report_json" | "metrics_json" | "summary_json" => {
            ("json", "bijux.auxiliary.artifact.json.v1")
        }
        "summary_tsv" => ("tsv", "bijux.auxiliary.artifact.table.v1"),
        "report_html" => ("html", "bijux.auxiliary.artifact.html.v1"),
        "log" => ("log", "bijux.auxiliary.artifact.text.v1"),
        "index" => ("index", "bijux.auxiliary.artifact.index.v1"),
        "metrics_envelope" => ("json", "bijux.auxiliary.artifact.json.v1"),
        "stage_report" => ("file", "bijux.auxiliary.artifact.file.v1"),
        _ => ("file", "bijux.auxiliary.artifact.file.v1"),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn artifact_kind_schema_trims_role_before_lookup() {
        assert_eq!(
            super::artifact_kind_schema(" trimmed_reads "),
            ("fastq", "bijux.fastq.artifact.reads.v1")
        );
    }

    #[test]
    fn artifact_kind_schema_never_returns_legacy_contract_ids() {
        for role in ["report_html", "log", "index", "metrics_envelope", "stage_report", "unknown"] {
            let (_, contract_id) = super::artifact_kind_schema(role);
            assert!(
                crate::canonical_artifact_contract_catalog().contract(contract_id).is_some(),
                "{role} resolved to unknown artifact contract {contract_id}"
            );
        }
    }
}
