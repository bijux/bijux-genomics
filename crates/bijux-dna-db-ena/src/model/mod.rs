mod manifest;
mod query;
mod record;
mod source_selection;

pub use manifest::{
    build_workflow_manifest, build_workflow_manifest_from_offline_fixture, EnaOfflineFixture,
    EnaRunManifest, EnaWorkflowManifest, EnaWorkflowRun,
};
pub use query::{EnaQuery, QueryValidationError};
pub use record::{split_ena_field, split_ena_u64_field, EnaRecord};
pub use source_selection::{normalize_url, EnaFileSource, EnaResultKind, EnaSourcePreference};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_ena_field_handles_empty_and_multi_values() {
        assert_eq!(split_ena_field("a;b; ;c"), vec!["a", "b", "c"]);
    }

    #[test]
    fn normalize_url_adds_expected_scheme() {
        assert_eq!(
            normalize_url("ftp.sra.ebi.ac.uk/vol1/x.fastq.gz", EnaSourcePreference::Ftp),
            "ftp://ftp.sra.ebi.ac.uk/vol1/x.fastq.gz"
        );
        assert_eq!(
            normalize_url("ftp.sra.ebi.ac.uk/vol1/x.fastq.gz", EnaSourcePreference::Https),
            "https://ftp.sra.ebi.ac.uk/vol1/x.fastq.gz"
        );
    }

    #[test]
    fn normalize_url_trims_raw_values_before_scheme_handling() {
        assert_eq!(
            normalize_url(" ftp.sra.ebi.ac.uk/vol1/x.fastq.gz ", EnaSourcePreference::Ftp),
            "ftp://ftp.sra.ebi.ac.uk/vol1/x.fastq.gz"
        );
        assert_eq!(
            normalize_url(" https://example.org/reads.fastq.gz ", EnaSourcePreference::Ftp),
            "https://example.org/reads.fastq.gz"
        );
    }
}
