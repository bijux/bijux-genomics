use crate::model::{EnaFileSource, EnaRecord, EnaSourcePreference};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

mod planning;
mod transfer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadTask {
    pub project: Option<String>,
    pub sample: Option<String>,
    pub accession: String,
    pub source: EnaFileSource,
    pub url: String,
    pub output: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadConfig {
    pub output_dir: PathBuf,
    pub jobs: usize,
    pub retries: usize,
    pub source: EnaFileSource,
    pub preference: EnaSourcePreference,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadReport {
    pub attempted: usize,
    pub downloaded: usize,
    pub failed: usize,
    pub failed_outputs: Vec<PathBuf>,
}

impl DownloadConfig {
    #[must_use]
    pub fn from_defaults(output_dir: PathBuf) -> Self {
        Self {
            output_dir,
            jobs: 8,
            retries: 2,
            source: EnaFileSource::FastqFtp,
            preference: EnaSourcePreference::Ftp,
            dry_run: false,
        }
    }
}

#[must_use]
pub fn build_download_tasks(records: &[EnaRecord], config: &DownloadConfig) -> Vec<DownloadTask> {
    planning::build_download_tasks(records, config)
}

/// # Errors
/// Returns an error if configuration is invalid, the thread pool cannot be
/// created, or HTTP client initialization fails.
pub fn download_tasks(tasks: &[DownloadTask], config: &DownloadConfig) -> Result<DownloadReport> {
    transfer::download_tasks(tasks, config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::EnaResultKind;

    #[test]
    fn build_download_tasks_materializes_accession_subdirs() {
        let record = EnaRecord {
            study_accession: Some("PRJEB1".to_string()),
            sample_accession: Some("SAMEA1".to_string()),
            experiment_accession: Some("ERX1".to_string()),
            run_accession: Some("ERR1".to_string()),
            analysis_accession: None,
            tax_id: None,
            scientific_name: None,
            library_layout: Some("PAIRED".to_string()),
            library_source: Some("GENOMIC".to_string()),
            library_strategy: Some("WGS".to_string()),
            instrument_model: Some("Illumina".to_string()),
            base_count: Some(1000),
            read_count: Some(100),
            fastq_bytes: vec![10, 10],
            fastq_ftp: vec!["ftp.sra.ebi.ac.uk/vol1/ERR1_1.fastq.gz".to_string()],
            submitted_ftp: Vec::new(),
            sra_ftp: Vec::new(),
            bam_ftp: Vec::new(),
        };
        let cfg = DownloadConfig {
            output_dir: PathBuf::from("out"),
            jobs: 1,
            retries: 0,
            source: EnaFileSource::FastqFtp,
            preference: EnaSourcePreference::Ftp,
            dry_run: true,
        };
        let tasks = build_download_tasks(&[record], &cfg);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].accession, "ERR1");
        assert!(tasks[0]
            .output
            .to_string_lossy()
            .contains("out/ERR1/ERR1_1.fastq.gz"));
        let _ = EnaResultKind::ReadRun;
    }
}
