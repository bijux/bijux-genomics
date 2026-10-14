use super::{
    ensure, input_manifest, lane_environment, native_command, nextest_membership, terminal,
    verify_inputs, Context, ExecutionPlan, Instant, LaneDefinition, LaneReceipt, Path, Result,
    Value, Workspace,
};

pub(super) fn nextest_lane(
    workspace: &Workspace,
    lane: &LaneDefinition,
    plan: &ExecutionPlan,
    owner: &Path,
    started: Instant,
) -> Result<LaneReceipt> {
    let mut environment = lane_environment(plan, owner)?;
    environment.extend([
        ("BIJUX_CI_FULL_SHARD_SCRATCH_ROOT".into(), plan.scratch.display().to_string()),
        ("BIJUX_CI_FULL_WORKER_BUDGET".into(), lane.resource_units.to_string()),
    ]);
    let mut inputs = input_manifest(workspace, lane, plan, vec![])?;
    verify_inputs(&workspace.root, &inputs)?;
    let output = native_command(
        workspace,
        std::env::current_exe()?.to_str().context("developer executable path")?,
        &["tooling".into(), "run".into(), "ci-test-all-sharded".into(), "--local-products".into()],
        &environment,
        owner,
        "execution",
    )?;
    let full_path = owner.join("ci/full-gate-timing.json");
    let full: Value = serde_json::from_slice(&std::fs::read(&full_path)?)?;
    ensure!(
        full["source_commit"] == plan.source.sha
            && full["source_patch_sha256"] == plan.source.patch_sha256,
        "Nextest receipt source differs from admitted source"
    );
    let contract: Value =
        serde_json::from_slice(&std::fs::read(workspace.path("configs/ci/full-shards.json"))?)?;
    let report_root = owner.join(
        Path::new(contract["report_root"].as_str().context("Nextest report root absent")?)
            .strip_prefix("artifacts")?,
    );
    let inventory = report_root.join("test-inventory.stdout.log");
    let inventory_receipt: Value =
        serde_json::from_slice(&std::fs::read(report_root.join("test-inventory.receipt.json"))?)?;
    ensure!(
        inventory_receipt["source_commit"] == plan.source.sha
            && inventory_receipt["source_patch_sha256"] == plan.source.patch_sha256
            && bijux_dna_infra::hash_file_sha256(&inventory)?
                == inventory_receipt["sha256"]
                    .as_str()
                    .context("Nextest inventory digest absent")?,
        "Nextest inventory bytes or source changed"
    );
    let mut streams = Vec::new();
    let mut artifacts = vec![
        full_path,
        inventory.clone(),
        owner.join("execution.stdout.log"),
        owner.join("execution.stderr.log"),
    ];
    for shard in contract["shards"].as_array().context("Nextest declared shards absent")? {
        let id = shard["id"].as_str().context("Nextest shard id absent")?.to_string();
        let partition =
            shard["partition"].as_str().context("Nextest partition absent")?.to_string();
        let path = report_root.join(format!("{id}.stdout.log"));
        let bytes = if path.is_file() {
            artifacts.push(path.clone());
            std::fs::read(path)?
        } else {
            Vec::new()
        };
        streams.push((id, partition, bytes));
    }
    let mut evidence = nextest_membership(&std::fs::read(inventory)?, &streams)?;
    if full["all_passed"] != true
        || full["execution_scope"]["complete_contract_coverage"] != true
        || !output.status.success()
    {
        evidence.errors.push("Nextest terminal acceptance failed".into());
    }
    inputs.expected = evidence.expected;
    verify_inputs(&workspace.root, &inputs)?;
    terminal(
        lane,
        plan,
        inputs,
        evidence.outcomes,
        artifacts,
        evidence.errors,
        output.status.code().unwrap_or(1),
        started,
    )
}
