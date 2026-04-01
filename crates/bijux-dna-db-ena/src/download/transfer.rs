use anyhow::{Context, Result};
use reqwest::blocking::Client;
use std::fs;

use super::DownloadTask;

pub(super) fn download_one(task: &DownloadTask, retries: usize, http: &Client) -> Result<()> {
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
