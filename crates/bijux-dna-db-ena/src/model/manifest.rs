use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use super::{EnaFileSource, EnaQuery, EnaRecord, EnaSourcePreference};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnaRunManifest {
    pub query: EnaQuery,
    pub source: EnaFileSource,
    pub preference: EnaSourcePreference,
    pub records: Vec<EnaRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnaWorkflowRun {
    pub run_accession: String,
    pub sample_accession: Option<String>,
    pub read_layout: String,
    pub fastq_urls: Vec<String>,
    pub fastq_sha256: Vec<Option<String>>,
    pub uncertainty: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnaWorkflowManifest {
    pub schema_version: String,
    pub runs: Vec<EnaWorkflowRun>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnaOfflineFixture {
    pub schema_version: String,
    pub runs: Vec<EnaRecord>,
}

/// # Errors
/// Returns an error if records cannot be converted into a deterministic workflow manifest.
pub fn build_workflow_manifest(records: &[EnaRecord]) -> Result<EnaWorkflowManifest> {
    let mut runs = Vec::with_capacity(records.len());
    for record in records {
        let run_accession = record
            .run_accession
            .clone()
            .or_else(|| record.analysis_accession.clone())
            .or_else(|| record.experiment_accession.clone())
            .ok_or_else(|| anyhow::anyhow!("record missing run/analysis/experiment accession"))?;
        let read_layout = record
            .library_layout
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| infer_layout(record.fastq_ftp.len()).to_string());
        let mut uncertainty = Vec::new();
        if record.fastq_ftp.is_empty() {
            uncertainty.push("missing_fastq_urls".to_string());
        }
        let expected_fastq = expected_fastq_count(&read_layout);
        if let Some(expected) = expected_fastq {
            if record.fastq_ftp.len() != expected {
                uncertainty.push("layout_fastq_count_mismatch".to_string());
            }
        } else {
            uncertainty.push("unknown_layout".to_string());
        }
        let checksum_count = expected_fastq.unwrap_or(record.fastq_ftp.len());
        let fastq_sha256 = vec![None; checksum_count];
        if !fastq_sha256.is_empty() && fastq_sha256.iter().all(Option::is_none) {
            uncertainty.push("missing_fastq_sha256".to_string());
        }
        if record.sample_accession.is_none() {
            uncertainty.push("missing_sample_accession".to_string());
        }
        runs.push(EnaWorkflowRun {
            run_accession,
            sample_accession: record.sample_accession.clone(),
            read_layout,
            fastq_urls: record.fastq_ftp.clone(),
            fastq_sha256,
            uncertainty,
        });
    }
    Ok(EnaWorkflowManifest {
        schema_version: "bijux.ena_workflow_manifest.v1".to_string(),
        runs,
    })
}

/// # Errors
/// Returns an error if the fixture is malformed or does not use the supported schema version.
pub fn build_workflow_manifest_from_offline_fixture(raw: &str) -> Result<EnaWorkflowManifest> {
    let fixture: EnaOfflineFixture = serde_json::from_str(raw)?;
    if fixture.schema_version != "bijux.ena.offline_fixture.v1" {
        bail!(
            "unsupported ENA offline fixture schema `{}`",
            fixture.schema_version
        );
    }
    build_workflow_manifest(&fixture.runs)
}

fn infer_layout(fastq_count: usize) -> &'static str {
    match fastq_count {
        1 => "SINGLE",
        2 => "PAIRED",
        _ => "UNKNOWN",
    }
}

fn expected_fastq_count(read_layout: &str) -> Option<usize> {
    match read_layout.trim().to_ascii_uppercase().as_str() {
        "SINGLE" => Some(1),
        "PAIRED" => Some(2),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{build_workflow_manifest, build_workflow_manifest_from_offline_fixture};
    use crate::model::EnaRecord;

    #[test]
    fn workflow_manifest_bridge_preserves_run_sample_layout_and_uncertainty() {
        let manifest = build_workflow_manifest(&[EnaRecord {
            study_accession: Some("PRJX".to_string()),
            sample_accession: Some("SAMEA1".to_string()),
            experiment_accession: Some("ERX1".to_string()),
            run_accession: Some("ERR1".to_string()),
            analysis_accession: None,
            tax_id: None,
            scientific_name: None,
            library_layout: Some("PAIRED".to_string()),
            library_source: None,
            library_strategy: None,
            instrument_model: None,
            base_count: None,
            read_count: None,
            fastq_bytes: vec![],
            fastq_ftp: vec![
                "ftp.sra.ebi.ac.uk/vol1/a_1.fastq.gz".to_string(),
                "ftp.sra.ebi.ac.uk/vol1/a_2.fastq.gz".to_string(),
            ],
            submitted_ftp: vec![],
            sra_ftp: vec![],
            bam_ftp: vec![],
        }])
        .unwrap_or_else(|error| panic!("build workflow manifest: {error}"));

        assert_eq!(manifest.schema_version, "bijux.ena_workflow_manifest.v1");
        assert_eq!(manifest.runs.len(), 1);
        assert_eq!(manifest.runs[0].run_accession, "ERR1");
        assert!(manifest.runs[0].uncertainty.contains(&"missing_fastq_sha256".to_string()));
    }

    #[test]
    fn offline_fixture_mode_supports_mixed_layout_and_inconsistent_cases() {
        let fixture = serde_json::json!({
            "schema_version": "bijux.ena.offline_fixture.v1",
            "runs": [
                {
                    "study_accession": "PRJX",
                    "sample_accession": "SAMEA1",
                    "experiment_accession": "ERX1",
                    "run_accession": "ERR_SINGLE",
                    "analysis_accession": null,
                    "tax_id": "9606",
                    "scientific_name": "Homo sapiens",
                    "library_layout": "SINGLE",
                    "library_source": "GENOMIC",
                    "library_strategy": "WGS",
                    "instrument_model": "NovaSeq",
                    "base_count": 100,
                    "read_count": 10,
                    "fastq_bytes": [42],
                    "fastq_ftp": ["ftp.sra.ebi.ac.uk/vol1/single.fastq.gz"],
                    "submitted_ftp": [],
                    "sra_ftp": [],
                    "bam_ftp": []
                },
                {
                    "study_accession": "PRJX",
                    "sample_accession": null,
                    "experiment_accession": "ERX2",
                    "run_accession": "ERR_INCONSISTENT",
                    "analysis_accession": null,
                    "tax_id": "9606",
                    "scientific_name": "Homo sapiens",
                    "library_layout": "PAIRED",
                    "library_source": "GENOMIC",
                    "library_strategy": "WGS",
                    "instrument_model": "NovaSeq",
                    "base_count": 100,
                    "read_count": 10,
                    "fastq_bytes": [42],
                    "fastq_ftp": ["ftp.sra.ebi.ac.uk/vol1/inconsistent.fastq.gz"],
                    "submitted_ftp": [],
                    "sra_ftp": [],
                    "bam_ftp": []
                }
            ]
        });
        let manifest = build_workflow_manifest_from_offline_fixture(&fixture.to_string())
            .unwrap_or_else(|error| panic!("build manifest from offline fixture: {error}"));
        assert_eq!(manifest.runs.len(), 2);
        assert!(
            manifest.runs[1]
                .uncertainty
                .contains(&"layout_fastq_count_mismatch".to_string())
        );
        assert!(
            manifest.runs[1]
                .uncertainty
                .contains(&"missing_sample_accession".to_string())
        );
    }
}
