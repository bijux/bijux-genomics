use anyhow::{ensure, Context, Result};
use bijux_dna_policies::{validate_ci_build_authority, CiBuildAuthority};

use crate::commands::command_support::{pass, run_command};
use crate::model::check::{CheckDefinition, CheckOutcome};
use crate::runtime::workspace::Workspace;

const AUTHORITY_PATH: &str = "configs/ci/build-authority.json";
const WORKFLOW_PATH: &str = ".github/workflows/ci.yml";
const RUNNER_PATH: &str = "makes/bin/run_github_workflow_gate.sh";
const CARGO_MAKE_PATH: &str = "makes/cargo.mk";
const FAST_GROUP_PATH: &str = "configs/ci/fast-groups.json";

fn ensure_no_legacy_build_authority(workspace: &Workspace) -> Result<()> {
    ensure!(
        !workspace.path("configs/runtime").join("cargo_build.toml").exists(),
        "legacy runtime Cargo build authority must not coexist with {AUTHORITY_PATH}"
    );
    Ok(())
}

pub(crate) fn check_ci_build_authority(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let raw = std::fs::read_to_string(workspace.path(AUTHORITY_PATH))
        .with_context(|| format!("read {AUTHORITY_PATH}"))?;
    let authority: CiBuildAuthority =
        serde_json::from_str(&raw).with_context(|| format!("parse {AUTHORITY_PATH}"))?;
    validate_ci_build_authority(&authority)?;
    ensure_no_legacy_build_authority(workspace)?;

    let toolchain_raw = std::fs::read_to_string(workspace.path(&authority.toolchain.manifest))
        .context("read CI toolchain manifest")?;
    let toolchain: toml::Value =
        toml::from_str(&toolchain_raw).context("parse CI toolchain manifest")?;
    let declared_channel = toolchain
        .get("toolchain")
        .and_then(|value| value.get("channel"))
        .and_then(toml::Value::as_str)
        .context("toolchain manifest omits its channel")?;
    ensure!(
        declared_channel == authority.toolchain.channel,
        "build authority channel differs from rust-toolchain.toml"
    );
    let mut declared_components = toolchain
        .get("toolchain")
        .and_then(|value| value.get("components"))
        .and_then(toml::Value::as_array)
        .context("toolchain manifest omits its components")?
        .iter()
        .map(|value| value.as_str().context("toolchain component is not a string"))
        .collect::<Result<Vec<_>>>()?;
    declared_components.sort_unstable();
    let mut required_components =
        authority.toolchain.required_components.iter().map(String::as_str).collect::<Vec<_>>();
    required_components.sort_unstable();
    ensure!(
        declared_components == required_components,
        "build authority components differ from rust-toolchain.toml"
    );

    let compilation_raw = std::fs::read_to_string(workspace.path(&authority.compilation.manifest))
        .context("read CI compilation manifest")?;
    let compilation: toml::Value =
        toml::from_str(&compilation_raw).context("parse CI compilation manifest")?;
    for (profile, expected_codegen_units) in [
        ("dev", authority.compilation.dev_codegen_units),
        ("test", authority.compilation.test_codegen_units),
    ] {
        let profile_table = compilation
            .get("profile")
            .and_then(|value| value.get(profile))
            .with_context(|| format!("compilation manifest omits profile.{profile}"))?;
        let codegen_units = profile_table
            .get("codegen-units")
            .and_then(toml::Value::as_integer)
            .with_context(|| format!("profile.{profile} omits integer codegen-units"))?;
        ensure!(
            codegen_units == i64::from(expected_codegen_units),
            "profile.{profile} codegen-units differs from the build authority"
        );
        let incremental = profile_table
            .get("incremental")
            .and_then(toml::Value::as_bool)
            .with_context(|| format!("profile.{profile} omits incremental policy"))?;
        ensure!(
            incremental == authority.compilation.incremental,
            "profile.{profile} incremental policy differs from the build authority"
        );
    }

    let lockfile = std::fs::read_to_string(workspace.path(&authority.lockfile.manifest))
        .context("read workspace Cargo lock")?;
    ensure!(lockfile.lines().any(|line| line.trim() == "version = 4"), "unsupported Cargo.lock");
    let metadata = run_command(
        workspace,
        "cargo",
        &["metadata", "--format-version", "1", "--locked", "--no-deps"],
    )?;
    ensure!(metadata.status.success(), "locked cargo metadata failed");

    let workflow = std::fs::read_to_string(workspace.path(WORKFLOW_PATH))
        .with_context(|| format!("read {WORKFLOW_PATH}"))?;
    validate_workflow_environment(
        &workflow,
        &[
            ("RUST_TOOLCHAIN_VERSION", authority.toolchain.channel.as_str()),
            ("RUSTC_WRAPPER", authority.cache.compiler_wrapper.as_str()),
            ("CARGO_NEXTEST_VERSION", authority.test_executor.version.as_str()),
        ],
    )?;

    let runner = std::fs::read_to_string(workspace.path(RUNNER_PATH))
        .with_context(|| format!("read {RUNNER_PATH}"))?;
    for marker in [
        "require_tool sccache",
        "sccache --show-stats --stats-format=json",
        "publish_sccache_metrics before",
        "trap publish_final_metrics EXIT",
        "gate_target_dir=\"${artifact_dir}/targets/${gate_name}\"",
        "CARGO_TARGET_DIR=\"${gate_target_dir}\"",
        "DEV_DNA_BIN=\"${shared_dev_bin}\"",
    ] {
        ensure!(
            runner.contains(marker),
            "GitHub gate runner omits build-authority marker: {marker}"
        );
    }

    let cargo_make = std::fs::read_to_string(workspace.path(CARGO_MAKE_PATH))
        .with_context(|| format!("read {CARGO_MAKE_PATH}"))?;
    let fast_body = make_target_body(&cargo_make, "_ci-fast")?;
    ensure!(
        fast_body.contains("tooling run ci-fast"),
        "_ci-fast must delegate to the native DAG supervisor"
    );
    let fast_groups = std::fs::read_to_string(workspace.path(FAST_GROUP_PATH))
        .with_context(|| format!("read {FAST_GROUP_PATH}"))?;
    ensure!(
        fast_groups.contains("_ci-authority-fast"),
        "ci-fast groups must include the build-authority target"
    );
    let slow_body = make_target_body(&cargo_make, "_ci-slow")?;
    ensure!(
        slow_body.contains("checks run check-ci-build-authority"),
        "_ci-slow must validate the CI build authority"
    );

    pass(
        check,
        format!(
            "validated Rust {}, cargo-nextest {}, {}-unit dev/test codegen, locked metadata, sccache JSON receipts, and isolated concurrent targets",
            authority.toolchain.channel,
            authority.test_executor.version,
            authority.compilation.dev_codegen_units
        ),
    )
}

fn validate_workflow_environment(source: &str, required: &[(&str, &str)]) -> Result<()> {
    let workflow: serde_yaml::Value =
        serde_yaml::from_str(source).context("parse GitHub workflow YAML")?;
    let environment = workflow
        .get("env")
        .and_then(serde_yaml::Value::as_mapping)
        .context("GitHub workflow omits its global env mapping")?;
    for (name, expected) in required {
        let declared =
            environment.get(*name).and_then(serde_yaml::Value::as_str).with_context(|| {
                format!("GitHub workflow env omits string {name} required by {AUTHORITY_PATH}")
            })?;
        ensure!(
            declared == *expected,
            "GitHub workflow env {name} differs from the build authority: expected {expected}, observed {declared}"
        );
    }
    Ok(())
}

fn make_target_body(source: &str, target: &str) -> Result<String> {
    let marker = format!("{target}:");
    let mut lines = source.lines();
    ensure!(lines.by_ref().any(|line| line.starts_with(&marker)), "missing Make target {target}");
    let body = lines
        .take_while(|line| line.starts_with('\t') || line.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    ensure!(!body.trim().is_empty(), "Make target {target} has no recipe");
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_build_authority_must_remain_absent() -> Result<()> {
        let root = tempfile::tempdir()?;
        let workspace = Workspace { root: root.path().to_path_buf() };
        ensure_no_legacy_build_authority(&workspace)?;
        let legacy = workspace.path("configs/runtime").join("cargo_build.toml");
        std::fs::create_dir_all(legacy.parent().context("legacy parent")?)?;
        std::fs::write(&legacy, "preserved obsolete authority")?;
        let error = ensure_no_legacy_build_authority(&workspace)
            .err()
            .context("legacy authority must fail")?;
        assert_eq!(
            error.to_string(),
            "legacy runtime Cargo build authority must not coexist with configs/ci/build-authority.json"
        );
        assert_eq!(std::fs::read_to_string(legacy)?, "preserved obsolete authority");
        Ok(())
    }

    const REQUIRED_ENV: &[(&str, &str)] = &[
        ("RUST_TOOLCHAIN_VERSION", "1.95.0"),
        ("RUSTC_WRAPPER", "sccache"),
        ("CARGO_NEXTEST_VERSION", "0.9.100"),
    ];

    #[test]
    fn workflow_environment_accepts_equivalent_yaml_quoting() -> Result<()> {
        for yaml in [
            "env:\n  RUST_TOOLCHAIN_VERSION: 1.95.0\n  RUSTC_WRAPPER: sccache\n  CARGO_NEXTEST_VERSION: 0.9.100\n",
            "\"env\":\n  \"RUST_TOOLCHAIN_VERSION\": \"1.95.0\"\n  \"RUSTC_WRAPPER\": \"sccache\"\n  \"CARGO_NEXTEST_VERSION\": \"0.9.100\"\n",
            "'env':\n  'RUST_TOOLCHAIN_VERSION': '1.95.0'\n  'RUSTC_WRAPPER': 'sccache'\n  'CARGO_NEXTEST_VERSION': '0.9.100'\n",
        ] {
            validate_workflow_environment(yaml, REQUIRED_ENV)?;
        }
        Ok(())
    }

    #[test]
    fn workflow_environment_rejects_wrong_missing_or_non_string_authority() -> Result<()> {
        for &(changed, expected) in REQUIRED_ENV {
            let mut yaml = format!("# {changed}: {expected}\nenv:\n");
            for &(name, value) in REQUIRED_ENV {
                let actual = if name == changed { "incorrect" } else { value };
                yaml.push_str(&format!("  {name}: {actual}\n"));
            }
            let error = validate_workflow_environment(&yaml, REQUIRED_ENV)
                .err()
                .context("wrong value must fail")?;
            assert!(error.to_string().contains(changed));
            assert!(error.to_string().contains("differs from the build authority"));
        }
        let base = "env:\n  RUST_TOOLCHAIN_VERSION: 1.95.0\n  RUSTC_WRAPPER: sccache\n";
        for suffix in [
            "",
            "jobs:\n  test:\n    env:\n      CARGO_NEXTEST_VERSION: 0.9.100\n",
            "  CARGO_NEXTEST_VERSION: 100\n",
        ] {
            let error = validate_workflow_environment(&format!("{base}{suffix}"), REQUIRED_ENV)
                .err()
                .context("missing global string pin must fail")?;
            assert!(error.to_string().contains("CARGO_NEXTEST_VERSION"));
        }
        assert!(validate_workflow_environment("env: [", REQUIRED_ENV).is_err());
        assert!(validate_workflow_environment("env: scalar", REQUIRED_ENV).is_err());
        Ok(())
    }
}
