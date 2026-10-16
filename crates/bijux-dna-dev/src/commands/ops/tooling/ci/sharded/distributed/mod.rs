//! Build once, execute isolated declared partitions, and reconcile their exact union.

mod aggregate;
mod consumer;
mod producer;
mod protocol;

#[cfg(test)]
mod contract_tests;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{ensure, Context, Result};
use serde_json::{json, Value};

use crate::commands::ops::tooling::ci::ci_output_path;
use crate::commands::ops::tooling::write_json_pretty;
use crate::model::ops::OpsCommandOutcome;
use crate::runtime::owned_artifacts::remove_owned_tree;
use crate::runtime::process::{ParallelProcessSpec, ProcessRunner};
use crate::runtime::source_snapshot::{IsolatedSourceIdentity, TrackedSourceSnapshot};
use crate::runtime::workspace::Workspace;

use super::contract::{validate_full_shard_contract, FullShardContract};
use super::preflight::{
    observe_full_shard_runtime_dependencies, require_ci_test_executor,
    validate_full_shard_runtime_dependency_catalog,
};
use protocol::{fingerprint, matrix, FileInput, Matrix, Producer};

struct Admission {
    condition: IsolatedSourceIdentity,
    snapshot: TrackedSourceSnapshot,
    contract: FullShardContract,
    compatibility: Value,
    matrix: Matrix,
}

struct Session {
    admission: Option<Admission>,
    scratch: PathBuf,
    protected: BTreeMap<PathBuf, FileInput>,
}

pub(super) fn run(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    let root = ci_output_path(workspace, "artifacts/ci/distributed")?;
    let (mode, partition) = match args {
        [mode] if mode == "--github-producer" || mode == "--github-aggregate" => {
            (mode.as_str(), None)
        }
        [mode, partition] if mode == "--github-consumer" => {
            (mode.as_str(), Some(partition.as_str()))
        }
        _ => anyhow::bail!(
            "expected --github-producer, --github-consumer hash:m/n, or --github-aggregate"
        ),
    };
    let id = partition.map(partition_id).transpose()?;
    let reports = match mode {
        "--github-producer" => root.join("build"),
        "--github-consumer" => root.join("shards").join(id.as_ref().context("partition absent")?),
        _ => root.join("aggregate"),
    };
    std::fs::create_dir_all(&reports)?;
    let receipt_path = reports.join("receipt.json");
    let scratch =
        root.join("scratch").join(id.as_deref().unwrap_or(if mode == "--github-aggregate" {
            "aggregate"
        } else {
            "producer"
        }));
    let mut session = Session { admission: None, scratch, protected: BTreeMap::new() };
    crate::runtime::gate_receipt::run_reported(
        &receipt_path,
        json!({"schema_version":"bijux.ci.distributed_operation.v1", "mode":mode,"all_passed":false}),
        Instant::now(),
        &mut session,
        |session, receipt| {
            let admission = admit(workspace, &reports, mode != "--github-aggregate")?;
            receipt["source_condition"] = admission.condition.descriptor();
            session.admission = Some(admission);
            match mode {
                "--github-producer" => {
                    producer::produce(workspace, &root, &reports, session, receipt)
                }
                "--github-consumer" => consumer::consume(
                    workspace,
                    &root,
                    &reports,
                    partition.context("partition absent")?,
                    session,
                    receipt,
                ),
                _ => aggregate::aggregate(workspace, &root, session, receipt),
            }
        },
        |session, receipt| finish(workspace, session, receipt),
    )?;
    Ok(OpsCommandOutcome::success(format!(
        "distributed test gate passed; receipt: {}\n",
        receipt_path.display()
    )))
}

fn partition_id(partition: &str) -> Result<String> {
    let (index, count) = partition
        .strip_prefix("hash:")
        .and_then(|rest| rest.split_once('/'))
        .context("expected hash:m/n")?;
    ensure!(
        index.bytes().all(|byte| byte.is_ascii_digit())
            && count.bytes().all(|byte| byte.is_ascii_digit()),
        "partition contains nondecimal identity"
    );
    let index: usize = index.parse()?;
    let count: usize = count.parse()?;
    let matrix = matrix(count)?;
    let expected = format!("hash:{index}/{count}");
    ensure!(partition == expected, "partition is not canonical");
    matrix
        .include
        .into_iter()
        .find(|row| row.partition == partition)
        .map(|row| row.id)
        .context("partition is outside declared hash space")
}

fn config_hashes(workspace: &Workspace) -> Result<Value> {
    let mut hashes = BTreeMap::new();
    for relative in [
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        ".cargo/config.toml",
        "configs/rust/nextest.toml",
        "configs/ci/build-authority.json",
        "configs/ci/full-shards.json",
    ] {
        hashes.insert(relative, bijux_dna_infra::hash_file_sha256(&workspace.path(relative))?);
    }
    Ok(serde_json::to_value(hashes)?)
}

fn tool_version(workspace: &Workspace, program: &str, args: &[&str]) -> Result<String> {
    let output = ProcessRunner::new(workspace)
        .run_owned(program, &args.iter().map(ToString::to_string).collect::<Vec<_>>())?;
    ensure!(
        output.status.success(),
        "cannot observe {program} compatibility: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

fn admit(workspace: &Workspace, reports: &Path, observe_tools: bool) -> Result<Admission> {
    let snapshot = TrackedSourceSnapshot::capture(workspace)?;
    ensure!(snapshot.is_clean(), "distributed tests require a clean committed checkout");
    let condition = IsolatedSourceIdentity::capture(workspace)?;
    condition.require_snapshot(&snapshot)?;
    let contract: FullShardContract =
        serde_json::from_slice(&std::fs::read(workspace.path("configs/ci/full-shards.json"))?)?;
    validate_full_shard_contract(&contract)?;
    let matrix = matrix(contract.ci_partition_count.unwrap_or(20))?;
    let mut compatibility = json!({"os":std::env::consts::OS,"architecture":std::env::consts::ARCH,"cargo_profile":"ci","nextest_profile":"full","workspace":true,"all_targets":true,"all_features":true,"locked":true,"run_ignored":"all","config_sha256":config_hashes(workspace)?,"cargo_build_target":std::env::var("CARGO_BUILD_TARGET").ok(),"rustflags":std::env::var("RUSTFLAGS").ok(),"encoded_rustflags":std::env::var("CARGO_ENCODED_RUSTFLAGS").ok()});
    if observe_tools {
        let authority =
            super::super::read_ci_build_authority(&workspace.path(&contract.build_authority))?;
        require_ci_test_executor(workspace, &authority)?;
        validate_full_shard_runtime_dependency_catalog(workspace, &contract.runtime_dependencies)?;
        compatibility["rustc"] = json!(tool_version(workspace, "rustc", &["-vV"])?);
        compatibility["cargo"] = json!(tool_version(workspace, "cargo", &["--version"])?);
        compatibility["nextest"] =
            json!(tool_version(workspace, "cargo", &["nextest", "--version"])?);
        compatibility["runtime"] = json!(observe_full_shard_runtime_dependencies(
            workspace,
            &contract.runtime_dependencies,
            &reports.join("runtime-dependencies.json")
        )?);
    }
    Ok(Admission { condition, snapshot, contract, compatibility, matrix })
}

fn command(
    workspace: &Workspace,
    reports: &Path,
    label: &str,
    args: Vec<String>,
    envs: Vec<(String, String)>,
) -> Result<()> {
    let output = ProcessRunner::new(workspace)
        .run_parallel_logged(
            vec![ParallelProcessSpec {
                label: label.to_owned(),
                program: "cargo".to_owned(),
                args,
                envs,
                current_dir: Some(workspace.root.clone()),
            }],
            reports,
        )?
        .pop()
        .context("command was not collected")?;
    ensure!(
        output.output.status.success(),
        "{label} failed; see {}",
        reports.join(format!("{label}.stderr.log")).display()
    );
    Ok(())
}

fn command_recorded(
    workspace: &Workspace,
    reports: &Path,
    label: &str,
    args: Vec<String>,
    envs: Vec<(String, String)>,
    receipt: &mut Value,
) -> Result<()> {
    let output = ProcessRunner::new(workspace)
        .run_parallel_logged(
            vec![ParallelProcessSpec {
                label: label.to_owned(),
                program: "cargo".to_owned(),
                args: args.clone(),
                envs,
                current_dir: Some(workspace.root.clone()),
            }],
            reports,
        )?
        .pop()
        .context("command was not collected")?;
    receipt[label] = json!({"state":"complete","exit_code":output.output.status.code(),
        "argv":args,"duration_milliseconds":output.duration.as_millis()});
    ensure!(
        output.output.status.success(),
        "{label} failed; see {}",
        reports.join(format!("{label}.stderr.log")).display()
    );
    Ok(())
}

fn protect(session: &mut Session, paths: impl IntoIterator<Item = PathBuf>) -> Result<()> {
    for path in paths {
        session.protected.insert(path.clone(), fingerprint(&path)?);
    }
    Ok(())
}

fn load_producer(root: &Path, session: &mut Session, observe_tools: bool) -> Result<Producer> {
    let build = root.join("build");
    let producer_path = build.join("producer.json");
    let producer: Producer = serde_json::from_slice(&std::fs::read(&producer_path)?)?;
    let admission = session.admission.as_ref().context("source admission absent")?;
    if !observe_tools {
        for field in [
            "os",
            "architecture",
            "cargo_profile",
            "nextest_profile",
            "workspace",
            "all_targets",
            "all_features",
            "locked",
            "run_ignored",
            "config_sha256",
            "cargo_build_target",
            "rustflags",
            "encoded_rustflags",
        ] {
            ensure!(
                producer.compatibility[field] == admission.compatibility[field],
                "aggregate source/platform configuration changed: {field}"
            );
        }
    }
    let compatibility =
        if observe_tools { &admission.compatibility } else { &producer.compatibility };
    protocol::admit_producer(
        &producer,
        &admission.condition.descriptor(),
        compatibility,
        &admission.matrix,
    )?;
    for (name, expected) in [
        ("archive.tar.zst", &producer.archive_sha256),
        ("inventory.json", &producer.inventory_sha256),
        ("matrix.json", &producer.matrix_sha256),
        ("bijux-dna-dev", &producer.runner_sha256),
    ] {
        ensure!(
            bijux_dna_infra::hash_file_sha256(&build.join(name))? == *expected,
            "transported {name} changed"
        );
    }
    let matrix: Matrix = serde_json::from_slice(&std::fs::read(build.join("matrix.json"))?)?;
    ensure!(matrix == producer.matrix, "transported matrix differs from descriptor");
    let receipt: Value = serde_json::from_slice(&std::fs::read(build.join("receipt.json"))?)?;
    ensure!(
        receipt["state"] == "complete"
            && receipt["all_passed"] == true
            && receipt["producer_sha256"] == bijux_dna_infra::hash_file_sha256(&producer_path)?
            && receipt["source_postcondition"]["unchanged"] == true
            && receipt["cleanup"]["completed"] == true
            && receipt["producer_input_postcondition"]["checksum_stable"] == true
            && receipt["extracted_input_postcondition"]["checksum_stable"] == true
            && receipt["transport_inputs_unchanged"] == true,
        "producer did not complete its source/input transaction"
    );
    ensure!(
        receipt["doctests"]["state"] == "complete" && receipt["doctests"]["exit_code"] == 0,
        "producer doctests did not pass"
    );
    protect(
        session,
        [
            "producer.json",
            "archive.tar.zst",
            "inventory.json",
            "matrix.json",
            "bijux-dna-dev",
            "receipt.json",
        ]
        .map(|name| build.join(name)),
    )?;
    Ok(producer)
}

fn finish(workspace: &Workspace, session: &mut Session, receipt: &mut Value) -> Result<()> {
    let source =
        session.admission.as_ref().map(|admission| admission.condition.postcondition(workspace));
    receipt["source_postcondition"] = json!(source);
    let inputs = session
        .protected
        .iter()
        .all(|(path, expected)| fingerprint(path).is_ok_and(|actual| actual == *expected));
    receipt["transport_inputs_unchanged"] = json!(inputs);
    let started = Instant::now();
    let cleanup = remove_owned_tree(&session.scratch.join("run"));
    receipt["cleanup"] = json!({"completed":cleanup.is_ok(),"duration_milliseconds":started.elapsed().as_millis(),"error":cleanup.as_ref().err().map(|error|format!("{error:#}"))});
    cleanup?;
    ensure!(
        inputs && source.is_some_and(|value| value["unchanged"] == true),
        "distributed source or transported inputs changed"
    );
    Ok(())
}
