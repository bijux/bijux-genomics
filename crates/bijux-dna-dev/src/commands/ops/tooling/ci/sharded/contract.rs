//! Declared workflow contract and exact selection.

use crate::commands::ops::tooling::ci::validate_artifact_relative_path;
use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;

pub(in super::super) const FULL_SHARDS_SCHEMA_V5: &str = "bijux.ci.full_shards.v5";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in super::super) struct FullShardContract {
    pub(in super::super) schema_version: String,
    pub(in super::super) coverage: FullShardCoverage,
    pub(in super::super) partition_strategy: String,
    pub(in super::super) workspace_isolation: String,
    pub(in super::super) build_authority: String,
    pub(in super::super) runtime_dependency_report: String,
    pub(in super::super) runtime_dependencies: Vec<FullShardRuntimeDependency>,
    pub(in super::super) archive_build_jobs: usize,
    pub(in super::super) archive_root: String,
    pub(in super::super) shard_artifact_root: String,
    pub(in super::super) report_root: String,
    pub(in super::super) shards: Vec<FullShard>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in super::super) struct FullShardRuntimeDependency {
    pub(in super::super) id: String,
    pub(in super::super) executable: String,
    pub(in super::super) version_args: Vec<String>,
    pub(in super::super) expected_version_line: String,
    pub(in super::super) provisioning: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in super::super) struct FullShardCoverage {
    pub(in super::super) workspace: bool,
    pub(in super::super) all_targets: bool,
    pub(in super::super) all_features: bool,
    pub(in super::super) run_ignored: String,
    pub(in super::super) overlap_policy: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in super::super) struct FullShard {
    pub(in super::super) id: String,
    pub(in super::super) partition: String,
}

pub(in super::super) fn select_full_shards(
    contract: &FullShardContract,
    args: &[String],
) -> Result<Vec<FullShard>> {
    if args.is_empty() {
        return Ok(contract.shards.clone());
    }
    if !args.len().is_multiple_of(2) {
        anyhow::bail!("ci-test-all-sharded expects repeated --shard <id> pairs");
    }
    let mut selected_ids = BTreeSet::new();
    let mut selected = Vec::with_capacity(args.len() / 2);
    for pair in args.chunks_exact(2) {
        if pair[0] != "--shard" || pair[1].trim().is_empty() {
            anyhow::bail!("ci-test-all-sharded expects repeated --shard <id> pairs");
        }
        if !selected_ids.insert(pair[1].as_str()) {
            anyhow::bail!("full shard selection repeats {}", pair[1]);
        }
        selected.push(
            contract
                .shards
                .iter()
                .find(|shard| shard.id == pair[1])
                .cloned()
                .with_context(|| format!("unknown full shard selection {}", pair[1]))?,
        );
    }
    Ok(selected)
}

pub(in super::super) fn full_shard_execution_scope(
    contract: &FullShardContract,
    selected_shards: &[FullShard],
    observed_complete: bool,
) -> serde_json::Value {
    serde_json::json!({
        "mode": if selected_shards.len() == contract.shards.len() {
            "complete"
        } else {
            "affected_recovery"
        },
        "all_declared_shards_selected": selected_shards.len() == contract.shards.len(),
        "complete_contract_coverage": selected_shards.len() == contract.shards.len() && observed_complete,
        "selected_shard_ids": selected_shards.iter().map(|shard| &shard.id).collect::<Vec<_>>(),
        "selected_partitions": selected_shards
            .iter()
            .map(|shard| &shard.partition)
            .collect::<Vec<_>>(),
        "contract_shard_count": contract.shards.len(),
        "selected_shard_count": selected_shards.len(),
    })
}

pub(in super::super) fn validate_full_shard_contract(contract: &FullShardContract) -> Result<()> {
    if contract.schema_version != FULL_SHARDS_SCHEMA_V5
        || !contract.coverage.workspace
        || !contract.coverage.all_targets
        || !contract.coverage.all_features
        || contract.coverage.run_ignored != "all"
        || contract.coverage.overlap_policy != "exactly_once"
        || contract.partition_strategy != "hash"
        || contract.workspace_isolation != "tracked_source_clones_with_shared_immutable_archive"
        || contract.build_authority != "configs/ci/build-authority.json"
        || contract.runtime_dependencies.is_empty()
        || !(1..=16).contains(&contract.archive_build_jobs)
        || contract.shards.len() < 2
    {
        anyhow::bail!("full shard coverage contract is incomplete");
    }
    validate_artifact_relative_path(&contract.archive_root)?;
    validate_artifact_relative_path(&contract.shard_artifact_root)?;
    validate_artifact_relative_path(&contract.report_root)?;
    validate_artifact_relative_path(&contract.runtime_dependency_report)?;
    let report_root = Path::new(&contract.report_root);
    let archive_root = Path::new(&contract.archive_root);
    let shard_artifact_root = Path::new(&contract.shard_artifact_root);
    let runtime_dependency_report = Path::new(&contract.runtime_dependency_report);
    if archive_root == report_root
        || !archive_root.starts_with(report_root)
        || shard_artifact_root == report_root
        || !shard_artifact_root.starts_with(report_root)
        || archive_root.starts_with(shard_artifact_root)
        || shard_artifact_root.starts_with(archive_root)
        || !runtime_dependency_report.starts_with(report_root)
        || runtime_dependency_report == report_root
        || runtime_dependency_report.starts_with(archive_root)
        || runtime_dependency_report.starts_with(shard_artifact_root)
        || archive_root.starts_with(runtime_dependency_report)
        || shard_artifact_root.starts_with(runtime_dependency_report)
    {
        anyhow::bail!(
            "full shard archive, shard, and runtime dependency paths must be disjoint children of the report root"
        );
    }
    let mut runtime_dependency_ids = BTreeSet::new();
    for dependency in &contract.runtime_dependencies {
        if dependency.id.trim().is_empty()
            || dependency.executable.trim().is_empty()
            || dependency.version_args.is_empty()
            || dependency.version_args.iter().any(|arg| arg.trim().is_empty())
            || dependency.expected_version_line.trim().is_empty()
            || dependency.provisioning.trim().is_empty()
            || !runtime_dependency_ids.insert(dependency.id.as_str())
        {
            anyhow::bail!("full shard runtime dependency contract is incomplete");
        }
    }
    let denominator = contract.shards.len();
    let mut ids = BTreeSet::new();
    let mut indices = BTreeSet::new();
    for shard in &contract.shards {
        if shard.id.is_empty()
            || !shard.id.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            || !ids.insert(shard.id.as_str())
        {
            anyhow::bail!("full shard identity is invalid");
        }
        let value = shard
            .partition
            .strip_prefix("hash:")
            .context("full shard partition must use hash strategy")?;
        let (index, declared_denominator) =
            value.split_once('/').context("full shard partition is malformed")?;
        let index = index.parse::<usize>().context("full shard index is invalid")?;
        let declared_denominator =
            declared_denominator.parse::<usize>().context("full shard denominator is invalid")?;
        if declared_denominator != denominator
            || !(1..=denominator).contains(&index)
            || !indices.insert(index)
        {
            anyhow::bail!("full shard partitions do not cover the hash space exactly once");
        }
    }
    Ok(())
}

#[cfg(test)]
pub(super) fn contract() -> FullShardContract {
    FullShardContract {
        schema_version: "bijux.ci.full_shards.v5".to_string(),
        coverage: FullShardCoverage {
            workspace: true,
            all_targets: true,
            all_features: true,
            run_ignored: "all".to_string(),
            overlap_policy: "exactly_once".to_string(),
        },
        partition_strategy: "hash".to_string(),
        workspace_isolation: "tracked_source_clones_with_shared_immutable_archive".to_string(),
        build_authority: "configs/ci/build-authority.json".to_string(),
        runtime_dependency_report: "artifacts/ci/full-shards/runtime-dependencies.json".to_string(),
        runtime_dependencies: vec![FullShardRuntimeDependency {
            id: "bcftools".to_string(),
            executable: "bcftools".to_string(),
            version_args: vec!["--version".to_string()],
            expected_version_line: "bcftools 1.21".to_string(),
            provisioning: "caller-supplied PATH".to_string(),
        }],
        archive_build_jobs: 4,
        archive_root: "artifacts/ci/full-shards/archive".to_string(),
        shard_artifact_root: "artifacts/ci/full-shards/shards".to_string(),
        report_root: "artifacts/ci/full-shards".to_string(),
        shards: (1..=5)
            .map(|index| FullShard {
                id: format!("hash-{index}-of-5"),
                partition: format!("hash:{index}/5"),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::contract;
    use super::full_shard_execution_scope;
    use super::select_full_shards;
    use super::validate_full_shard_contract;
    use super::FullShard;
    use super::FullShardContract;
    use crate::commands::ops::tooling::ci::sharded::preflight::validate_full_shard_runtime_dependency_catalog;
    use crate::runtime::workspace::Workspace;

    #[test]
    fn full_shard_contract_covers_hash_space_exactly_once() {
        assert!(validate_full_shard_contract(&contract()).is_ok());
    }

    #[test]
    fn full_shard_contract_admits_hundreds_of_complete_hash_partitions() {
        for count in [100, 256] {
            let mut contract = contract();
            contract.shards = (1..=count)
                .map(|index| FullShard {
                    id: format!("hash-{index}-of-{count}"),
                    partition: format!("hash:{index}/{count}"),
                })
                .collect();
            assert!(validate_full_shard_contract(&contract).is_ok());
            contract.shards[0].partition = format!("hash:2/{count}");
            assert!(validate_full_shard_contract(&contract).is_err());
            contract.shards[0].partition = format!("hash:1/{}", count + 1);
            assert!(validate_full_shard_contract(&contract).is_err());
            contract.shards[0].partition = format!("hash:1/{count}");
            contract.archive_build_jobs = 17;
            assert!(validate_full_shard_contract(&contract).is_err());
        }
    }

    #[test]
    fn full_shard_selection_supports_exact_affected_recovery() -> anyhow::Result<()> {
        let contract = contract();
        assert_eq!(select_full_shards(&contract, &[])?.len(), 5);

        let selected = select_full_shards(
            &contract,
            &[
                "--shard".to_string(),
                "hash-4-of-5".to_string(),
                "--shard".to_string(),
                "hash-5-of-5".to_string(),
            ],
        )?;
        assert_eq!(
            selected.iter().map(|shard| shard.id.as_str()).collect::<Vec<_>>(),
            ["hash-4-of-5", "hash-5-of-5"]
        );
        let scope = full_shard_execution_scope(&contract, &selected, false);
        assert_eq!(scope["mode"], "affected_recovery");
        assert_eq!(scope["complete_contract_coverage"], false);
        assert_eq!(scope["selected_shard_count"], 2);
        assert!(select_full_shards(
            &contract,
            &[
                "--shard".to_string(),
                "hash-4-of-5".to_string(),
                "--shard".to_string(),
                "hash-4-of-5".to_string(),
            ]
        )
        .is_err());
        assert!(select_full_shards(&contract, &["--shard".to_string(), "hash-9-of-5".to_string()])
            .is_err());
        assert!(select_full_shards(&contract, &["--shard".to_string()]).is_err());
        Ok(())
    }

    #[test]
    fn checked_in_full_shard_contract_uses_five_complete_partitions() -> anyhow::Result<()> {
        let repository_root =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize()?;
        let path = repository_root.join("configs/ci/full-shards.json");
        let contract: FullShardContract = serde_json::from_slice(&std::fs::read(path)?)?;

        validate_full_shard_contract(&contract)?;
        validate_full_shard_runtime_dependency_catalog(
            &Workspace { root: repository_root },
            &contract.runtime_dependencies,
        )?;
        assert_eq!(contract.shards.len(), 5);
        assert_eq!(
            contract.shards.iter().map(|shard| shard.partition.as_str()).collect::<Vec<_>>(),
            ["hash:1/5", "hash:2/5", "hash:3/5", "hash:4/5", "hash:5/5"]
        );
        assert_eq!(
            contract.runtime_dependencies[0].provisioning,
            "checksum-bound runner execution binding; site modules are not required"
        );
        Ok(())
    }

    #[test]
    fn full_shard_contract_rejects_overlap_and_escaping_outputs() {
        let mut overlap = contract();
        overlap.shards[4].partition = "hash:4/5".to_string();
        assert!(validate_full_shard_contract(&overlap).is_err());

        let mut escaping = contract();
        escaping.report_root = "../reports".to_string();
        assert!(validate_full_shard_contract(&escaping).is_err());

        let mut shared_workspace = contract();
        shared_workspace.workspace_isolation = "shared".to_string();
        assert!(validate_full_shard_contract(&shared_workspace).is_err());

        let mut shared_outputs = contract();
        shared_outputs.shard_artifact_root = shared_outputs.report_root.clone();
        assert!(validate_full_shard_contract(&shared_outputs).is_err());

        let mut shared_archive = contract();
        shared_archive.archive_root = shared_archive.shard_artifact_root.clone();
        assert!(validate_full_shard_contract(&shared_archive).is_err());

        let mut unbounded_archive = contract();
        unbounded_archive.archive_build_jobs = 0;
        assert!(validate_full_shard_contract(&unbounded_archive).is_err());

        let mut unknown_authority = contract();
        unknown_authority.build_authority = "configs/ci/another-authority.json".to_string();
        assert!(validate_full_shard_contract(&unknown_authority).is_err());
    }
}
