//! Lane process boundaries and the immutable complete-local execution plan.

use super::{
    admit_lane, ensure, json, lane_inputs, nextest, python, reference, rustdoc, write_value,
    AdmittedResources, ArtifactReference, BTreeSet, CaseOutcome, Context, Deserialize, Instant,
    LaneDefinition, LaneInputs, LaneReceipt, NativeExecutionPlan, OpsCommandOutcome,
    ParallelProcessSpec, Path, PathBuf, ProcessRunner, Result, Serialize, SourceIdentity,
    ToolIdentity, Workspace, LANE_SCHEMA,
};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LocalVerificationPlan {
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
    #[serde(default)]
    pub(super) native_execution: Option<NativeExecutionPlan>,
    #[serde(default)]
    pub(super) provisioned_resources: Option<AdmittedResources>,
    #[serde(default)]
    pub(super) resource_request: Option<ArtifactReference>,
}

pub(super) fn input_manifest(
    workspace: &Workspace,
    lane: &LaneDefinition,
    plan: &LocalVerificationPlan,
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
        inputs.environment.extend(native_environment(plan)?);
    }
    Ok(inputs)
}

/// Native controls come from the sealed coordinator plan, never worker ambient state.
pub(super) fn native_environment(plan: &LocalVerificationPlan) -> Result<Vec<(String, String)>> {
    let native =
        plan.native_execution.as_ref().context("sealed native execution allocation absent")?;
    native.validate(plan.worker_budget)?;
    let lane = plan
        .lanes
        .iter()
        .find(|lane| lane.id == "rust-nextest")
        .context("native lane absent from sealed plan")?;
    ensure!(
        lane.resource_units == native.worker_units,
        "native allocation differs from admitted lane weight"
    );
    Ok(native.environment())
}

pub(super) fn execute_lane(
    workspace: &Workspace,
    id: &str,
    plan: &LocalVerificationPlan,
) -> Result<OpsCommandOutcome> {
    native_environment(plan)?;
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
                &super::terminal_evidence::error_receipt(
                    lane,
                    plan,
                    &owner,
                    &error,
                    u64::try_from(started.elapsed().as_millis())?,
                ),
            )?;
            Ok(OpsCommandOutcome::failure(format!("{id}: {error:#}\n")))
        }
    }
}

pub(super) fn terminal(
    lane: &LaneDefinition,
    plan: &LocalVerificationPlan,
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

/// Bind worker CLI admission to its future lane owner without creating that owner.
pub(super) fn worker_environment(
    workspace: &Workspace,
    plan: &LocalVerificationPlan,
    lane_id: &str,
) -> Result<Vec<(String, String)>> {
    let owner = workspace.admit_artifact_path(&plan.output.join(lane_id))?;
    let mut environment = plan.environment.clone();
    environment.retain(|(key, _)| {
        !matches!(key.as_str(), "BIJUX_REPO_ROOT" | "ARTIFACT_ROOT" | "ISO_ROOT" | "ARTIFACT_DIR")
    });
    environment.push(("BIJUX_REPO_ROOT".into(), workspace.root.display().to_string()));
    for key in ["ARTIFACT_ROOT", "ISO_ROOT", "ARTIFACT_DIR"] {
        environment.push((key.into(), owner.display().to_string()));
    }
    Ok(environment)
}

pub(super) fn lane_environment(
    plan: &LocalVerificationPlan,
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

#[cfg(test)]
pub(super) mod tests {
    use super::*;

    pub(in super::super) fn plan(root: &Path) -> LocalVerificationPlan {
        LocalVerificationPlan {
            schema_version: "bijux.ci.local_test_execution.v1".into(),
            source: SourceIdentity {
                sha: "source".into(),
                tree: "tree".into(),
                patch_sha256: "patch".into(),
            },
            workspace: root.to_path_buf(),
            output: root.join("artifacts/output"),
            scratch: root.join("artifacts/products"),
            cache: root.join("artifacts/python"),
            lanes: vec![LaneDefinition {
                id: "rust-nextest".into(),
                harness: "nextest".into(),
                discovery_roots: vec![],
                explicit_modules: vec![],
                excluded_modules: vec![],
                input_files: vec![],
                input_trees: vec![],
                tools: vec![],
                resource_units: 4,
            }],
            tools: vec![],
            tool_errors: vec![],
            environment: vec![("CARGO_BUILD_JOBS".into(), "2".into())],
            worker_budget: 6,
            native_execution: Some(NativeExecutionPlan {
                worker_units: 4,
                process_jobs: 2,
                case_threads: 2,
                compiler_jobs: 2,
                configured_compiler_jobs: 8,
                partitions: 5,
            }),
            provisioned_resources: None,
            resource_request: None,
        }
    }

    #[test]
    fn native_controls_survive_plan_serialization_and_terminal_seal() -> Result<()> {
        let owner = tempfile::tempdir()?;
        let workspace = Workspace { root: owner.path().canonicalize()? };
        let mut admitted = plan(&workspace.root);
        let path = workspace.path("artifacts/execution-plan.json");
        write_value(&path, &admitted)?;
        let seal = reference(&path)?;
        let decoded: LocalVerificationPlan = serde_json::from_slice(&std::fs::read(&path)?)?;
        let environment = native_environment(&decoded)?;
        assert_eq!(
            environment,
            vec![
                ("BIJUX_CI_FULL_WORKER_BUDGET".into(), "4".into()),
                ("BIJUX_CI_FULL_SHARD_JOBS".into(), "2".into()),
                ("BIJUX_CI_FULL_SHARD_TEST_THREADS".into(), "2".into()),
                ("CARGO_BUILD_JOBS".into(), "2".into()),
            ]
        );
        crate::runtime::local_tests::verify_execution_plan(&workspace, &seal)?;
        admitted.native_execution.as_mut().context("native plan")?.compiler_jobs = 3;
        write_value(&path, &admitted)?;
        assert!(crate::runtime::local_tests::verify_execution_plan(&workspace, &seal).is_err());
        admitted.lanes[0].resource_units = 8;
        assert!(native_environment(&admitted).is_err());
        admitted.native_execution = None;
        assert!(native_environment(&admitted).is_err());
        Ok(())
    }

    #[test]
    fn default_native_compiler_is_independent_of_python_fixture_compiler() -> Result<()> {
        let mut admitted = plan(Path::new("/owned/source"));
        admitted.worker_budget = 9;
        admitted.lanes[0].resource_units = 8;
        admitted.native_execution =
            Some(super::super::NativeExecutionControls::default().admit(9, 8, 5)?);
        assert_eq!(admitted.environment, vec![("CARGO_BUILD_JOBS".into(), "2".into())]);
        assert_eq!(
            native_environment(&admitted)?
                .into_iter()
                .find(|(name, _)| name == "CARGO_BUILD_JOBS")
                .context("native compiler control")?
                .1,
            "8"
        );
        Ok(())
    }

    #[test]
    fn complete_workers_admit_only_their_isolated_lane_output_roots() -> Result<()> {
        let first = tempfile::tempdir()?;
        let second = tempfile::tempdir()?;
        let caller = tempfile::tempdir()?;
        let foreign = caller.path().join("artifacts/caller-output");
        let shared_target = caller.path().join("artifacts/shared-cargo-target");
        for root in [first.path(), second.path()] {
            let workspace = Workspace { root: root.canonicalize()? };
            let mut admitted = plan(&workspace.root);
            admitted.environment.extend([
                ("BIJUX_REPO_ROOT".into(), caller.path().display().to_string()),
                ("ARTIFACT_ROOT".into(), foreign.display().to_string()),
                ("ISO_ROOT".into(), foreign.display().to_string()),
                ("ARTIFACT_DIR".into(), foreign.display().to_string()),
                ("CARGO_TARGET_DIR".into(), shared_target.display().to_string()),
            ]);
            let owner = admitted.output.join("rust-nextest");
            assert!(!owner.exists(), "worker output belongs to execute_lane");
            let environment = worker_environment(&workspace, &admitted, "rust-nextest")?;
            for key in ["ARTIFACT_ROOT", "ISO_ROOT", "ARTIFACT_DIR", "BIJUX_REPO_ROOT"] {
                assert_eq!(
                    environment.iter().filter(|(name, _)| name == key).count(),
                    1,
                    "worker binding must replace inherited {key}"
                );
                let (_, value) = environment
                    .iter()
                    .find(|(name, _)| name == key)
                    .with_context(|| format!("worker binding missing: {key}"))?;
                if key == "BIJUX_REPO_ROOT" {
                    assert_eq!(Path::new(value), workspace.root);
                } else {
                    assert_eq!(
                        workspace.admit_artifact_path(Path::new(value))?,
                        owner,
                        "worker output must belong to its isolated lane"
                    );
                }
            }
            assert_eq!(
                environment.iter().find(|(name, _)| name == "CARGO_TARGET_DIR"),
                admitted.environment.iter().find(|(name, _)| name == "CARGO_TARGET_DIR")
            );
            assert_eq!(
                environment.iter().find(|(name, _)| name == "CARGO_BUILD_JOBS"),
                admitted.environment.iter().find(|(name, _)| name == "CARGO_BUILD_JOBS")
            );
            assert!(!owner.exists(), "launch binding must not materialize a lane owner");
            assert!(!owner.join("scratch").exists());
            assert!(!foreign.exists());
            assert!(!shared_target.exists());
        }
        Ok(())
    }
}
