use super::{
    ensure, input_manifest, lane_environment, native_command, nextest_membership, terminal,
    verify_inputs, Context, Instant, LaneDefinition, LaneReceipt, LocalVerificationPlan, Path,
    Result, Value, Workspace,
};

pub(super) fn nextest_lane(
    workspace: &Workspace,
    lane: &LaneDefinition,
    plan: &LocalVerificationPlan,
    owner: &Path,
    started: Instant,
) -> Result<LaneReceipt> {
    let mut environment = lane_environment(plan, owner)?;
    let native = super::execution::native_environment(plan)?;
    environment.retain(|(key, _)| !native.iter().any(|(name, _)| name == key));
    environment.extend(native);
    environment
        .push(("BIJUX_CI_FULL_SHARD_SCRATCH_ROOT".into(), plan.scratch.display().to_string()));
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
    let allocation_error = verify_native_allocation(
        &full,
        plan.native_execution.as_ref().context("native plan absent")?,
    )
    .err();
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
    if let Some(error) = allocation_error {
        evidence.errors.push(format!("{error:#}"));
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

fn verify_native_allocation(full: &Value, plan: &super::NativeExecutionPlan) -> Result<()> {
    let requested = plan
        .process_jobs
        .checked_mul(plan.case_threads)
        .context("native requested case-worker product overflow")?;
    ensure!(
        full["execution_resources"]["worker_budget"] == plan.worker_units
            && full["execution_resources"]["process_jobs"] == plan.process_jobs
            && full["execution_resources"]["test_threads_per_process"] == plan.case_threads
            && full["execution_resources"]["requested_case_workers"] == requested
            && full["execution_resources"]["maximum_case_workers"]
                == requested.min(plan.worker_units)
            && full["execution_resources"]["selected_shards"] == plan.partitions
            && full["archive_build_jobs"] == plan.compiler_jobs
            && full["compiler_admission"]["configured_jobs"] == plan.configured_compiler_jobs
            && full["compiler_admission"]["effective_jobs"] == plan.compiler_jobs
            && full["compiler_admission"]["state"] == "admitted",
        "Nextest terminal resource allocation differs from sealed coordinator plan"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_native_resource_mismatch_cannot_inherit_green_acceptance() -> Result<()> {
        let plan = super::super::NativeExecutionControls::default().admit(9, 8, 5)?;
        let full = serde_json::json!({"all_passed":true,
            "execution_resources":{"worker_budget":8,"process_jobs":4,
                "test_threads_per_process":2,"selected_shards":5,
                "requested_case_workers":8,"maximum_case_workers":8},
            "archive_build_jobs":8,"compiler_admission":{"configured_jobs":8,
                "effective_jobs":8,"state":"admitted"}});
        verify_native_allocation(&full, &plan)?;
        for (object, field) in [
            ("execution_resources", "worker_budget"),
            ("execution_resources", "process_jobs"),
            ("execution_resources", "test_threads_per_process"),
            ("execution_resources", "selected_shards"),
            ("execution_resources", "requested_case_workers"),
            ("execution_resources", "maximum_case_workers"),
            ("compiler_admission", "configured_jobs"),
            ("compiler_admission", "effective_jobs"),
            ("compiler_admission", "state"),
        ] {
            let mut changed = full.clone();
            changed[object][field] = Value::Null;
            assert!(verify_native_allocation(&changed, &plan).is_err(), "{object}/{field}");
        }
        let mut changed = full;
        changed["archive_build_jobs"] = serde_json::json!(2);
        assert!(verify_native_allocation(&changed, &plan).is_err());
        let adaptive = super::super::NativeExecutionControls::read(|name| {
            Ok(match name {
                "BIJUX_CI_FULL_WORKER_BUDGET" => Some("4".into()),
                "BIJUX_CI_FULL_SHARD_JOBS" => Some("3".into()),
                "BIJUX_CI_FULL_SHARD_TEST_THREADS" | "CARGO_BUILD_JOBS" => Some("2".into()),
                _ => None,
            })
        })?
        .admit(6, 8, 5)?;
        let mut adaptive_full = serde_json::json!({"execution_resources":{
            "worker_budget":4,"process_jobs":3,"test_threads_per_process":2,"selected_shards":5,
            "requested_case_workers":6,"maximum_case_workers":4},
            "archive_build_jobs":2,"compiler_admission":{"configured_jobs":8,"effective_jobs":2,"state":"admitted"}});
        verify_native_allocation(&adaptive_full, &adaptive)?;
        adaptive_full["execution_resources"]["maximum_case_workers"] = serde_json::json!(6);
        assert!(
            verify_native_allocation(&adaptive_full, &adaptive).is_err(),
            "nominal product replaced weighted ceiling"
        );
        Ok(())
    }
}
