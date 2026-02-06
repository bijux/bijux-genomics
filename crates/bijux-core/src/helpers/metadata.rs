use crate::metadata::RunMetadataV1;

fn resolve_hostname() -> String {
    #[cfg(feature = "sysinfo")]
    {
        return sysinfo::System::host_name().unwrap_or_else(|| "unknown-host".to_string());
    }
    #[cfg(not(feature = "sysinfo"))]
    {
        "unknown-host".to_string()
    }
}

#[must_use]
pub fn build_run_metadata_v1(
    run_id: &str,
    pipeline_id: &str,
    planner_version: &str,
    platform: &str,
    runner: &str,
    started_at: &str,
    finished_at: &str,
) -> RunMetadataV1 {
    let hostname = resolve_hostname();
    RunMetadataV1 {
        schema_version: "bijux.run_metadata.v1".to_string(),
        run_id: run_id.to_string(),
        pipeline_id: pipeline_id.to_string(),
        planner_version: planner_version.to_string(),
        platform: platform.to_string(),
        runner: runner.to_string(),
        hostname,
        started_at: started_at.to_string(),
        finished_at: finished_at.to_string(),
    }
}
