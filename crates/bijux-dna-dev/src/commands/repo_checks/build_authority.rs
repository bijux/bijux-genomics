use anyhow::{ensure, Context, Result};
use bijux_dna_policies::{validate_ci_build_authority, CiBuildAuthority};

use crate::commands::command_support::{pass, run_command};
use crate::model::check::{CheckDefinition, CheckOutcome};
use crate::runtime::workspace::Workspace;

const AUTHORITY_PATH: &str = "configs/ci/build-authority.json";
const WORKFLOW_PATH: &str = ".github/workflows/ci.yml";
const RUNNER_PATH: &str = "makes/bin/run_github_workflow_gate.sh";
const CARGO_MAKE_PATH: &str = "makes/cargo.mk";

pub(crate) fn check_ci_build_authority(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let raw = std::fs::read_to_string(workspace.path(AUTHORITY_PATH))
        .with_context(|| format!("read {AUTHORITY_PATH}"))?;
    let authority: CiBuildAuthority =
        serde_json::from_str(&raw).with_context(|| format!("parse {AUTHORITY_PATH}"))?;
    validate_ci_build_authority(&authority)?;

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
    ensure!(
        workflow.contains(&format!("RUST_TOOLCHAIN_VERSION: {}", authority.toolchain.channel)),
        "GitHub workflow toolchain differs from the build authority"
    );
    ensure!(
        workflow.contains(&format!("RUSTC_WRAPPER: {}", authority.cache.compiler_wrapper)),
        "GitHub workflow compiler wrapper differs from the build authority"
    );

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
    for target in ["_ci-fast", "_ci-slow"] {
        let body = make_target_body(&cargo_make, target)?;
        ensure!(
            body.contains("checks run check-ci-build-authority"),
            "{target} must validate the CI build authority"
        );
    }

    pass(
        check,
        format!(
            "validated Rust {}, locked metadata, sccache JSON receipts, and isolated concurrent targets",
            authority.toolchain.channel
        ),
    )
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
