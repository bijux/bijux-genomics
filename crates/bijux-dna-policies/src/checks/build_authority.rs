use std::collections::BTreeSet;

use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

pub const CI_BUILD_AUTHORITY_SCHEMA_V3: &str = "bijux.ci.build_authority.v3";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CiBuildAuthority {
    pub schema: String,
    pub toolchain: CiToolchainAuthority,
    pub test_executor: CiTestExecutorAuthority,
    pub lockfile: CiLockfileAuthority,
    pub compilation: CiCompilationAuthority,
    pub cache: CiCacheAuthority,
    pub targets: CiTargetAuthority,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CiTestExecutorAuthority {
    pub package: String,
    pub subcommand: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CiToolchainAuthority {
    pub manifest: String,
    pub channel: String,
    pub required_components: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CiLockfileAuthority {
    pub manifest: String,
    pub metadata_mode: String,
    pub dependency_resolution: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CiCompilationAuthority {
    pub manifest: String,
    pub dev_codegen_units: u16,
    pub test_codegen_units: u16,
    pub incremental: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CiCacheAuthority {
    pub compiler_wrapper: String,
    pub metrics_format: String,
    pub before_filename: String,
    pub after_filename: String,
    pub publish_on_failure: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CiTargetAuthority {
    pub shared_prebuild_root: String,
    pub concurrent_target_pattern: String,
    pub concurrent_policy: String,
    pub reuse_policy: String,
}

/// Validate the single CI toolchain, compilation, lock, cache, and target-isolation authority.
///
/// # Errors
/// Returns an error when the declaration permits toolchain drift, unlocked resolution, missing
/// cache metrics, excessive codegen parallelism, or a shared Cargo target lock across concurrent
/// gates.
pub fn validate_ci_build_authority(authority: &CiBuildAuthority) -> Result<()> {
    ensure!(
        authority.schema == CI_BUILD_AUTHORITY_SCHEMA_V3,
        "unsupported CI build authority schema"
    );
    ensure!(
        authority.toolchain.manifest == "rust-toolchain.toml",
        "CI toolchain authority must use rust-toolchain.toml"
    );
    ensure!(!authority.toolchain.channel.trim().is_empty(), "CI toolchain channel is required");
    let components =
        authority.toolchain.required_components.iter().map(String::as_str).collect::<BTreeSet<_>>();
    ensure!(
        components == BTreeSet::from(["clippy", "rustfmt"]),
        "CI toolchain must require exactly clippy and rustfmt"
    );
    ensure!(
        authority.test_executor.package == "cargo-nextest"
            && authority.test_executor.subcommand == "nextest"
            && authority.test_executor.version == "0.9.100",
        "CI test executor must use the pinned cargo-nextest authority"
    );
    ensure!(authority.lockfile.manifest == "Cargo.lock", "CI lock authority must use Cargo.lock");
    ensure!(
        authority.lockfile.metadata_mode == "locked"
            && authority.lockfile.dependency_resolution == "workspace_lock_only",
        "CI dependency resolution must fail closed to the workspace lock"
    );
    ensure!(
        authority.compilation.manifest == "Cargo.toml",
        "CI compilation authority must use the workspace Cargo.toml"
    );
    ensure!(
        authority.compilation.dev_codegen_units == 16
            && authority.compilation.test_codegen_units == 16,
        "CI development and test profiles must use the verified 16 codegen units"
    );
    ensure!(
        authority.compilation.incremental,
        "CI development and test profiles must retain incremental compilation"
    );
    ensure!(authority.cache.compiler_wrapper == "sccache", "CI compiler cache must use sccache");
    ensure!(authority.cache.metrics_format == "json", "CI cache metrics must be JSON");
    ensure!(
        authority.cache.before_filename == "sccache-before.json"
            && authority.cache.after_filename == "sccache-after.json"
            && authority.cache.publish_on_failure,
        "CI cache metrics must publish before and after receipts, including failures"
    );
    ensure!(
        authority.targets.shared_prebuild_root == "artifacts/target",
        "CI shared prebuild root must use the governed artifact target"
    );
    ensure!(
        authority.targets.concurrent_target_pattern == "artifacts/github-all/targets/{gate}"
            && authority.targets.concurrent_policy == "isolated_per_gate",
        "concurrent CI gates must use isolated target directories"
    );
    ensure!(
        authority.targets.reuse_policy == "sccache_and_shared_dev_binary",
        "CI gates may reuse only sccache and the prebuilt developer binary"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        validate_ci_build_authority, CiBuildAuthority, CiCacheAuthority, CiCompilationAuthority,
        CiLockfileAuthority, CiTargetAuthority, CiTestExecutorAuthority, CiToolchainAuthority,
        CI_BUILD_AUTHORITY_SCHEMA_V3,
    };

    fn authority() -> CiBuildAuthority {
        CiBuildAuthority {
            schema: CI_BUILD_AUTHORITY_SCHEMA_V3.to_string(),
            toolchain: CiToolchainAuthority {
                manifest: "rust-toolchain.toml".to_string(),
                channel: "1.95.0".to_string(),
                required_components: vec!["clippy".to_string(), "rustfmt".to_string()],
            },
            test_executor: CiTestExecutorAuthority {
                package: "cargo-nextest".to_string(),
                subcommand: "nextest".to_string(),
                version: "0.9.100".to_string(),
            },
            lockfile: CiLockfileAuthority {
                manifest: "Cargo.lock".to_string(),
                metadata_mode: "locked".to_string(),
                dependency_resolution: "workspace_lock_only".to_string(),
            },
            compilation: CiCompilationAuthority {
                manifest: "Cargo.toml".to_string(),
                dev_codegen_units: 16,
                test_codegen_units: 16,
                incremental: true,
            },
            cache: CiCacheAuthority {
                compiler_wrapper: "sccache".to_string(),
                metrics_format: "json".to_string(),
                before_filename: "sccache-before.json".to_string(),
                after_filename: "sccache-after.json".to_string(),
                publish_on_failure: true,
            },
            targets: CiTargetAuthority {
                shared_prebuild_root: "artifacts/target".to_string(),
                concurrent_target_pattern: "artifacts/github-all/targets/{gate}".to_string(),
                concurrent_policy: "isolated_per_gate".to_string(),
                reuse_policy: "sccache_and_shared_dev_binary".to_string(),
            },
        }
    }

    #[test]
    fn accepts_one_toolchain_lock_cache_and_isolation_authority() -> anyhow::Result<()> {
        validate_ci_build_authority(&authority())
    }

    #[test]
    fn rejects_shared_target_lock_for_concurrent_gates() {
        let mut authority = authority();
        authority.targets.concurrent_policy = "shared".to_string();
        assert!(validate_ci_build_authority(&authority).is_err());
    }

    #[test]
    fn rejects_cache_metrics_that_disappear_on_failure() {
        let mut authority = authority();
        authority.cache.publish_on_failure = false;
        assert!(validate_ci_build_authority(&authority).is_err());
    }

    #[test]
    fn rejects_unpinned_test_executor() {
        let mut authority = authority();
        authority.test_executor.version.clear();
        assert!(validate_ci_build_authority(&authority).is_err());
    }

    #[test]
    fn rejects_excessive_codegen_parallelism() {
        let mut authority = authority();
        authority.compilation.dev_codegen_units = 256;
        assert!(validate_ci_build_authority(&authority).is_err());
    }
}
