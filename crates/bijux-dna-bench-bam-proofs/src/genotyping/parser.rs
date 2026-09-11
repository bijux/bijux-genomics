use crate::downstream::observations::ParserObservation;
pub(super) struct ParserSnapshot {
    pub call_rate: f64,
    pub mean_posterior: f64,
    pub reference_fasta: String,
    pub sites_vcf: String,
}
pub(super) fn observe_parser(parser_row: &ParserObservation) -> ParserSnapshot {
    let parser_smoke_call_rate = parser_row
        .normalized_snapshot
        .get("call_rate")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    let parser_smoke_mean_posterior = parser_row
        .normalized_snapshot
        .get("mean_posterior")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    let parser_smoke_reference_fasta = parser_row
        .normalized_snapshot
        .get("reference")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    let parser_smoke_sites_vcf = parser_row
        .normalized_snapshot
        .get("sites")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    ParserSnapshot {
        call_rate: parser_smoke_call_rate,
        mean_posterior: parser_smoke_mean_posterior,
        reference_fasta: parser_smoke_reference_fasta,
        sites_vcf: parser_smoke_sites_vcf,
    }
}
