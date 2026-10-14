use super::{
    doctest_inventory, doctest_outcomes, doctest_targets, ensure, input_manifest, json,
    lane_environment, native_command, source_identity, terminal, tool_path, verify_inputs,
    verify_producer_manifest, write_value, CompilationCache, Context, Instant, LaneDefinition,
    LaneReceipt, LocalVerificationPlan, Path, Result, TrackedSourceSnapshot, Value, Workspace,
};

pub(super) fn doctest_lane(
    workspace: &Workspace,
    lane: &LaneDefinition,
    plan: &LocalVerificationPlan,
    owner: &Path,
    started: Instant,
) -> Result<LaneReceipt> {
    let mut inputs = input_manifest(workspace, lane, plan, vec![])?;
    verify_inputs(&workspace.root, &inputs)?;
    let shared_workspace = plan.scratch.join("compile-cache/workspace");
    let full_path = plan.output.join("rust-nextest/ci/full-gate-timing.json");
    let producer: Option<Value> =
        std::fs::read(&full_path).ok().and_then(|bytes| serde_json::from_slice(&bytes).ok());
    let qualification = (|| -> Result<()> {
        let producer = producer.as_ref().context("Nextest producer receipt unavailable")?;
        ensure!(
            producer["source_commit"] == plan.source.sha
                && producer["source_patch_sha256"] == plan.source.patch_sha256
                && producer["compiled_source_verification"]["unchanged"] == true
                && producer["evidence_retention"]["compilation_workspace"]
                    == json!(shared_workspace),
            "Nextest producer source unqualified"
        );
        let descriptor = if producer["product_transport"] == "local_metadata" {
            &producer["compiled_products"]["input_manifest"]
        } else {
            &producer["archive"]["producer_input_manifest"]
        };
        verify_producer_manifest(descriptor)?;
        ensure!(
            source_identity(&Workspace { root: shared_workspace.clone() })? == plan.source,
            "retained producer source changed"
        );
        Ok(())
    })();
    // One source/product-qualified writer reuses the retained producer. If producer admission
    // failed, the independent doctest lane builds in its own admitted owner and records cost.
    let reused = qualification.is_ok();
    let scratch = if reused { plan.scratch.clone() } else { owner.join("rust-products") };
    let cache = CompilationCache::acquire(&scratch)?;
    let compiler_root = scratch.join("compile-cache/workspace");
    TrackedSourceSnapshot::capture(workspace)?.materialize_cached_clean(&compiler_root)?;
    let compiler = Workspace { root: compiler_root };
    let mut environment = lane_environment(plan, owner)?;
    environment.extend([
        ("CARGO_TARGET_DIR".into(), cache.target().display().to_string()),
        ("CARGO_BUILD_JOBS".into(), lane.resource_units.to_string()),
        ("BIJUX_REPO_ROOT".into(), compiler.root.display().to_string()),
    ]);
    write_value(
        &owner.join("producer-admission.json"),
        &json!({"schema_version":"bijux.ci.doctest_producer.v1","reuse_admitted":reused,"assertion_outcome_is_separate":true,"source":plan.source,"compilation_workspace":compiler.root,"compilation_target":cache.target(),"fallback_reason":qualification.err().map(|error|format!("{error:#}"))}),
    )?;
    let cargo = tool_path(&plan.tools, "cargo")?;
    let metadata = native_command(
        &compiler,
        cargo,
        &[
            "metadata".into(),
            "--locked".into(),
            "--no-deps".into(),
            "--format-version".into(),
            "1".into(),
        ],
        &environment,
        owner,
        "metadata",
    )?;
    ensure!(metadata.status.success(), "native Cargo doctest target discovery failed");
    let declared = doctest_targets(&serde_json::from_slice(&metadata.stdout)?)?;
    let mut args = vec![
        "test".into(),
        "--locked".into(),
        "--workspace".into(),
        "--all-features".into(),
        "--doc".into(),
        "--".into(),
    ];
    let mut listing = args.clone();
    listing.push("--list".into());
    let listed = native_command(&compiler, cargo, &listing, &environment, owner, "discovery")?;
    ensure!(listed.status.success(), "native Rustdoc example discovery failed");
    let expected = doctest_inventory(
        &declared,
        std::str::from_utf8(&listed.stdout)?,
        std::str::from_utf8(&listed.stderr)?,
    )?;
    inputs.expected = expected.clone();
    args.push(format!("--test-threads={}", lane.resource_units));
    let executed = native_command(&compiler, cargo, &args, &environment, owner, "execution")?;
    let outcomes = doctest_outcomes(
        &declared,
        &expected,
        std::str::from_utf8(&executed.stdout)?,
        std::str::from_utf8(&executed.stderr)?,
    )?;
    verify_inputs(&workspace.root, &inputs)?;
    ensure!(source_identity(&compiler)? == plan.source, "compiled doctest source changed");
    terminal(
        lane,
        plan,
        inputs,
        outcomes,
        vec![
            owner.join("producer-admission.json"),
            owner.join("metadata.stdout.log"),
            owner.join("discovery.stdout.log"),
            owner.join("discovery.stderr.log"),
            owner.join("execution.stdout.log"),
            owner.join("execution.stderr.log"),
        ],
        vec![],
        executed.status.code().unwrap_or(1),
        started,
    )
}
