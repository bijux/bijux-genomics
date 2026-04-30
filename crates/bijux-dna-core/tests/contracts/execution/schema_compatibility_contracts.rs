use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_dna_core::contract::{
    governed_api_route_adapters, governed_schema_registry, migrate_plan_manifest_value,
    migrate_workflow_manifest_value, ArtifactRole, CompressionSupport, PlanEnvironmentContractV1,
    PlanManifestStepV1, PlanPolicy, ReadLayoutMode, ToolConstraints, WorkflowInputArtifactV1,
    WorkflowManifestV1,
};

#[test]
fn schema_registry_covers_governed_iteration_contracts() {
    let registry = governed_schema_registry();
    let ids = registry
        .iter()
        .map(|entry| (entry.schema_family.as_str(), entry.schema_version.as_str()))
        .collect::<BTreeMap<_, _>>();

    assert_eq!(ids.get("workflow_manifest"), Some(&"bijux.workflow_manifest.v1"));
    assert_eq!(ids.get("plan_manifest"), Some(&"bijux.plan_manifest.v1"));
    assert_eq!(ids.get("artifact_inventory"), Some(&"bijux.artifact_inventory.v1"));
    assert_eq!(ids.get("evidence_bundle"), Some(&"bijux.evidence_bundle.v1"));
    assert_eq!(ids.get("metrics_envelope"), Some(&"bijux.metrics_envelope.v2"));
    assert_eq!(ids.get("report"), Some(&"bijux.report.v1"));
    assert_eq!(ids.get("run_state"), Some(&"bijux.run_state.v1"));
    assert_eq!(ids.get("run_failure"), Some(&"bijux.run_failure.v1"));

    assert!(
        registry
            .iter()
            .all(|entry| !entry.semantic_version.trim().is_empty() && !entry.notes.trim().is_empty())
    );
}

#[test]
fn workflow_manifest_v0_upgrades_deterministically() -> anyhow::Result<()> {
    let legacy = serde_json::json!({
        "schema_version": "bijux.workflow_manifest.v0",
        "domain": "fastq",
        "profile_id": "essential_qc",
        "inputs": [
            {
                "artifact_id": "reads",
                "role": "reads",
                "path": "inputs/sample.fastq.gz",
                "layout": "single_end",
                "compression": "gzip"
            }
        ],
        "sample_metadata": {
            "sample_id": "s1"
        },
        "requested_stages": [
            {
                "stage_id": "fastq.validate_reads",
                "advisory_only": false
            }
        ]
    });

    let (first_manifest, first_audit) = migrate_workflow_manifest_value(&legacy)?;
    let (second_manifest, second_audit) = migrate_workflow_manifest_value(&legacy)?;

    assert_eq!(first_manifest, second_manifest);
    assert_eq!(first_audit, second_audit);
    assert_eq!(first_manifest.schema_version, "bijux.workflow_manifest.v1");
    assert!(first_manifest.evidence_expectations.is_empty());
    assert_eq!(
        first_audit.exact_reason,
        "workflow manifest upgraded from governed legacy v0 by filling explicit execution and evidence defaults"
    );
    Ok(())
}

#[test]
fn plan_manifest_v0_upgrade_preserves_equivalent_v1_fingerprint() -> anyhow::Result<()> {
    let current = current_plan_manifest()?;
    let legacy = serde_json::json!({
        "schema_version": "bijux.plan_manifest.v0",
        "domain": current.domain,
        "profile_id": current.profile_id,
        "pipeline_id": current.pipeline_id,
        "planner_version": current.planner_version,
        "policy": current.policy,
        "workflow_fingerprint": current.workflow_fingerprint,
        "graph_hash": current.graph_hash,
        "plan_fingerprint": current.plan_fingerprint,
        "ordered_steps": current.ordered_steps
    });

    let (migrated, audit) = migrate_plan_manifest_value(&legacy)?;
    assert_eq!(migrated.plan_fingerprint, current.plan_fingerprint);
    assert_eq!(migrated.schema_version, "bijux.plan_manifest.v1");
    assert_eq!(audit.status, bijux_dna_core::contract::ManifestMigrationStatusV1::Upgraded);
    assert_eq!(
        audit.exact_reason,
        "plan manifest upgraded from governed legacy v0 by materializing empty review surfaces before recomputing the plan fingerprint"
    );
    Ok(())
}

#[test]
fn unsupported_manifest_versions_are_refused_with_exact_reason() {
    let unsupported = serde_json::json!({
        "schema_version": "bijux.workflow_manifest.v9",
        "domain": "fastq",
        "profile_id": "essential_qc"
    });
    let err = migrate_workflow_manifest_value(&unsupported).unwrap_err();
    assert_eq!(
        err.to_string(),
        "validation error: workflow_manifest schema_version bijux.workflow_manifest.v9 is unsupported; supported versions: bijux.workflow_manifest.v0, bijux.workflow_manifest.v1"
    );
}

#[test]
fn api_route_adapters_link_v1_routes_to_governed_schema_families() {
    let adapters = governed_api_route_adapters();
    assert!(adapters.iter().any(|entry| {
        entry.route_id == "v1.plan"
            && entry.writes_schema_families.contains(&"plan_manifest".to_string())
            && entry.writes_schema_families.contains(&"workflow_manifest".to_string())
    }));
    assert!(adapters.iter().any(|entry| {
        entry.route_id == "v1.execute"
            && entry.writes_schema_families.contains(&"evidence_bundle".to_string())
            && entry.writes_schema_families.contains(&"run_state".to_string())
    }));
}

fn current_plan_manifest() -> anyhow::Result<bijux_dna_core::contract::PlanManifestV1> {
    let workflow = WorkflowManifestV1 {
        schema_version: "bijux.workflow_manifest.v1".to_string(),
        domain: "fastq".to_string(),
        profile_id: "essential_qc".to_string(),
        inputs: vec![WorkflowInputArtifactV1 {
            artifact_id: "reads".to_string(),
            role: ArtifactRole::Reads,
            path: PathBuf::from("/tmp/runtime-root/inputs/sample.fastq.gz"),
            layout: Some(ReadLayoutMode::SingleEnd),
            compression: Some(CompressionSupport::Gzip),
            format_id: Some("fastq".to_string()),
        }],
        sample_metadata: BTreeMap::from([("sample_id".to_string(), "s1".to_string())]),
        reference_assets: Vec::new(),
        requested_stages: Vec::new(),
        policies: bijux_dna_core::contract::WorkflowPolicySurfaceV1::default(),
        executor_preferences: bijux_dna_core::contract::WorkflowExecutorPreferencesV1::default(),
        evidence_expectations: Vec::new(),
        labels: BTreeMap::new(),
        notes: None,
    };
    let workflow_fingerprint = workflow.fingerprint()?;
    let mut manifest = bijux_dna_core::contract::PlanManifestV1 {
        schema_version: "bijux.plan_manifest.v1".to_string(),
        domain: "fastq".to_string(),
        profile_id: "essential_qc".to_string(),
        pipeline_id: "fastq-to-fastq__default__v1".to_string(),
        planner_version: "planner.test".to_string(),
        policy: PlanPolicy::PreferAccuracy,
        workflow_fingerprint,
        graph_hash: "sha256:graph".to_string(),
        plan_fingerprint: String::new(),
        ordered_steps: vec![PlanManifestStepV1 {
            step_id: "fastq.validate_reads".to_string(),
            stage_id: "fastq.validate_reads".to_string(),
            dependencies: Vec::new(),
            stage_contract_ref: Some("fastq.validate_reads@v1".to_string()),
            effective_parameters_json: serde_json::json!({
                "input_path": "/tmp/runtime-root/inputs/sample.fastq.gz"
            }),
            environment: PlanEnvironmentContractV1 {
                image: "ghcr.io/bijux/test:1".to_string(),
                image_digest: Some("sha256:deadbeef".to_string()),
                command: vec![
                    "bijux-tool".to_string(),
                    "--reads".to_string(),
                    "/tmp/runtime-root/inputs/sample.fastq.gz".to_string(),
                ],
                resources: ToolConstraints::default(),
                out_dir: PathBuf::from("/tmp/runtime-root/stages/validate"),
            },
            artifact_promises: Vec::new(),
            reference_asset_ids: Vec::new(),
            cache_key: "cache:reads".to_string(),
            advisory: false,
        }],
        stage_decisions: Vec::new(),
        refusal_records: Vec::new(),
        warning_records: Vec::new(),
        parameter_traces: Vec::new(),
        cross_domain_handoffs: Vec::new(),
    };
    manifest.refresh_fingerprint()?;
    Ok(manifest)
}
