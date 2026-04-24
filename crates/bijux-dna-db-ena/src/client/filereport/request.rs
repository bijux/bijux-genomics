use crate::model::EnaResultKind;

const ENA_API_BASE: &str = "https://www.ebi.ac.uk/ena/portal/api/filereport";

#[must_use]
pub(crate) fn build_filereport_url(accession: &str, result: EnaResultKind) -> String {
    let fields = filereport_fields(result).join(",");

    format!(
        "{ENA_API_BASE}?accession={accession}&result={}&fields={fields}&format=tsv&download=true&limit=0",
        result.as_api_value()
    )
}

pub(crate) fn filereport_fields(result: EnaResultKind) -> &'static [&'static str] {
    match result {
        EnaResultKind::ReadRun => &[
            "study_accession",
            "sample_accession",
            "experiment_accession",
            "run_accession",
            "tax_id",
            "scientific_name",
            "library_layout",
            "library_source",
            "library_strategy",
            "instrument_model",
            "base_count",
            "read_count",
            "fastq_bytes",
            "fastq_ftp",
            "submitted_ftp",
            "sra_ftp",
        ],
        EnaResultKind::Analysis => &[
            "study_accession",
            "sample_accession",
            "experiment_accession",
            "analysis_accession",
            "analysis_type",
            "tax_id",
            "scientific_name",
            "submitted_ftp",
            "bam_ftp",
        ],
    }
}
