//! Declared workflow contract and exact selection.

use crate::commands::ops::tooling::ci::validate_artifact_relative_path;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeSet;

pub(super) const FAST_GROUPS_SCHEMA_V3: &str = "bijux.ci.fast_groups.v3";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FastGroupContract {
    pub(super) schema_version: String,
    pub(super) max_parallelism: usize,
    pub(super) warm_cache_target_milliseconds: u64,
    pub(super) minimum_cache_hit_ratio_basis_points: u64,
    pub(super) history_limit: usize,
    pub(super) report_root: String,
    pub(super) timing_report: String,
    pub(super) timing_history: String,
    pub(super) baseline_report: String,
    pub(super) reference_runner: FastReferenceRunner,
    pub(super) excluded_lanes: Vec<FastExcludedLane>,
    pub(super) groups: Vec<FastGroup>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FastExcludedLane {
    pub(super) id: String,
    pub(super) reason: String,
    pub(super) evidence_command: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FastReferenceRunner {
    pub(super) os: String,
    pub(super) architecture: String,
    pub(super) rust_toolchain: String,
    pub(super) compiler_wrapper: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FastGroup {
    pub(super) id: String,
    pub(super) dependencies: Vec<String>,
    pub(super) program: String,
    pub(super) args: Vec<String>,
}

pub(super) fn validate_fast_group_contract(contract: &FastGroupContract) -> Result<()> {
    if contract.schema_version != FAST_GROUPS_SCHEMA_V3
        || !(2..=16).contains(&contract.max_parallelism)
        || !(2..=16).contains(&contract.groups.len())
        || !(5..=100).contains(&contract.history_limit)
        || contract.warm_cache_target_milliseconds == 0
        || !(1..=10_000).contains(&contract.minimum_cache_hit_ratio_basis_points)
        || contract.reference_runner.os.is_empty()
        || contract.reference_runner.architecture.is_empty()
        || contract.reference_runner.rust_toolchain.is_empty()
        || contract.reference_runner.compiler_wrapper != "sccache"
    {
        anyhow::bail!("ci-fast group contract is incomplete");
    }
    validate_artifact_relative_path(&contract.report_root)?;
    validate_artifact_relative_path(&contract.timing_report)?;
    validate_artifact_relative_path(&contract.timing_history)?;
    validate_artifact_relative_path(&contract.baseline_report)?;
    if BTreeSet::from([
        contract.timing_report.as_str(),
        contract.timing_history.as_str(),
        contract.baseline_report.as_str(),
    ])
    .len()
        != 3
    {
        anyhow::bail!("ci-fast report paths must be distinct");
    }

    let excluded_lane_ids =
        contract.excluded_lanes.iter().map(|lane| lane.id.as_str()).collect::<BTreeSet<_>>();
    if excluded_lane_ids
        != BTreeSet::from(["full-workspace", "lunarc-integration", "nightly-release"])
        || excluded_lane_ids.len() != contract.excluded_lanes.len()
        || contract.excluded_lanes.iter().any(|lane| {
            lane.reason.trim().is_empty()
                || lane.evidence_command.trim().is_empty()
                || lane.evidence_command.contains(['\n', '\0'])
        })
    {
        anyhow::bail!("ci-fast excluded lanes are incomplete");
    }

    let mut ids = BTreeSet::new();
    for group in &contract.groups {
        if group.id.is_empty()
            || !group.id.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            || !ids.insert(group.id.as_str())
            || group.program != "make"
            || group.args.first().map(String::as_str) != Some("--no-print-directory")
            || group.args.iter().any(|arg| arg.is_empty() || arg.contains(['\n', '\0']))
        {
            anyhow::bail!("ci-fast group identity or command is invalid");
        }
    }
    for group in &contract.groups {
        let mut dependencies = BTreeSet::new();
        for dependency in &group.dependencies {
            if dependency == &group.id
                || !ids.contains(dependency.as_str())
                || !dependencies.insert(dependency.as_str())
            {
                anyhow::bail!("ci-fast group dependency is invalid");
            }
        }
    }
    let mut resolved = BTreeSet::new();
    while resolved.len() < contract.groups.len() {
        let ready = contract
            .groups
            .iter()
            .filter(|group| {
                !resolved.contains(group.id.as_str())
                    && group
                        .dependencies
                        .iter()
                        .all(|dependency| resolved.contains(dependency.as_str()))
            })
            .map(|group| group.id.as_str())
            .collect::<Vec<_>>();
        if ready.is_empty() {
            anyhow::bail!("ci-fast group dependencies contain a cycle");
        }
        resolved.extend(ready);
    }
    Ok(())
}

#[cfg(test)]
pub(super) fn fast_contract() -> FastGroupContract {
    FastGroupContract {
        schema_version: "bijux.ci.fast_groups.v3".to_string(),
        max_parallelism: 2,
        warm_cache_target_milliseconds: 300_000,
        minimum_cache_hit_ratio_basis_points: 8_000,
        history_limit: 40,
        report_root: "artifacts/ci/ci-fast-groups".to_string(),
        timing_report: "artifacts/ci/ci-fast-timing.json".to_string(),
        timing_history: "artifacts/ci/ci-fast-timing-history.json".to_string(),
        baseline_report: "artifacts/ci/baseline-timing.json".to_string(),
        reference_runner: FastReferenceRunner {
            os: "linux".to_string(),
            architecture: "x86_64".to_string(),
            rust_toolchain: "1.95.0".to_string(),
            compiler_wrapper: "sccache".to_string(),
        },
        excluded_lanes: [
            ("full-workspace", "full", "make _ci-slow"),
            ("lunarc-integration", "heavy", "bijux-dna benchmark cycle advance"),
            ("nightly-release", "release", "make _ci-slow"),
        ]
        .into_iter()
        .map(|(id, reason, evidence_command)| FastExcludedLane {
            id: id.to_string(),
            reason: reason.to_string(),
            evidence_command: evidence_command.to_string(),
        })
        .collect(),
        groups: ["metadata", "unit"]
            .into_iter()
            .map(|id| FastGroup {
                id: id.to_string(),
                dependencies: Vec::new(),
                program: "make".to_string(),
                args: vec!["--no-print-directory".to_string(), format!("_{id}")],
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::fast_contract;
    use super::validate_fast_group_contract;

    #[test]
    fn fast_group_contract_is_bounded_and_acyclic() {
        assert!(validate_fast_group_contract(&fast_contract()).is_ok());

        let mut cycle = fast_contract();
        cycle.groups[0].dependencies = vec!["unit".to_string()];
        cycle.groups[1].dependencies = vec!["metadata".to_string()];
        assert!(validate_fast_group_contract(&cycle).is_err());

        let mut escaping = fast_contract();
        escaping.timing_report = "../ci-fast.json".to_string();
        assert!(validate_fast_group_contract(&escaping).is_err());

        let mut omitted_lane = fast_contract();
        omitted_lane.excluded_lanes.pop();
        assert!(validate_fast_group_contract(&omitted_lane).is_err());

        let mut duplicate_lane = fast_contract();
        duplicate_lane.excluded_lanes.push(duplicate_lane.excluded_lanes[0].clone());
        assert!(validate_fast_group_contract(&duplicate_lane).is_err());

        let mut colliding_report = fast_contract();
        colliding_report.baseline_report = colliding_report.timing_report.clone();
        assert!(validate_fast_group_contract(&colliding_report).is_err());
    }
}
