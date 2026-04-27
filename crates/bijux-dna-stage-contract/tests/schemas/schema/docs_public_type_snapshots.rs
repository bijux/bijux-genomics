use std::fs;

#[test]
fn public_type_snapshots_have_doc_anchors() {
    let doc = crate::support::crate_root("bijux-dna-stage-contract")
        .unwrap_or_else(|err| panic!("resolve crate root: {err}"))
        .join("docs")
        .join("PUBLIC_API.md");
    let content =
        fs::read_to_string(&doc).unwrap_or_else(|err| panic!("read {}: {err}", doc.display()));

    for anchor in ["StagePlanV1", "ExecutionPlanV1", "StagePluginOutputV1"] {
        assert!(content.contains(anchor), "PUBLIC_API.md must include example for {anchor}");
    }
}

#[test]
fn public_api_doc_uses_current_root_type_names() {
    let doc = crate::support::crate_root("bijux-dna-stage-contract")
        .unwrap_or_else(|err| panic!("resolve crate root: {err}"))
        .join("docs")
        .join("PUBLIC_API.md");
    let content =
        fs::read_to_string(&doc).unwrap_or_else(|err| panic!("read {}: {err}", doc.display()));

    for current_type in [
        "ExecutionPlan",
        "PlanEdge",
        "PlanValidationContext",
        "RunExecutionPlan",
        "StagePlanV1",
        "StagePlanJsonV1",
        "PlannedArtifactV1",
        "PlannerContractV1",
        "StageInvocationV1",
        "StagePluginOutputV1",
        "StagePlugin",
        "Executor",
        "DryRunExecutor",
        "PlanDecisionReason",
        "PlanReasonKind",
        "StageExecutorEntry",
        "ReadinessBadge",
        "StageDomain",
        "ArtifactRef",
        "StageIO",
    ] {
        assert!(
            content.contains(&format!("`{current_type}`")),
            "PUBLIC_API.md must document root type `{current_type}`"
        );
    }

    for stale_type in ["`StagePlan`", "`StageSpecRef`", "`PluginSpec`"] {
        assert!(!content.contains(stale_type), "PUBLIC_API.md must not list stale {stale_type}");
    }
}
