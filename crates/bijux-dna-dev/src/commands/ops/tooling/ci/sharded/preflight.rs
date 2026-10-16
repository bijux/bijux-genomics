//! Governed executor and runtime dependency admission.

use crate::commands::ops::tooling::ci::sharded::contract::FullShardRuntimeDependency;
use crate::commands::ops::tooling::write_json_pretty;
use crate::runtime::process::ProcessRunner;
use crate::runtime::workspace::Workspace;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_policies::CiBuildAuthority;
use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

pub(in super::super) fn require_ci_test_executor(
    workspace: &Workspace,
    authority: &CiBuildAuthority,
) -> Result<()> {
    let output = ProcessRunner::new(workspace).run_owned(
        "cargo",
        &[authority.test_executor.subcommand.clone(), "--version".to_string()],
    )?;
    let observed = if output.status.success() {
        String::from_utf8(output.stdout)?.trim().to_string()
    } else {
        String::from_utf8_lossy(&output.stderr).trim().to_string()
    };
    validate_ci_test_executor_observation(authority, output.status.success(), &observed)
}

pub(in super::super) fn observe_full_shard_runtime_dependencies(
    workspace: &Workspace,
    dependencies: &[FullShardRuntimeDependency],
    report_path: &Path,
) -> Result<Vec<serde_json::Value>> {
    observe_full_shard_runtime_dependencies_with_env(workspace, dependencies, report_path, None)
}

pub(in super::super) fn observe_full_shard_runtime_dependencies_with_env(
    workspace: &Workspace,
    dependencies: &[FullShardRuntimeDependency],
    report_path: &Path,
    execution_env: Option<&BTreeMap<String, String>>,
) -> Result<Vec<serde_json::Value>> {
    let mut observations = Vec::with_capacity(dependencies.len());
    let mut all_available = true;
    for dependency in dependencies {
        let (available, observed, error) =
            match bijux_dna_runner::api::run_command_with_context_and_timeout(
                &dependency.executable,
                &dependency.version_args,
                Some(&workspace.root),
                execution_env,
                Duration::from_secs(30),
            ) {
                Ok(timed) => {
                    let output = timed.output;
                    let rendered = if output.stdout.is_empty() {
                        output.stderr.trim().to_string()
                    } else {
                        output.stdout.trim().to_string()
                    };
                    let available = !timed.timed_out
                        && output.exit_code == 0
                        && rendered.lines().next()
                            == Some(dependency.expected_version_line.as_str());
                    (available, rendered, None)
                }
                Err(error) => (false, String::new(), Some(format!("{error:#}"))),
            };
        all_available &= available;
        observations.push(serde_json::json!({
            "id": dependency.id,
            "executable": dependency.executable,
            "version_args": dependency.version_args,
            "expected_version_line": dependency.expected_version_line,
            "provisioning": dependency.provisioning,
            "available": available,
            "observed": observed,
            "error": error,
        }));
    }
    write_json_pretty(
        report_path,
        &serde_json::json!({
            "schema_version": "bijux.ci.full_shard_runtime_dependencies.v1",
            "all_available": all_available,
            "dependencies": &observations,
        }),
    )?;
    if !all_available {
        let unavailable = observations
            .iter()
            .filter(|observation| observation["available"] == false)
            .map(|observation| {
                format!(
                    "{}: expected {:?}, observed {:?}",
                    observation["id"].as_str().unwrap_or("unknown"),
                    observation["expected_version_line"].as_str().unwrap_or("unknown"),
                    observation["observed"]
                        .as_str()
                        .unwrap_or("")
                        .lines()
                        .next()
                        .unwrap_or("unavailable")
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
        anyhow::bail!(
            "full workspace shard runtime dependencies are unavailable ({unavailable}); select the pinned tools with BIJUX_TEST_RUNTIME_ROOT or PATH, or set BIJUX_DNA_RUNNER_BINDINGS to an absolute checksum-bound manifest; run `make test-preflight` before the full suite; see {}",
            report_path.display()
        );
    }
    Ok(observations)
}

pub(in super::super) fn validate_ci_test_executor_observation(
    authority: &CiBuildAuthority,
    available: bool,
    observed: &str,
) -> Result<()> {
    if !available {
        anyhow::bail!(
            "required CI test executor {} {} is unavailable ({observed}); run `cargo run -p bijux-dna-dev -- tooling run ci-install-tools` once before launching shards",
            authority.test_executor.package,
            authority.test_executor.version
        );
    }
    let expected =
        format!("{} {}", authority.test_executor.package, authority.test_executor.version);
    if !observed.lines().next().is_some_and(|line| line.starts_with(&expected)) {
        anyhow::bail!(
            "CI test executor version mismatch: expected {expected}, observed {observed}"
        );
    }
    Ok(())
}

pub(in super::super) fn validate_full_shard_runtime_dependency_catalog(
    workspace: &Workspace,
    dependencies: &[FullShardRuntimeDependency],
) -> Result<()> {
    let catalog = bijux_dna_containers::api::load_container_catalog(&workspace.root)
        .context("load container catalog for full-shard runtime dependencies")?;
    for dependency in dependencies {
        let tool =
            catalog.tools.iter().find(|tool| tool.id == dependency.id).with_context(|| {
                format!(
                    "full-shard runtime dependency {} is absent from the container catalog",
                    dependency.id
                )
            })?;
        let expected_version_line = format!("{} {}", tool.expected_bin, tool.version);
        if dependency.executable != tool.expected_bin
            || dependency.expected_version_line != expected_version_line
        {
            anyhow::bail!(
                "full-shard runtime dependency {} drifted from container catalog executable/version identity",
                dependency.id
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::observe_full_shard_runtime_dependencies;
    use super::observe_full_shard_runtime_dependencies_with_env;
    use super::validate_ci_test_executor_observation;
    use crate::commands::ops::tooling::ci::read_ci_build_authority;
    use crate::commands::ops::tooling::ci::sharded::contract::FullShardRuntimeDependency;
    use crate::runtime::workspace::Workspace;
    use anyhow::Context;
    use std::collections::BTreeMap;

    #[test]
    fn full_shards_publish_runtime_dependency_preflight_receipts() -> anyhow::Result<()> {
        let root = tempfile::tempdir()?;
        let workspace = Workspace { root: root.path().to_path_buf() };
        let report_path = workspace.root.join("artifacts/runtime-dependencies.json");
        let dependencies = vec![FullShardRuntimeDependency {
            id: "bcftools".to_string(),
            executable: "sh".to_string(),
            version_args: vec!["-c".to_string(), "printf 'bcftools 1.21\\n'".to_string()],
            expected_version_line: "bcftools 1.21".to_string(),
            provisioning: "test PATH".to_string(),
        }];

        let observations =
            observe_full_shard_runtime_dependencies(&workspace, &dependencies, &report_path)?;
        assert_eq!(observations.len(), 1);
        let receipt: serde_json::Value = serde_json::from_slice(&std::fs::read(report_path)?)?;
        assert_eq!(receipt.get("all_available").and_then(serde_json::Value::as_bool), Some(true));
        assert_eq!(
            receipt.pointer("/dependencies/0/observed").and_then(serde_json::Value::as_str),
            Some("bcftools 1.21")
        );
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn full_shard_preflight_honors_checksum_bound_runtime_dependencies() -> anyhow::Result<()> {
        use std::os::unix::fs::PermissionsExt as _;

        let root = tempfile::tempdir()?;
        let workspace = Workspace { root: root.path().to_path_buf() };
        let executable = root.path().join("bound-bcftools");
        std::fs::write(&executable, b"#!/bin/sh\nprintf 'bcftools 1.21\\n'\n")?;
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700))?;
        let manifest_path = root.path().join("execution-bindings.json");
        let manifest = bijux_dna_runner::api::ExecutionBindingManifestV1 {
            schema_version: bijux_dna_runner::api::RUNNER_EXECUTION_BINDINGS_SCHEMA_V1.to_string(),
            bindings: vec![bijux_dna_runner::api::CommandExecutionBindingV1 {
                command: "bijux-unavailable-bcftools".to_string(),
                program: executable.display().to_string(),
                prefix_args: Vec::new(),
                required_files: vec![bijux_dna_runner::api::BoundFileIdentityV1 {
                    path: executable.display().to_string(),
                    sha256: bijux_dna_infra::hash_file_sha256(&executable)?,
                    size_bytes: executable.metadata()?.len(),
                }],
            }],
        };
        std::fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)?;
        let execution_env = BTreeMap::from([(
            bijux_dna_runner::api::RUNNER_EXECUTION_BINDINGS_ENV.to_string(),
            manifest_path.display().to_string(),
        )]);
        let report_path = workspace.root.join("artifacts/runtime-dependencies.json");
        let dependencies = vec![FullShardRuntimeDependency {
            id: "bcftools".to_string(),
            executable: "bijux-unavailable-bcftools".to_string(),
            version_args: vec!["--version".to_string()],
            expected_version_line: "bcftools 1.21".to_string(),
            provisioning: "checksum-bound runner execution binding".to_string(),
        }];

        let observations = observe_full_shard_runtime_dependencies_with_env(
            &workspace,
            &dependencies,
            &report_path,
            Some(&execution_env),
        )?;

        assert_eq!(observations.len(), 1);
        assert_eq!(
            observations[0].get("available").and_then(serde_json::Value::as_bool),
            Some(true)
        );
        assert_eq!(
            observations[0].get("observed").and_then(serde_json::Value::as_str),
            Some("bcftools 1.21")
        );
        Ok(())
    }

    #[test]
    fn full_shards_stop_before_launch_when_runtime_dependencies_are_missing() -> anyhow::Result<()>
    {
        let root = tempfile::tempdir()?;
        let workspace = Workspace { root: root.path().to_path_buf() };
        let report_path = workspace.root.join("artifacts/runtime-dependencies.json");
        let dependencies = vec![FullShardRuntimeDependency {
            id: "missing-tool".to_string(),
            executable: "bijux-missing-full-shard-runtime-tool".to_string(),
            version_args: vec!["--version".to_string()],
            expected_version_line: "missing-tool 1.0".to_string(),
            provisioning: "test PATH".to_string(),
        }];

        let error =
            observe_full_shard_runtime_dependencies(&workspace, &dependencies, &report_path)
                .err()
                .context("a missing runtime dependency must stop full-shard launch")?;
        assert!(error.to_string().contains("runtime dependencies are unavailable"));
        let receipt: serde_json::Value = serde_json::from_slice(&std::fs::read(report_path)?)?;
        assert_eq!(receipt.get("all_available").and_then(serde_json::Value::as_bool), Some(false));
        assert_eq!(
            receipt.pointer("/dependencies/0/available").and_then(serde_json::Value::as_bool),
            Some(false)
        );
        Ok(())
    }

    #[test]
    fn full_shards_report_exact_version_mismatch_and_runtime_recovery() -> anyhow::Result<()> {
        let root = tempfile::tempdir()?;
        let workspace = Workspace { root: root.path().to_path_buf() };
        let report_path = workspace.root.join("artifacts/runtime-dependencies.json");
        let dependencies = vec![FullShardRuntimeDependency {
            id: "bcftools".to_string(),
            executable: "sh".to_string(),
            version_args: vec!["-c".to_string(), "printf 'bcftools 1.21\\n'".to_string()],
            expected_version_line: "bcftools 1.20".to_string(),
            provisioning: "pinned test tool".to_string(),
        }];
        let error =
            observe_full_shard_runtime_dependencies(&workspace, &dependencies, &report_path)
                .expect_err("another installed version must not satisfy the pinned tool");
        let message = error.to_string();
        assert!(message.contains("expected \"bcftools 1.20\", observed \"bcftools 1.21\""));
        assert!(message.contains("make test-preflight"));
        assert!(message.contains("BIJUX_DNA_RUNNER_BINDINGS"));
        let receipt: serde_json::Value = serde_json::from_slice(&std::fs::read(report_path)?)?;
        assert_eq!(receipt["all_available"], false);
        assert_eq!(receipt["dependencies"][0]["available"], false);
        Ok(())
    }

    #[test]
    fn full_shards_refuse_missing_or_drifted_test_executor_before_launch() -> anyhow::Result<()> {
        let root =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize()?;
        let workspace = Workspace { root };
        let authority =
            read_ci_build_authority(&workspace.path("configs/ci/build-authority.json"))?;

        let missing = validate_ci_test_executor_observation(&authority, false, "no such command")
            .err()
            .ok_or_else(|| anyhow::anyhow!("missing nextest did not fail closed"))?;
        assert!(missing.to_string().contains("before launching shards"));

        let drifted =
            validate_ci_test_executor_observation(&authority, true, "cargo-nextest 0.9.99")
                .err()
                .ok_or_else(|| anyhow::anyhow!("drifted nextest did not fail closed"))?;
        assert!(drifted.to_string().contains("version mismatch"));

        validate_ci_test_executor_observation(&authority, true, "cargo-nextest 0.9.100 (build)")
    }
}
