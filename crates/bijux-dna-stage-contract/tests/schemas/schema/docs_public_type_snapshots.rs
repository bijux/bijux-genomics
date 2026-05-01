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
        "StageAdmissionRequestV1",
        "StageAdmissionOutcomeV1",
        "StageRefusalV1",
        "StagePlanV1",
        "StagePlanJsonV1",
        "PlannedArtifactV1",
        "StageArtifactPromiseV1",
        "StageProvenanceV1",
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

#[test]
fn contract_doc_points_to_executable_stage_contract_fixtures() {
    let root = crate::support::crate_root("bijux-dna-stage-contract")
        .unwrap_or_else(|err| panic!("resolve crate root: {err}"));
    let doc = root.join("docs").join("CONTRACT.md");
    let content =
        fs::read_to_string(&doc).unwrap_or_else(|err| panic!("read {}: {err}", doc.display()));

    for (fixture, stage_id, backend_tool_id) in [
        ("tests/fixtures/docs/fastq_trim_stage_contract.json", "fastq.trim_reads", "fastp"),
        ("tests/fixtures/docs/bam_align_stage_contract.json", "bam.align", "bwa"),
        ("tests/fixtures/docs/vcf_filter_stage_contract.json", "vcf.filter", "bcftools"),
    ] {
        assert!(
            content.contains(fixture),
            "CONTRACT.md must reference executable fixture {fixture}"
        );
        let path = root.join(fixture);
        let payload = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("read fixture {}: {err}", path.display()));
        let contract: bijux_dna_core::contract::CanonicalStageContractV1 =
            serde_json::from_str(&payload).unwrap_or_else(|err| {
                panic!("parse canonical stage contract {}: {err}", path.display())
            });
        assert_eq!(contract.stage_id.as_str(), stage_id);
        assert_eq!(contract.backend_tool_id.as_str(), backend_tool_id);
        assert_ne!(contract.stage_id.as_str(), contract.backend_tool_id.as_str());
    }
}
