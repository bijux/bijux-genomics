//! Complete local test execution with exact native lane membership and one writer lease.

mod execution;
mod nextest;
mod python;
mod rustdoc;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::application::ci::local_tests::{
    admit_lane, admit_python_cases, doctest_inventory, doctest_outcomes, doctest_targets,
    nextest_membership, reconcile_lanes, CaseOutcome, LaneDefinition, LaneInputs, LaneReceipt,
    SourceIdentity, ToolIdentity, LANE_SCHEMA, REQUIRED_LANES,
};
use crate::model::ops::OpsCommandOutcome;
use crate::runtime::compilation_cache::CompilationCache;
use crate::runtime::gate_receipt::run_reported;
use crate::runtime::local_tests::{
    bind_tools, lane_inputs, load_catalog, observe_capacity, reference, selections,
    source_identity, tool_path, verify_artifacts, verify_inputs, verify_producer_manifest,
    verify_source_reads, write_value,
};
use crate::runtime::process::{
    DependencyProcessOutput, DependentProcessSpec, ParallelProcessSpec, ProcessRunner,
    WeightedProcessSpec,
};
use crate::runtime::source_snapshot::TrackedSourceSnapshot;
use crate::runtime::workspace::Workspace;
use execution::{
    execute_lane, input_manifest, lane_environment, native_command, terminal, ExecutionPlan,
};

pub(in super::super::super) fn tooling_ci_test_complete(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    if matches!(args, [arg] if arg == "--help" || arg == "-h") {
        return Ok(OpsCommandOutcome::success("ci-test-complete: run all declared local Rust, doctest and Python test lanes; BIJUX_CI_COMPLETE_WORKER_BUDGET limits CPU units (one hardware worker reserved by default).\n"));
    }
    if let [mode, lane, plan] = args {
        ensure!(
            mode == "--lane" && REQUIRED_LANES.contains(&lane.as_str()),
            "invalid complete local test worker invocation"
        );
        let path = workspace.admit_artifact_path(Path::new(plan))?;
        let plan: ExecutionPlan = serde_json::from_slice(&std::fs::read(path)?)?;
        ensure!(
            plan.schema_version == "bijux.ci.local_test_execution.v1"
                && plan.workspace == workspace.root,
            "worker plan belongs to another source workspace"
        );
        return execute_lane(workspace, lane, &plan);
    }
    ensure!(args.is_empty(), "ci-test-complete accepts no public positional arguments");
    let started = Instant::now();
    let receipt_path =
        workspace.admit_artifact_path(Path::new("artifacts/ci/local-test-gate.json"))?;
    bijux_dna_infra::ensure_dir(receipt_path.parent().context("complete receipt owner")?)?;
    let owner = workspace.admit_artifact_path(Path::new("artifacts/ci/local-tests"))?;
    bijux_dna_infra::ensure_dir(&owner)?;
    let mut coordinator = bijux_dna_infra::FileLock::try_acquire(&owner.join("coordinator.lock"))?;
    run_reported(
        &receipt_path,
        json!({"schema_version":"bijux.ci.local_test_gate.v1","all_passed":false}),
        started,
        &mut coordinator,
        |_, receipt| {
            // Admission runs after replacement of the previous green receipt. No invalid source,
            // runtime, catalog or capacity can leave stale successful aggregate evidence behind.
            receipt["capacity_observation"] = observe_capacity(workspace, &owner)?;
            let source = source_identity(workspace)?;
            let snapshot = TrackedSourceSnapshot::capture(workspace)?;
            ensure!(snapshot.is_clean(), "complete local tests require committed clean source for reusable Cargo fingerprints");
            let mut catalog = load_catalog(&workspace.root)?;
            let hardware = std::thread::available_parallelism()?.get();
            let budget = std::env::var("BIJUX_CI_COMPLETE_WORKER_BUDGET").map_or(
                Ok(hardware.saturating_sub(1).max(1)),
                |value| {
                    value
                        .parse::<usize>()
                        .context("complete worker budget must be a positive integer")
                },
            )?;
            ensure!(
                budget > 0
                    && budget < hardware
                    && catalog.lanes.iter().all(|lane| lane.resource_units <= budget),
                "complete lane exceeds admitted CPU budget"
            );
            let python_units = catalog
                .lanes
                .iter()
                .filter(|lane| lane.id.starts_with("python-"))
                .map(|lane| lane.resource_units)
                .sum::<usize>();
            let rust_lane = catalog
                .lanes
                .iter_mut()
                .find(|lane| lane.id == "rust-nextest")
                .context("Nextest lane absent")?;
            rust_lane.resource_units =
                budget.saturating_sub(python_units).max(rust_lane.resource_units);
            let full: Value = serde_json::from_slice(&std::fs::read(
                workspace.path("configs/ci/full-shards.json"),
            )?)?;
            ensure!(
                full["archive_build_jobs"].as_u64().context("archive build jobs absent")?
                    <= budget as u64,
                "archive producer exceeds complete CPU budget"
            );
            let isolated_root = owner.join("source");
            snapshot.materialize_cached_clean(&isolated_root)?;
            let isolated = Workspace { root: isolated_root };
            let runs = isolated.path("artifacts/ci/local-tests");
            bijux_dna_infra::ensure_dir(&runs)?;
            let output = runs.join(format!(
                "run-{}-{}",
                SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(),
                std::process::id()
            ));
            std::fs::create_dir(&output)?;
            let tool_names = catalog
                .lanes
                .iter()
                .flat_map(|lane| lane.tools.iter().cloned())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            let mut tools = Vec::new();
            let mut tool_errors = Vec::new();
            for name in tool_names {
                match bind_tools(workspace, std::slice::from_ref(&name)) {
                    Ok(bound) => tools.extend(bound),
                    Err(error) => tool_errors.push((name, format!("{error:#}"))),
                }
            }
            let directories = tools
                .iter()
                .filter_map(|tool| Path::new(&tool.path).parent())
                .collect::<BTreeSet<_>>();
            let mut search = directories.into_iter().map(Path::to_path_buf).collect::<Vec<_>>();
            search.extend(std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()));
            let mut environment = vec![
                (
                    "PATH".into(),
                    std::env::join_paths(search)?.to_str().context("tool PATH encoding")?.into(),
                ),
                ("PYTHONNOUSERSITE".into(), "1".into()),
                ("PYTHONDONTWRITEBYTECODE".into(), "1".into()),
                ("PYTEST_DISABLE_PLUGIN_AUTOLOAD".into(), "1".into()),
                ("PYTEST_ADDOPTS".into(), String::new()),
                ("PYTEST_PLUGINS".into(), String::new()),
                ("PYTHONHASHSEED".into(), "0".into()),
                ("PYTHONOPTIMIZE".into(), "0".into()),
                ("RUSTC_WORKSPACE_WRAPPER".into(), String::new()),
                ("PYTHONPATH".into(), String::new()),
                ("RUSTC_WRAPPER".into(), String::new()),
                ("CARGO_BUILD_JOBS".into(), "2".into()),
            ];
            for (key, name) in [
                ("PYTHON", "python"),
                ("BIJUX_GATE_TEST_BASH", "bash"),
                ("RUSTC", "rustc"),
                ("RUSTDOC", "rustdoc"),
            ] {
                if let Ok(path) = tool_path(&tools, name) {
                    environment.push((key.into(), path.into()));
                }
            }
            let scratch = owner.join("rust-products");
            let plan = ExecutionPlan {
                schema_version: "bijux.ci.local_test_execution.v1".into(),
                source: source.clone(),
                workspace: isolated.root.clone(),
                output: output.clone(),
                scratch,
                cache: owner.join("qualified-python"),
                lanes: catalog.lanes,
                tools,
                tool_errors,
                environment,
                worker_budget: budget,
            };
            let plan_path = output.join("execution-plan.json");
            write_value(&plan_path, &plan)?;
            receipt["source"] = json!(source);
            receipt["execution_plan"] = json!(reference(&plan_path)?);
            receipt["resources"] = json!({"cpu_units":budget,"hardware_workers":hardware,"foreground_reserve":hardware.saturating_sub(budget),"process_limit":5,"rust_writer_policy":"terminally_sequenced_shared_cache","memory_io_qualification":"external runner admission required; CPU units do not enforce RAM/disk/IO limits","shared_compilation_writers":1,"independent_python_fixture_compile_jobs":2,"memory_budget_enforced":false,"disk_io_budget_enforced":false});
            let executable = std::env::current_exe()?.canonicalize()?;
            let specs = plan
                .lanes
                .iter()
                .map(|lane| {
                    let mut envs = plan.environment.clone();
                    envs.push(("BIJUX_REPO_ROOT".into(), isolated.root.display().to_string()));
                    WeightedProcessSpec {
                        process: DependentProcessSpec {
                            process: ParallelProcessSpec {
                                label: lane.id.clone(),
                                program: executable.display().to_string(),
                                args: vec![
                                    "tooling".into(),
                                    "run".into(),
                                    "ci-test-complete".into(),
                                    "--lane".into(),
                                    lane.id.clone(),
                                    plan_path.display().to_string(),
                                ],
                                envs,
                                current_dir: Some(isolated.root.clone()),
                            },
                            dependencies: vec![],
                        },
                        resource_units: lane.resource_units,
                        terminal_dependencies: if lane.id == "rust-doctests" {
                            vec!["rust-nextest".into()]
                        } else {
                            vec![]
                        },
                    }
                })
                .collect();
            let completed =
                ProcessRunner::new(&isolated).run_weighted_dependency_graph(specs, 5, budget)?;
            let mut failed_processes = Vec::new();
            for result in completed {
                match result {
                    DependencyProcessOutput::Completed(result) => {
                        bijux_dna_infra::write_bytes(
                            output.join(format!("{}.stdout.log", result.label)),
                            &result.output.stdout,
                        )?;
                        bijux_dna_infra::write_bytes(
                            output.join(format!("{}.stderr.log", result.label)),
                            &result.output.stderr,
                        )?;
                        if !result.output.status.success() {
                            failed_processes.push(result.label);
                        }
                    }
                    DependencyProcessOutput::BlockedDependency { label } => {
                        failed_processes.push(label);
                    }
                }
            }
            let mut receipts = Vec::new();
            let mut terminal_records = Vec::new();
            let mut acceptance_errors = Vec::new();
            let mut inputs = Vec::new();
            for lane in &plan.lanes {
                let path = output.join(&lane.id).join("receipt.json");
                let acceptance = (|| -> Result<(LaneReceipt, LaneInputs)> {
                    let bytes = std::fs::read(&path)
                        .with_context(|| format!("missing terminal lane: {}", lane.id))?;
                    terminal_records.push(json!({"lane":lane.id,"receipt":reference(&path)?,"terminal":serde_json::from_slice::<Value>(&bytes)?}));
                    let evidence: LaneReceipt = serde_json::from_slice(&bytes)?;
                    verify_artifacts(&evidence)?;
                    verify_inputs(&isolated.root, &evidence.inputs)?;
                    let mut admitted =
                        input_manifest(&isolated, lane, &plan, evidence.expected.clone())?;
                    admitted.closure_qualified = true;
                    if lane.id.starts_with("python-") {
                        python::validate_native_cases(&plan, &evidence, &admitted)?;
                    }
                    Ok((evidence, admitted))
                })();
                match acceptance {
                    Ok((evidence, admitted)) => {
                        inputs.push(admitted);
                        receipts.push(evidence);
                    }
                    Err(error) => acceptance_errors.push(format!("{}: {error:#}", lane.id)),
                }
            }
            receipt["lanes"] = json!(receipts);
            receipt["terminal_records"] = json!(terminal_records);
            receipt["acceptance_errors"] = json!(acceptance_errors);
            receipt["tool_admission_errors"] = json!(plan.tool_errors);
            receipt["failed_processes"] = json!(failed_processes);
            ensure!(
                source_identity(workspace)? == source && source_identity(&isolated)? == source,
                "source changed during complete local tests"
            );
            ensure!(
                acceptance_errors.is_empty(),
                "local lane receipt admission failed: {}",
                acceptance_errors.join("; ")
            );
            reconcile_lanes(&receipts, &source, &inputs)?;
            ensure!(failed_processes.is_empty(), "local lane process failed");
            receipt["all_passed"] = json!(true);
            Ok(OpsCommandOutcome::success(format!(
                "all complete local test lanes passed; receipt: {}\n",
                receipt_path.display()
            )))
        },
        |_, _| Ok(()),
    )
}
