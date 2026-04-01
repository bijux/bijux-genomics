use super::{output_layout, DownloadConfig, DownloadTask};
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
            let file = output_layout::file_name_from_url(&url);
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
