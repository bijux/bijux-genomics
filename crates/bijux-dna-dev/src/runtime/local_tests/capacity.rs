//! Record kernel capacity observations separately from logical workload allocation.

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
    let (available_memory, availability_kind) = if cfg!(target_os = "macos") {
        let output = ProcessRunner::new(workspace).run(&["/usr/bin/memory_pressure", "-Q"])?;
        ensure!(output.status.success(), "kernel memory availability query failed");
        let facts = std::str::from_utf8(&output.stdout)?;
        let percent = facts
            .lines()
            .find_map(|line| line.strip_prefix("System-wide memory free percentage: "))
            .context("kernel memory availability absent")?
            .trim()
            .strip_suffix('%')
            .context("kernel memory availability unit absent")?
            .parse::<u64>()?;
        ensure!(percent <= 100, "kernel memory availability malformed");
        (
            physical_ram.checked_mul(percent).context("kernel memory availability overflow")? / 100,
            "kernel_memory_pressure_estimate",
        )
    } else {
        let facts = std::fs::read_to_string("/proc/meminfo")?;
        let value = facts
            .lines()
            .find_map(|line| line.strip_prefix("MemAvailable:"))
            .context("kernel MemAvailable observation absent")?;
        (
            value
                .split_whitespace()
                .next()
                .context("kernel MemAvailable value absent")?
                .parse::<u64>()?
                .checked_mul(1024)
                .context("kernel MemAvailable overflow")?,
            "kernel_mem_available",
        )
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
        json!({"schema_version":"bijux.ci.local_test_capacity.v1","physical_ram_bytes":physical_ram,"kernel_reported_available_memory_bytes":available_memory,"memory_availability_kind":availability_kind,"artifact_volume":fields[0],"artifact_volume_free_bytes":free_disk,
        "memory_demand_admission":"see provisioned_resource_admission; observation alone does not admit workload peaks or enforce actual RSS","disk_demand_admission":"see provisioned_resource_admission; observation alone does not admit persistent growth or enforce actual bytes",
        "io_assumptions":{"shared_cargo_target_writers":1,"independent_python_fixture_compile_jobs":2,"enforced_io_rate":false},
        "process_assumptions":{"top_level_lane_process_limit":5,"nested_process_limits":"Cargo jobs and full-gate resource receipt; lane limit is not an OS process cap"}}),
    )
}
