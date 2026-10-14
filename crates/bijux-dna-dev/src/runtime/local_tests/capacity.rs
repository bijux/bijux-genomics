//! Record observed machine capacity separately from enforced local CPU admission.

use std::path::Path;

use anyhow::{ensure, Context, Result};
use serde_json::{json, Value};

use crate::runtime::{process::ProcessRunner, workspace::Workspace};

pub(crate) fn observe_capacity(workspace: &Workspace, owner: &Path) -> Result<Value> {
    let physical_ram = if cfg!(target_os = "macos") {
        let output =
            ProcessRunner::new(workspace).run(&["/usr/sbin/sysctl", "-n", "hw.memsize"])?;
        ensure!(output.status.success(), "physical RAM observation failed");
        std::str::from_utf8(&output.stdout)?.trim().parse::<u64>()?
    } else {
        let facts = std::fs::read_to_string("/proc/meminfo")?;
        let value = facts
            .lines()
            .find_map(|line| line.strip_prefix("MemTotal:"))
            .context("physical RAM observation absent")?;
        value
            .split_whitespace()
            .next()
            .context("physical RAM value absent")?
            .parse::<u64>()?
            .checked_mul(1024)
            .context("physical RAM size overflow")?
    };
    let output = ProcessRunner::new(workspace)
        .run_owned("/bin/df", &["-Pk".into(), owner.display().to_string()])?;
    ensure!(output.status.success(), "artifact volume free-space observation failed");
    let facts = std::str::from_utf8(&output.stdout)?;
    let fields = facts
        .lines()
        .last()
        .context("artifact volume observation absent")?
        .split_whitespace()
        .collect::<Vec<_>>();
    ensure!(fields.len() >= 6, "artifact volume observation malformed");
    let free_disk =
        fields[3].parse::<u64>()?.checked_mul(1024).context("free disk observation overflow")?;
    Ok(
        json!({"schema_version":"bijux.ci.local_test_capacity.v1","physical_ram_bytes":physical_ram,"artifact_volume":fields[0],"artifact_volume_free_bytes":free_disk,
        "memory_demand_admission":"not implemented; runner must provision compilation and test peaks","disk_demand_admission":"not implemented; runner must provision retained products and scratch peaks",
        "io_assumptions":{"shared_cargo_target_writers":1,"independent_python_fixture_compile_jobs":2,"enforced_io_rate":false},
        "process_assumptions":{"top_level_lane_process_limit":5,"nested_process_limits":"Cargo jobs and full-gate resource receipt; lane limit is not an OS process cap"}}),
    )
}
