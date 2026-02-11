use crate::model::{EnaFileSource, EnaRecord, EnaSourcePreference};
use anyhow::{Context, Result};
use rayon::prelude::*;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

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
    let mut tasks = Vec::new();

    for record in records {
        let urls = record.preferred_urls(config.source, config.preference);
        for url in urls {
            let file = file_name_from_url(&url);
            let accession = record.accession_label();
            let output = config.output_dir.join(&accession).join(file);
            tasks.push(DownloadTask {
                project: record.study_accession.clone(),
                sample: record.sample_accession.clone(),
                accession,
                source: config.source,
                url,
                output,
            });
        }
    }

    tasks.sort_by(|a, b| a.output.cmp(&b.output));
    tasks
}

pub fn download_tasks(tasks: &[DownloadTask], config: &DownloadConfig) -> Result<DownloadReport> {
    if config.dry_run {
        return Ok(DownloadReport {
            attempted: tasks.len(),
            downloaded: 0,
            failed: 0,
            failed_outputs: Vec::new(),
        });
    }

    if config.jobs == 0 {
        anyhow::bail!("jobs must be greater than zero");
    }

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(config.jobs)
        .build()
        .context("create rayon pool")?;

    let http = Client::builder()
        .user_agent("bijux-dna-db-ena/0.1")
        .build()
        .context("create download http client")?;

    let downloaded = AtomicUsize::new(0);
    let failed_outputs = pool.install(|| {
        tasks
            .par_iter()
            .filter_map(|task| match download_one(task, config.retries, &http) {
                Ok(()) => {
                    downloaded.fetch_add(1, Ordering::Relaxed);
                    None
                }
                Err(_) => Some(task.output.clone()),
            })
            .collect::<Vec<_>>()
    });

    let downloaded = downloaded.load(Ordering::Relaxed);
    Ok(DownloadReport {
        attempted: tasks.len(),
        downloaded,
        failed: tasks.len().saturating_sub(downloaded),
        failed_outputs,
    })
}

fn download_one(task: &DownloadTask, retries: usize, http: &Client) -> Result<()> {
    if task.output.exists() {
        return Ok(());
    }

    if let Some(parent) = task.output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create directory {}", parent.display()))?;
    }

    let mut last_err: Option<anyhow::Error> = None;
    for _attempt in 0..=retries {
        match http.get(&task.url).send() {
            Ok(resp) => match resp.error_for_status() {
                Ok(success) => {
                    let bytes = success
                        .bytes()
                        .with_context(|| format!("read bytes for {}", task.url))?;
                    fs::write(&task.output, &bytes).with_context(|| {
                        format!("write {} from {}", task.output.display(), task.url)
                    })?;
                    return Ok(());
                }
                Err(e) => {
                    last_err = Some(e.into());
                }
            },
            Err(e) => {
                last_err = Some(e.into());
            }
        }
    }

    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("download failed for {}", task.url)))
}

fn file_name_from_url(url: &str) -> String {
    Path::new(url)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown_file".to_string())
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
