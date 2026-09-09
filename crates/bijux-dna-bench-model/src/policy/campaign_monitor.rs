//! Pure campaign monitoring aggregation over explicit filesystem observations.
use crate::{
    SlurmMonitorEntry, SlurmMonitorReport, SlurmMonitorSnapshot, CAMPAIGN_MONITOR_SCHEMA_VERSION,
};
#[must_use]
pub fn summarize_campaign_monitor(
    campaign_id: String,
    domain: String,
    entries: Vec<SlurmMonitorEntry>,
) -> SlurmMonitorReport {
    let snapshot = SlurmMonitorSnapshot {
        total_jobs: entries.len(),
        jobs_with_log: entries.iter().filter(|row| row.log_exists).count(),
        jobs_with_out: entries.iter().filter(|row| row.out_exists).count(),
        jobs_with_err: entries.iter().filter(|row| row.err_exists).count(),
        jobs_with_results_bundle: entries
            .iter()
            .filter(|row| {
                row.results_exists && row.results_sidecar_exists && row.results_bundle_encrypted
            })
            .count(),
        jobs_with_code_bundle: entries
            .iter()
            .filter(|row| row.code_exists && row.code_sidecar_exists && row.code_bundle_encrypted)
            .count(),
        jobs_with_appraiser_done: entries.iter().filter(|row| row.appraiser_done).count(),
    };
    SlurmMonitorReport {
        schema_version: CAMPAIGN_MONITOR_SCHEMA_VERSION,
        campaign_id,
        domain,
        snapshot,
        jobs: entries,
    }
}
