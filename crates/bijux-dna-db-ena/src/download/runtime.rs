use anyhow::{Context, Result};
use rayon::prelude::*;
use reqwest::blocking::Client;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::client::CRATE_USER_AGENT;

use super::{transfer, DownloadConfig, DownloadReport, DownloadTask};

/// # Errors
/// Returns an error if configuration is invalid, the thread pool cannot be
/// created, or HTTP client initialization fails.
pub(super) fn download_tasks(
    tasks: &[DownloadTask],
    config: &DownloadConfig,
) -> Result<DownloadReport> {
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
        .user_agent(CRATE_USER_AGENT)
        .build()
        .context("create download http client")?;

    let downloaded = AtomicUsize::new(0);
    let failed_outputs = pool.install(|| {
        tasks
            .par_iter()
            .filter_map(
                |task| match transfer::download_one(task, config.retries, &http) {
                    Ok(()) => {
                        downloaded.fetch_add(1, Ordering::Relaxed);
                        None
                    }
                    Err(_) => Some(task.output.clone()),
                },
            )
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
