//! Admission of explicit runner allocations; these declarations are not measured workload peaks.

use std::collections::BTreeSet;

use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};

use super::{SourceIdentity, REQUIRED_LANES};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ResourceAllocation {
    pub memory_bytes: u64,
    pub scratch_bytes: u64,
    pub io_slots: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LaneResourceAllocation {
    pub lane: String,
    pub allocation: ResourceAllocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProvisionedResourceRequest {
    pub schema_version: String,
    pub source: SourceIdentity,
    pub lane_catalog_sha256: String,
    pub authority: String,
    pub measurement_claim: bool,
    pub cpu_worker_units: usize,
    pub memory_capacity_bytes: u64,
    pub artifact_disk_capacity_bytes: u64,
    pub foreground_memory_reserve_bytes: u64,
    pub foreground_disk_reserve_bytes: u64,
    pub retained_growth_reserve_bytes: u64,
    pub io_slots: usize,
    pub top_level_process_slots: usize,
    pub coordinator: ResourceAllocation,
    pub lanes: Vec<LaneResourceAllocation>,
}

pub(crate) struct ResourceCapacityObservation {
    pub physical_ram_bytes: u64,
    pub kernel_reported_available_memory_bytes: u64,
    pub artifact_volume_free_bytes: u64,
    pub cpu_worker_units: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AdmittedResources {
    pub request: ProvisionedResourceRequest,
    pub worker_memory_bytes: u64,
    pub worker_scratch_bytes: u64,
    pub worker_io_slots: usize,
    pub worker_process_slots: usize,
}

/// The archive producer runs inside its reserved Nextest lane, alongside other admitted lanes.
pub(crate) fn admit_archive_build_jobs(build_jobs: u64, lane_units: usize) -> Result<()> {
    ensure!(
        build_jobs > 0 && build_jobs <= u64::try_from(lane_units)?,
        "archive producer exceeds Nextest lane CPU allocation"
    );
    Ok(())
}

pub(crate) fn admit_resources(
    request: ProvisionedResourceRequest,
    candidate: &SourceIdentity,
    catalog_sha256: &str,
    observed: &ResourceCapacityObservation,
) -> Result<AdmittedResources> {
    ensure!(
        request.schema_version == "bijux.ci.provisioned_resources.v1"
            && request.source == *candidate
            && request.lane_catalog_sha256 == catalog_sha256,
        "resource request has stale or unknown source/config authority"
    );
    ensure!(
        !request.authority.trim().is_empty() && !request.measurement_claim,
        "resource request must name a provisioned allocation without claiming measured peaks"
    );
    ensure!(
        request.cpu_worker_units > 0 && request.cpu_worker_units <= observed.cpu_worker_units,
        "provisioned CPU allocation exceeds foreground-safe capacity"
    );
    ensure!(
        [
            (candidate.sha.as_str(), 40),
            (candidate.tree.as_str(), 40),
            (candidate.patch_sha256.as_str(), 64),
            (catalog_sha256, 64)
        ]
        .into_iter()
        .all(|(value, length)| value.len() == length
            && value.bytes().all(|byte| byte.is_ascii_hexdigit())),
        "resource request source/config identity is malformed"
    );
    ensure!(
        request.foreground_memory_reserve_bytes > 0 && request.foreground_disk_reserve_bytes > 0,
        "resource request must preserve foreground RAM and disk capacity"
    );
    let memory_total = request
        .memory_capacity_bytes
        .checked_add(request.foreground_memory_reserve_bytes)
        .context("RAM capacity/reserve overflow")?;
    let disk_total = request
        .artifact_disk_capacity_bytes
        .checked_add(request.foreground_disk_reserve_bytes)
        .context("disk capacity/reserve overflow")?;
    ensure!(request.memory_capacity_bytes>0&&memory_total<=observed.physical_ram_bytes&&memory_total<=observed.kernel_reported_available_memory_bytes,"provisioned RAM allocation does not fit reported available capacity and foreground reserve");
    ensure!(
        request.artifact_disk_capacity_bytes > 0
            && disk_total <= observed.artifact_volume_free_bytes,
        "provisioned disk allocation does not fit free artifact volume and foreground reserve"
    );
    ensure!(
        (2..=6).contains(&request.top_level_process_slots),
        "top-level process allocation must cover coordinator and one to five workers"
    );
    let identities = request.lanes.iter().map(|lane| lane.lane.as_str()).collect::<BTreeSet<_>>();
    ensure!(
        request.lanes.len() == REQUIRED_LANES.len()
            && identities == REQUIRED_LANES.into_iter().collect(),
        "resource request must declare each required lane exactly once"
    );
    ensure!(
        request.coordinator.memory_bytes > 0 && request.coordinator.io_slots > 0,
        "coordinator RAM/IO allocation is missing"
    );
    let retained_and_setup = request
        .retained_growth_reserve_bytes
        .checked_add(request.coordinator.scratch_bytes)
        .context("retained/setup disk allocation overflow")?;
    let worker_memory_bytes = request
        .memory_capacity_bytes
        .checked_sub(request.coordinator.memory_bytes)
        .context("coordinator exceeds RAM allocation")?;
    let worker_scratch_bytes = request
        .artifact_disk_capacity_bytes
        .checked_sub(retained_and_setup)
        .context("retained/setup growth exceeds disk allocation")?;
    let worker_io_slots = request
        .io_slots
        .checked_sub(request.coordinator.io_slots)
        .context("coordinator exceeds IO allocation")?;
    // Lane receipts and streams persist after completion. Reserve their declared disk growth
    // for the whole run rather than assuming completion releases physical disk consumption.
    let persistent_lane_growth = request.lanes.iter().try_fold(0u64, |total, lane| {
        total
            .checked_add(lane.allocation.scratch_bytes)
            .context("persistent lane disk allocation overflow")
    })?;
    ensure!(
        persistent_lane_growth <= worker_scratch_bytes,
        "persistent lane outputs exceed provisioned disk growth allocation"
    );
    for lane in &request.lanes {
        ensure!(
            lane.allocation.memory_bytes > 0
                && lane.allocation.memory_bytes <= worker_memory_bytes
                && lane.allocation.scratch_bytes <= worker_scratch_bytes
                && lane.allocation.io_slots > 0
                && lane.allocation.io_slots <= worker_io_slots,
            "lane {} does not fit provisioned RAM/disk/IO allocation",
            lane.lane
        );
    }
    Ok(AdmittedResources {
        worker_memory_bytes,
        worker_scratch_bytes,
        worker_io_slots,
        worker_process_slots: request.top_level_process_slots - 1,
        request,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn archive_jobs_cannot_borrow_capacity_reserved_by_another_lane() -> Result<()> {
        // A nine-unit coordinator can admit an eight-unit Nextest lane plus Python.
        // Its archive must remain within eight units even though nine exist overall.
        admit_archive_build_jobs(8, 8)?;
        admit_archive_build_jobs(2, 8)?;
        assert!(admit_archive_build_jobs(9, 8).is_err());
        assert!(admit_archive_build_jobs(0, 8).is_err());
        assert!(admit_archive_build_jobs(1, 0).is_err());
        assert!(admit_archive_build_jobs(u64::MAX, 8).is_err());
        Ok(())
    }

    fn fixture() -> (SourceIdentity, ProvisionedResourceRequest, ResourceCapacityObservation) {
        let source = SourceIdentity {
            sha: "a".repeat(40),
            tree: "b".repeat(40),
            patch_sha256: "c".repeat(64),
        };
        let request = ProvisionedResourceRequest {
            schema_version: "bijux.ci.provisioned_resources.v1".into(),
            source: source.clone(),
            lane_catalog_sha256: "d".repeat(64),
            authority: "explicit fixture runner allocation".into(),
            measurement_claim: false,
            cpu_worker_units: 8,
            memory_capacity_bytes: 1200,
            artifact_disk_capacity_bytes: 6000,
            foreground_memory_reserve_bytes: 200,
            foreground_disk_reserve_bytes: 500,
            retained_growth_reserve_bytes: 2000,
            io_slots: 5,
            top_level_process_slots: 6,
            coordinator: ResourceAllocation { memory_bytes: 100, scratch_bytes: 250, io_slots: 1 },
            lanes: REQUIRED_LANES
                .into_iter()
                .map(|lane| LaneResourceAllocation {
                    lane: lane.into(),
                    allocation: ResourceAllocation {
                        memory_bytes: 800,
                        scratch_bytes: 500,
                        io_slots: 1,
                    },
                })
                .collect(),
        };
        let observed = ResourceCapacityObservation {
            physical_ram_bytes: 3200,
            kernel_reported_available_memory_bytes: 1800,
            artifact_volume_free_bytes: 9000,
            cpu_worker_units: 11,
        };
        (source, request, observed)
    }

    #[test]
    fn provisioned_envelopes_reserve_coordinator_and_retained_growth_once() -> Result<()> {
        let (source, request, observed) = fixture();
        let admitted = admit_resources(request, &source, &"d".repeat(64), &observed)?;
        assert_eq!(admitted.worker_memory_bytes, 1100);
        assert_eq!(admitted.worker_scratch_bytes, 3750);
        assert_eq!(admitted.worker_io_slots, 4);
        assert_eq!(admitted.worker_process_slots, 5);
        Ok(())
    }

    #[test]
    fn completed_lane_outputs_remain_reserved_in_disk_admission() {
        let (source, mut request, observed) = fixture();
        for lane in &mut request.lanes {
            lane.allocation.scratch_bytes = 800;
        }
        assert!(admit_resources(request, &source, &"d".repeat(64), &observed).is_err());
    }

    #[test]
    fn stale_unknown_or_unavailable_declared_capacity_cannot_admit() -> Result<()> {
        let (source, request, observed) = fixture();
        for bad in [
            {
                let mut bad = request.clone();
                bad.source.sha = "e".repeat(40);
                bad
            },
            {
                let mut bad = request.clone();
                bad.lane_catalog_sha256 = "e".repeat(64);
                bad
            },
            {
                let mut bad = request.clone();
                bad.measurement_claim = true;
                bad
            },
            {
                let mut bad = request.clone();
                bad.memory_capacity_bytes = 1700;
                bad
            },
            {
                let mut bad = request.clone();
                bad.artifact_disk_capacity_bytes = 9000;
                bad
            },
            {
                let mut bad = request.clone();
                bad.io_slots = 1;
                bad
            },
            {
                let mut bad = request.clone();
                bad.top_level_process_slots = 1;
                bad
            },
            {
                let mut bad = request.clone();
                bad.lanes[4] = bad.lanes[0].clone();
                bad
            },
            {
                let mut bad = request.clone();
                bad.lanes[0].allocation.memory_bytes = 1101;
                bad
            },
            {
                let mut bad = request.clone();
                bad.lanes[0].allocation.scratch_bytes = 3751;
                bad
            },
            {
                let mut bad = request.clone();
                bad.foreground_memory_reserve_bytes = u64::MAX;
                bad
            },
            {
                let mut bad = request.clone();
                bad.foreground_disk_reserve_bytes = u64::MAX;
                bad
            },
            {
                let mut bad = request.clone();
                bad.retained_growth_reserve_bytes = u64::MAX;
                bad
            },
        ] {
            assert!(admit_resources(bad, &source, &"d".repeat(64), &observed).is_err());
        }
        let mut unavailable = observed;
        unavailable.kernel_reported_available_memory_bytes = 1399;
        assert!(admit_resources(request.clone(), &source, &"d".repeat(64), &unavailable).is_err());
        unavailable.kernel_reported_available_memory_bytes = 1800;
        unavailable.physical_ram_bytes = 1399;
        assert!(admit_resources(request.clone(), &source, &"d".repeat(64), &unavailable).is_err());
        let unknown = SourceIdentity {
            sha: String::new(),
            tree: source.tree.clone(),
            patch_sha256: source.patch_sha256.clone(),
        };
        let mut bad = request.clone();
        bad.source = unknown.clone();
        assert!(admit_resources(bad, &unknown, &"d".repeat(64), &unavailable).is_err());
        Ok(())
    }
}
