use std::path::Path;

use super::{DownloadConfig, DownloadTask};
use crate::model::EnaRecord;

#[must_use]
pub(super) fn build_download_tasks(
    records: &[EnaRecord],
    config: &DownloadConfig,
) -> Vec<DownloadTask> {
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

fn file_name_from_url(url: &str) -> String {
    Path::new(url)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown_file".to_string())
}
