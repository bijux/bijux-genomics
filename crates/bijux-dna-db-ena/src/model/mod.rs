use serde::{Deserialize, Serialize};

mod record;
mod query;

pub use record::{normalize_url, split_ena_field, split_ena_u64_field, EnaFileSource, EnaRecord, EnaRunManifest};
pub use query::{EnaQuery, EnaResultKind, EnaSourcePreference};

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
            normalize_url(
                "ftp.sra.ebi.ac.uk/vol1/x.fastq.gz",
                EnaSourcePreference::Ftp
            ),
            "ftp://ftp.sra.ebi.ac.uk/vol1/x.fastq.gz"
        );
        assert_eq!(
            normalize_url(
                "ftp.sra.ebi.ac.uk/vol1/x.fastq.gz",
                EnaSourcePreference::Https
            ),
            "https://ftp.sra.ebi.ac.uk/vol1/x.fastq.gz"
        );
    }
}
