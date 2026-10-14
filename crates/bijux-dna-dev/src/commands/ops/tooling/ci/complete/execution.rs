//! Lane process boundaries and the immutable complete-local execution plan.

use super::{
    admit_lane, ensure, json, lane_inputs, nextest, python, reference, rustdoc, write_value,
    BTreeSet, CaseOutcome, Context, Deserialize, Instant, LaneDefinition, LaneInputs, LaneReceipt,
    OpsCommandOutcome, ParallelProcessSpec, Path, PathBuf, ProcessRunner, Result, Serialize,
    SourceIdentity, ToolIdentity, Workspace, LANE_SCHEMA,
};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ExecutionPlan {
    pub(super) schema_version: String,
    pub(super) source: SourceIdentity,
    pub(super) workspace: PathBuf,
    pub(super) output: PathBuf,
    pub(super) scratch: PathBuf,
    pub(super) cache: PathBuf,
    pub(super) lanes: Vec<LaneDefinition>,
    pub(super) tools: Vec<ToolIdentity>,
    pub(super) tool_errors: Vec<(String, String)>,
    pub(super) environment: Vec<(String, String)>,
    pub(super) worker_budget: usize,
}

pub(super) fn input_manifest(
    workspace: &Workspace,
    lane: &LaneDefinition,
    plan: &ExecutionPlan,
    expected: Vec<String>,
) -> Result<LaneInputs> {
    let tools = plan
        .tools
        .iter()
        .filter(|tool| lane.tools.contains(&tool.name))
        .cloned()
        .collect::<Vec<_>>();
    ensure!(
        tools.len() == lane.tools.len(),
        "lane runtime unavailable: {:?}",
        plan.tool_errors.iter().filter(|(name, _)| lane.tools.contains(name)).collect::<Vec<_>>()
    );
    let mut inputs = lane_inputs(workspace, lane, tools, expected)?;
    for (key, value) in &plan.environment {
        inputs.environment.insert(key.clone(), value.clone());
    }
    inputs.environment.insert("PYTHONPYCACHEPREFIX".into(), "$LANE_ARTIFACTS/pycache".into());
    inputs.environment.insert("TMPDIR".into(), "$LANE_ARTIFACTS/scratch".into());
    if lane.id == "rust-nextest" {
        inputs
            .environment
            .insert("BIJUX_CI_FULL_WORKER_BUDGET".into(), lane.resource_units.to_string());
        for name in ["BIJUX_CI_FULL_SHARD_JOBS", "BIJUX_CI_FULL_SHARD_TEST_THREADS"] {
            inputs.environment.insert(name.into(), std::env::var(name).unwrap_or_default());
        }
    }
    Ok(inputs)
}

pub(super) fn execute_lane(
    workspace: &Workspace,
    id: &str,
    plan: &ExecutionPlan,
) -> Result<OpsCommandOutcome> {
    let lane = plan.lanes.iter().find(|lane| lane.id == id).context("worker lane absent")?;
    let owner = plan.output.join(id);
    std::fs::create_dir(&owner)?;
    let receipt_path = owner.join("receipt.json");
    let started = Instant::now();
    write_value(
        &receipt_path,
        &json!({"schema_version":"bijux.ci.local_test_lane_progress.v1","lane":id,"source":plan.source,"state":"running","all_passed":false}),
    )?;
    let result = match lane.harness.as_str() {
        "pytest" | "unittest" => python::python_lane(workspace, lane, plan, &owner, started),
        "nextest" => nextest::nextest_lane(workspace, lane, plan, &owner, started),
        "rustdoc" => rustdoc::doctest_lane(workspace, lane, plan, &owner, started),
        _ => Err(anyhow::anyhow!("unsupported local lane harness")),
    };
    match result {
        Ok(receipt) => {
            write_value(&receipt_path, &receipt)?;
            Ok(if admit_lane(&receipt, &plan.source, &receipt.inputs).is_ok() {
                OpsCommandOutcome::success(format!("{id} passed\n"))
            } else {
                OpsCommandOutcome::failure(format!("{id} failed; {}\n", receipt_path.display()))
            })
        }
        Err(error) => {
            write_value(
                &receipt_path,
                &json!({"schema_version":"bijux.ci.local_test_lane_error.v1","lane":id,"source":plan.source,"state":"error","all_passed":false,"native_evidence_root":owner,"execution_error":format!("{error:#}"),"elapsed_milliseconds":started.elapsed().as_millis()}),
            )?;
            Ok(OpsCommandOutcome::failure(format!("{id}: {error:#}\n")))
        }
    }
}

pub(super) fn terminal(
    lane: &LaneDefinition,
    plan: &ExecutionPlan,
    inputs: LaneInputs,
    outcomes: Vec<CaseOutcome>,
    artifacts: Vec<PathBuf>,
    errors: Vec<String>,
    exit_code: i32,
    started: Instant,
) -> Result<LaneReceipt> {
    let first_attempt =
        outcomes.iter().map(|case| &case.id).collect::<BTreeSet<_>>().len() == outcomes.len();
    Ok(LaneReceipt {
        schema_version: LANE_SCHEMA.into(),
        lane: lane.id.clone(),
        source: plan.source.clone(),
        state: if exit_code == 0 && errors.is_empty() { "passed" } else { "failed" }.into(),
        first_attempt,
        exit_code,
        expected: inputs.expected.clone(),
        inputs,
        outcomes,
        artifacts: artifacts.iter().map(|path| reference(path)).collect::<Result<Vec<_>>>()?,
        errors,
        elapsed_milliseconds: u64::try_from(started.elapsed().as_millis())?,
    })
}

pub(super) fn lane_environment(
    plan: &ExecutionPlan,
    owner: &Path,
) -> Result<Vec<(String, String)>> {
    let scratch = owner.join("scratch");
    bijux_dna_infra::ensure_dir(&scratch)?;
    let mut envs = plan.environment.clone();
    envs.extend([
        ("TMPDIR".into(), scratch.display().to_string()),
        ("TMP".into(), scratch.display().to_string()),
        ("TEMP".into(), scratch.display().to_string()),
        ("PYTHONPYCACHEPREFIX".into(), owner.join("pycache").display().to_string()),
        ("ARTIFACT_ROOT".into(), owner.display().to_string()),
        ("ISO_ROOT".into(), owner.display().to_string()),
    ]);
    Ok(envs)
}

pub(super) fn native_command(
    workspace: &Workspace,
    program: &str,
    args: &[String],
    envs: &[(String, String)],
    owner: &Path,
    label: &str,
) -> Result<std::process::Output> {
    let mut results = ProcessRunner::new(workspace).run_parallel_logged(
        vec![ParallelProcessSpec {
            label: label.into(),
            program: program.into(),
            args: args.to_vec(),
            envs: envs.to_vec(),
            current_dir: None,
        }],
        owner,
    )?;
    Ok(results.pop().context("native lane command completion missing")?.output)
}
