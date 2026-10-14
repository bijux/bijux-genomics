use super::{
    admit_lane, admit_python_cases, ensure, input_manifest, json, lane_environment, native_command,
    reference, selections, terminal, tool_path, verify_artifacts, verify_inputs,
    verify_source_reads, write_value, CaseOutcome, Context, ExecutionPlan, Instant, LaneDefinition,
    LaneInputs, LaneReceipt, Path, Result, Value, Workspace,
};

pub(super) fn python_lane(
    workspace: &Workspace,
    lane: &LaneDefinition,
    plan: &ExecutionPlan,
    owner: &Path,
    started: Instant,
) -> Result<LaneReceipt> {
    let mut inputs = input_manifest(workspace, lane, plan, vec![])?;
    verify_inputs(&workspace.root, &inputs)?;
    let selected = selections(workspace, lane)?;
    ensure!(!selected.is_empty(), "Python lane has no source selections");
    let discovered = owner.join("inventory.json");
    let cases = owner.join("cases.json");
    let mut args = vec![
        workspace.path("makes/bin/python_test_receipt.py").display().to_string(),
        "--source-root".into(),
        workspace.root.display().to_string(),
        "--output".into(),
        discovered.display().to_string(),
        "--scratch".into(),
        owner.join("scratch").display().to_string(),
        "--harness".into(),
        lane.harness.clone(),
    ];
    args.extend(selected);
    let environment = lane_environment(plan, owner)?;
    let python = tool_path(&plan.tools, "python")?;
    let mut discover_args = args.clone();
    discover_args.insert(1, "--discover".into());
    let discovery =
        native_command(workspace, python, &discover_args, &environment, owner, "discovery")?;
    ensure!(discovery.status.success(), "native Python discovery failed");
    let discovered_facts: Value = serde_json::from_slice(&std::fs::read(&discovered)?)?;
    let expected: Vec<String> = serde_json::from_value(discovered_facts["expected"].clone())?;
    inputs.expected = expected;
    verify_source_reads(&workspace.root, owner, &inputs, &discovered_facts)?;
    verify_inputs(&workspace.root, &inputs)?;
    inputs.closure_qualified = true;
    let cached = plan.cache.join(format!("{}.json", lane.id));
    if cached.is_file() {
        let evidence = serde_json::from_slice::<LaneReceipt>(&std::fs::read(&cached)?);
        if let Ok(evidence) = evidence {
            if admit_lane(&evidence, &plan.source, &inputs).is_ok()
                && verify_artifacts(&evidence).is_ok()
                && validate_native_cases(plan, &evidence, &inputs).is_ok()
            {
                write_value(
                    &owner.join("equivalence.json"),
                    &json!({"schema_version":"bijux.ci.local_test_equivalence.v1","original_receipt":reference(&cached)?,"origin_source":evidence.source,"candidate_source":plan.source,"input_manifest_equal":true,"discovery":reference(&discovered)?}),
                )?;
                return Ok(evidence);
            }
        }
    }
    args[4] = cases.display().to_string();
    args.insert(1, discovered.display().to_string());
    args.insert(1, "--expected".into());
    let execution = native_command(workspace, python, &args, &environment, owner, "execution")?;
    let facts: Value = serde_json::from_slice(&std::fs::read(&cases)?)?;
    verify_source_reads(&workspace.root, owner, &inputs, &facts)?;
    verify_inputs(&workspace.root, &inputs)?;
    let outcomes: Vec<CaseOutcome> = serde_json::from_value(facts["outcomes"].clone())?;
    let errors = facts["collection_errors"]
        .as_array()
        .context("native Python collection errors absent")?
        .iter()
        .map(Value::to_string)
        .collect();
    let evidence = terminal(
        lane,
        plan,
        inputs,
        outcomes,
        vec![
            discovered,
            cases,
            owner.join("execution.stdout.log"),
            owner.join("execution.stderr.log"),
        ],
        errors,
        execution.status.code().unwrap_or(1),
        started,
    )?;
    if admit_lane(&evidence, &plan.source, &evidence.inputs).is_ok() {
        write_value(&cached, &evidence)?;
    }
    Ok(evidence)
}

pub(super) fn validate_native_cases(
    plan: &ExecutionPlan,
    receipt: &LaneReceipt,
    inputs: &LaneInputs,
) -> Result<()> {
    let artifact = receipt
        .artifacts
        .iter()
        .find(|artifact| {
            Path::new(&artifact.path).file_name().is_some_and(|name| name == "cases.json")
        })
        .context("cached native Python case evidence absent")?;
    let path = Path::new(&artifact.path);
    let prior_owner = path.parent().context("cached Python owner absent")?;
    let run = prior_owner.parent().context("cached run owner absent")?;
    ensure!(
        run.parent() == plan.output.parent()
            && prior_owner.file_name().is_some_and(|name| name == receipt.lane.as_str()),
        "cached Python evidence belongs to an unowned run"
    );
    let facts: Value = serde_json::from_slice(&std::fs::read(path)?)?;
    let outcomes = admit_python_cases(inputs, &facts)?;
    ensure!(
        serde_json::to_value(outcomes)? == serde_json::to_value(&receipt.outcomes)?,
        "cached lane outcomes differ from native case evidence"
    );
    let original_root =
        Path::new(facts["source_root"].as_str().context("cached case source binding absent")?);
    ensure!(original_root == plan.workspace, "cached native source ownership changed");
    verify_source_reads(original_root, prior_owner, inputs, &facts)?;
    Ok(())
}
