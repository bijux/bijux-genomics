use bijux_dna_domain_fastq::params::correct::{FastqCorrectParams, CORRECT_SCHEMA_VERSION};
use bijux_dna_domain_fastq::params::defaults::{correct_defaults, stats_defaults, umi_defaults};
use bijux_dna_domain_fastq::params::stats::{FastqStatsParams, STATS_SCHEMA_VERSION};
use bijux_dna_domain_fastq::params::umi::{FastqUmiParams, UMI_SCHEMA_VERSION};

fn roundtrip<T>(value: &T) -> T
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    let json = serde_json::to_value(value).unwrap_or_else(|err| panic!("to_value failed: {err}"));
    serde_json::from_value(json).unwrap_or_else(|err| panic!("from_value failed: {err}"))
}

#[test]
fn stats_params_roundtrip_and_schema_version() {
    let params = stats_defaults(true);
    let decoded: FastqStatsParams = roundtrip(&params);
    assert_eq!(decoded.schema_version, STATS_SCHEMA_VERSION);
    assert!(decoded.missing_required_fields().is_empty());
}

#[test]
fn correct_params_roundtrip_and_schema_version() {
    let params = correct_defaults(true);
    let decoded: FastqCorrectParams = roundtrip(&params);
    assert_eq!(decoded.schema_version, CORRECT_SCHEMA_VERSION);
    assert!(decoded.missing_required_fields().is_empty());
}

#[test]
fn umi_params_roundtrip_and_schema_version() {
    let params = umi_defaults(true);
    let decoded: FastqUmiParams = roundtrip(&params);
    assert_eq!(decoded.schema_version, UMI_SCHEMA_VERSION);
    assert!(decoded.missing_required_fields().is_empty());
}
