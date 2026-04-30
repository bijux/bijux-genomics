use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_dna_core::contract::{
    build_plan_manifest, diff_plan_manifests, validate_cross_domain_handoffs, ArtifactRef,
    ArtifactRole, ExecutionEdge, ExecutionGraph, ExecutionStep, ParameterResolutionTraceV1,
    PlanManifestBuildInputV1, PlanPolicy, PlannerParameterSourceV1, PlannerRefusalCodeV1,
    PlannerRefusalRecordV1, PlannerWarningCodeV1, PlannerWarningRecordV1, StageIO,
    ToolConstraints, WorkflowInputArtifactV1, WorkflowManifestV1, WorkflowReferenceAssetV1,
    WorkflowStageRequestV1,
};
use bijux_dna_core::prelude::{ArtifactId, CommandSpecV1, ContainerImageRefV1, StageId, StepId};

fn mk_step(
    step_id: &'static str,
    stage_id: &'static str,
    input_role: ArtifactRole,
    output_role: ArtifactRole,
) -> ExecutionStep {
    ExecutionStep {
        step_id: StepId::from_static(step_id),
        stage_id: StageId::from_static(stage_id),
        image: ContainerImageRefV1 {
            image: format!("bijux/{stage_id}"),
            digest: Some(format!("sha256:{step_id}")),
        },
        command: CommandSpecV1 { template: vec![stage_id.to_string(), "--run".to_string()] },
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 2,
            tmp_gb: 1,
            threads: 1,
        },
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::new(format!("{step_id}.input")),
                format!("{step_id}.in").into(),
                input_role,
            )],
            outputs: vec![ArtifactRef::required(
                ArtifactId::new(format!("{step_id}.output")),
                format!("{step_id}.out").into(),
                output_role,
            )],
        },
        out_dir: format!("out/{step_id}").into(),
        aux_images: BTreeMap::new(),
        expected_artifact_ids: Vec::new(),
        metrics_schema_ids: Vec::new(),
    }
}

fn fastq_workflow_manifest() -> WorkflowManifestV1 {
    let mut manifest = WorkflowManifestV1::new("fastq", "fastq-to-fastq__default__v1");
    manifest.inputs = vec![WorkflowInputArtifactV1 {
        artifact_id: "reads".to_string(),
        role: ArtifactRole::Reads,
        path: "reads.fastq.gz".into(),
        layout: None,
        compression: None,
        format_id: Some("fastq.gz".to_string()),
    }];
    manifest.reference_assets = vec![WorkflowReferenceAssetV1 {
        asset_id: "reference".to_string(),
        role: ArtifactRole::Reference,
        path: "reference.fa".into(),
        checksum_sha256: Some("a".repeat(64)),
        build_id: Some("GRCh38".to_string()),
        alias_group: None,
    }];
    manifest.requested_stages = vec![
        WorkflowStageRequestV1 {
            stage_id: "fastq.validate_reads".to_string(),
            advisory_only: false,
        },
        WorkflowStageRequestV1 {
            stage_id: "fastq.trim_reads".to_string(),
            advisory_only: true,
        },
    ];
    manifest.notes = Some("operator note".to_string());
    manifest.labels.insert("ticket".to_string(), "ABC-123".to_string());
    manifest
}

#[test]
fn workflow_manifest_fingerprint_ignores_authoring_noise() -> anyhow::Result<()> {
    let manifest = fastq_workflow_manifest();
    let mut reordered = manifest.clone();
    reordered.requested_stages.reverse();
    reordered.inputs.reverse();
    reordered.reference_assets.reverse();
    reordered.notes = Some("updated note".to_string());
    reordered.labels.insert("owner".to_string(), "science".to_string());

    assert_eq!(manifest.fingerprint()?, reordered.fingerprint()?);
    Ok(())
}

#[test]
fn build_plan_manifest_is_deterministic_for_shuffled_graph_inputs() -> anyhow::Result<()> {
    let step_a =
        mk_step("fastq.validate_reads", "fastq.validate_reads", ArtifactRole::Reads, ArtifactRole::Reads);
    let step_b =
        mk_step("fastq.trim_reads", "fastq.trim_reads", ArtifactRole::Reads, ArtifactRole::TrimmedReads);
    let edges = vec![ExecutionEdge::new(
        StepId::from_static("fastq.validate_reads"),
        StepId::from_static("fastq.trim_reads"),
    )];

    let graph_a = ExecutionGraph::new(
        "fastq-to-fastq__default__v1",
        "planner-fastq",
        PlanPolicy::PreferAccuracy,
        vec![step_a.clone(), step_b.clone()],
        edges.clone(),
    )?;
    let graph_b = ExecutionGraph::new(
        "fastq-to-fastq__default__v1",
        "planner-fastq",
        PlanPolicy::PreferAccuracy,
        vec![step_b, step_a],
        edges,
    )?;

    let workflow = fastq_workflow_manifest();
    let manifest_a = build_plan_manifest(PlanManifestBuildInputV1 {
        workflow_manifest: workflow.clone(),
        graph: graph_a,
        stage_contract_refs: vec![(
            "fastq.trim_reads".to_string(),
            "sha256:trim-contract".to_string(),
        )],
        effective_parameters_by_step: BTreeMap::from([(
            "fastq.trim_reads".to_string(),
            serde_json::json!({ "min_length": 25, "adapter_mode": "auto" }),
        )]),
        parameter_traces: vec![ParameterResolutionTraceV1 {
            step_id: "fastq.trim_reads".to_string(),
            stage_id: "fastq.trim_reads".to_string(),
            parameter: "min_length".to_string(),
            source: PlannerParameterSourceV1::DomainDefault,
            resolved_value: serde_json::json!(25),
            detail: "default FASTQ trim contract".to_string(),
        }],
        refusal_records: Vec::new(),
        warning_records: vec![PlannerWarningRecordV1 {
            code: PlannerWarningCodeV1::AdvisoryStage,
            stage_id: Some("fastq.trim_reads".to_string()),
            message: "trim stage is advisory in this fixture".to_string(),
        }],
    })?;
    let manifest_b = build_plan_manifest(PlanManifestBuildInputV1 {
        workflow_manifest: workflow,
        graph: graph_b,
        stage_contract_refs: vec![(
            "fastq.trim_reads".to_string(),
            "sha256:trim-contract".to_string(),
        )],
        effective_parameters_by_step: BTreeMap::from([(
            "fastq.trim_reads".to_string(),
            serde_json::json!({ "adapter_mode": "auto", "min_length": 25 }),
        )]),
        parameter_traces: vec![ParameterResolutionTraceV1 {
            step_id: "fastq.trim_reads".to_string(),
            stage_id: "fastq.trim_reads".to_string(),
            parameter: "min_length".to_string(),
            source: PlannerParameterSourceV1::DomainDefault,
            resolved_value: serde_json::json!(25),
            detail: "default FASTQ trim contract".to_string(),
        }],
        refusal_records: Vec::new(),
        warning_records: vec![PlannerWarningRecordV1 {
            code: PlannerWarningCodeV1::AdvisoryStage,
            stage_id: Some("fastq.trim_reads".to_string()),
            message: "trim stage is advisory in this fixture".to_string(),
        }],
    })?;

    assert_eq!(manifest_a.plan_fingerprint, manifest_b.plan_fingerprint);
    assert_eq!(manifest_a.ordered_steps[0].step_id, "fastq.validate_reads");
    assert_eq!(manifest_a.ordered_steps[1].step_id, "fastq.trim_reads");
    assert!(manifest_a.ordered_steps[1].advisory);
    Ok(())
}

#[test]
fn build_plan_manifest_ignores_temp_root_path_noise() -> anyhow::Result<()> {
    let temp_a = tempfile::tempdir()?;
    let temp_b = tempfile::tempdir()?;

    let graph_for_root = |root: &std::path::Path| -> anyhow::Result<ExecutionGraph> {
        let sample = root.join("sample.bam");
        let filtered = root.join("out/bam_filter/filtered.bam");
        let summary = root.join("out/bam_filter/filter.summary.json");
        let step = ExecutionStep {
            step_id: StepId::from_static("bam.filter"),
            stage_id: StageId::from_static("bam.filter"),
            image: ContainerImageRefV1 {
                image: "bijux/bam-filter".to_string(),
                digest: Some("sha256:bam-filter".to_string()),
            },
            command: CommandSpecV1 {
                template: vec![
                    "/bin/sh".to_string(),
                    "-c".to_string(),
                    format!(
                        "samtools view -b {} > {} && echo {}",
                        sample.display(),
                        filtered.display(),
                        summary.display(),
                    ),
                ],
            },
            resources: ToolConstraints {
                runtime: "docker".to_string(),
                mem_gb: 2,
                tmp_gb: 1,
                threads: 1,
            },
            io: StageIO {
                inputs: vec![ArtifactRef::required(
                    ArtifactId::new("bam.input"),
                    sample,
                    ArtifactRole::Bam,
                )],
                outputs: vec![
                    ArtifactRef::required(
                        ArtifactId::new("bam.output"),
                        filtered,
                        ArtifactRole::Bam,
                    ),
                    ArtifactRef::required(
                        ArtifactId::new("summary.output"),
                        summary,
                        ArtifactRole::ReportJson,
                    ),
                ],
            },
            out_dir: root.join("out/bam_filter"),
            aux_images: BTreeMap::new(),
            expected_artifact_ids: Vec::new(),
            metrics_schema_ids: Vec::new(),
        };
        Ok(ExecutionGraph::new(
            "bam-to-bam__default__v1",
            "planner-bam",
            PlanPolicy::PreferAccuracy,
            vec![step],
            Vec::new(),
        )?)
    };

    let workflow_for_root = |root: &std::path::Path| -> WorkflowManifestV1 {
        let mut manifest = WorkflowManifestV1::new("bam", "bam-to-bam__default__v1");
        manifest.inputs = vec![WorkflowInputArtifactV1 {
            artifact_id: "bam".to_string(),
            role: ArtifactRole::Bam,
            path: root.join("sample.bam"),
            layout: None,
            compression: None,
            format_id: Some("bam".to_string()),
        }];
        manifest.requested_stages = vec![WorkflowStageRequestV1 {
            stage_id: "bam.filter".to_string(),
            advisory_only: false,
        }];
        manifest
    };

    let params_for_root = |root: &std::path::Path| {
        BTreeMap::from([(
            "bam.filter".to_string(),
            serde_json::json!({
                "input_bam": root.join("sample.bam"),
                "output_bam": root.join("out/bam_filter/filtered.bam"),
            }),
        )])
    };

    let manifest_a = build_plan_manifest(PlanManifestBuildInputV1 {
        workflow_manifest: workflow_for_root(temp_a.path()),
        graph: graph_for_root(temp_a.path())?,
        stage_contract_refs: vec![("bam.filter".to_string(), "sha256:bam-filter".to_string())],
        effective_parameters_by_step: params_for_root(temp_a.path()),
        parameter_traces: vec![ParameterResolutionTraceV1 {
            step_id: "bam.filter".to_string(),
            stage_id: "bam.filter".to_string(),
            parameter: "input_bam".to_string(),
            source: PlannerParameterSourceV1::PlannerInferred,
            resolved_value: serde_json::Value::String(
                PathBuf::from(temp_a.path()).join("sample.bam").display().to_string(),
            ),
            detail: format!(
                "planned from {}",
                PathBuf::from(temp_a.path()).join("sample.bam").display()
            ),
        }],
        refusal_records: Vec::new(),
        warning_records: Vec::new(),
    })?;
    let manifest_b = build_plan_manifest(PlanManifestBuildInputV1 {
        workflow_manifest: workflow_for_root(temp_b.path()),
        graph: graph_for_root(temp_b.path())?,
        stage_contract_refs: vec![("bam.filter".to_string(), "sha256:bam-filter".to_string())],
        effective_parameters_by_step: params_for_root(temp_b.path()),
        parameter_traces: vec![ParameterResolutionTraceV1 {
            step_id: "bam.filter".to_string(),
            stage_id: "bam.filter".to_string(),
            parameter: "input_bam".to_string(),
            source: PlannerParameterSourceV1::PlannerInferred,
            resolved_value: serde_json::Value::String(
                PathBuf::from(temp_b.path()).join("sample.bam").display().to_string(),
            ),
            detail: format!(
                "planned from {}",
                PathBuf::from(temp_b.path()).join("sample.bam").display()
            ),
        }],
        refusal_records: Vec::new(),
        warning_records: Vec::new(),
    })?;

    assert_eq!(manifest_a.graph_hash, manifest_b.graph_hash);
    assert_eq!(manifest_a.plan_fingerprint, manifest_b.plan_fingerprint);
    Ok(())
}

#[test]
fn plan_manifest_diff_is_semantic_and_ignores_notes() -> anyhow::Result<()> {
    let step_a =
        mk_step("fastq.validate_reads", "fastq.validate_reads", ArtifactRole::Reads, ArtifactRole::Reads);
    let step_b =
        mk_step("fastq.trim_reads", "fastq.trim_reads", ArtifactRole::Reads, ArtifactRole::TrimmedReads);
    let graph = ExecutionGraph::new(
        "fastq-to-fastq__default__v1",
        "planner-fastq",
        PlanPolicy::PreferAccuracy,
        vec![step_a, step_b],
        vec![ExecutionEdge::new(
            StepId::from_static("fastq.validate_reads"),
            StepId::from_static("fastq.trim_reads"),
        )],
    )?;

    let workflow_before = fastq_workflow_manifest();
    let mut workflow_after = workflow_before.clone();
    workflow_after.notes = Some("new note".to_string());
    let before = build_plan_manifest(PlanManifestBuildInputV1 {
        workflow_manifest: workflow_before.clone(),
        graph: graph.clone(),
        stage_contract_refs: Vec::new(),
        effective_parameters_by_step: BTreeMap::from([(
            "fastq.trim_reads".to_string(),
            serde_json::json!({ "min_length": 25 }),
        )]),
        parameter_traces: Vec::new(),
        refusal_records: Vec::new(),
        warning_records: Vec::new(),
    })?;
    let after = build_plan_manifest(PlanManifestBuildInputV1 {
        workflow_manifest: workflow_after.clone(),
        graph,
        stage_contract_refs: Vec::new(),
        effective_parameters_by_step: BTreeMap::from([(
            "fastq.trim_reads".to_string(),
            serde_json::json!({ "min_length": 30 }),
        )]),
        parameter_traces: Vec::new(),
        refusal_records: vec![PlannerRefusalRecordV1 {
            code: PlannerRefusalCodeV1::MissingReference,
            stage_id: Some("fastq.trim_reads".to_string()),
            message: "reference bundle missing".to_string(),
            remediation: Some("add a governed reference asset".to_string()),
        }],
        warning_records: Vec::new(),
    })?;

    let diff = diff_plan_manifests(&before, &after, Some(&workflow_before), Some(&workflow_after));
    assert!(!diff.semantically_equal);
    assert_eq!(diff.parameter_changes.len(), 1);
    assert!(diff
        .ignored_changes
        .iter()
        .any(|entry| entry.contains("workflow notes or labels changed")));
    Ok(())
}

#[test]
fn cross_domain_handoffs_require_typed_role_families() -> anyhow::Result<()> {
    let graph = ExecutionGraph::new(
        "fastq-to-vcf__fixture__v1",
        "planner-cross",
        PlanPolicy::PreferAccuracy,
        vec![
            mk_step("fastq.trim_reads", "fastq.trim_reads", ArtifactRole::Reads, ArtifactRole::Reads),
            mk_step("bam.align", "bam.align", ArtifactRole::Reads, ArtifactRole::Bam),
            mk_step("vcf.call", "vcf.call", ArtifactRole::Bam, ArtifactRole::Variant),
        ],
        vec![
            ExecutionEdge::new(
                StepId::from_static("fastq.trim_reads"),
                StepId::from_static("bam.align"),
            ),
            ExecutionEdge::new(
                StepId::from_static("bam.align"),
                StepId::from_static("vcf.call"),
            ),
        ],
    )?;

    let handoffs = validate_cross_domain_handoffs(&graph);
    assert_eq!(handoffs.len(), 2);
    assert!(handoffs.iter().all(|handoff| handoff.compatible));
    let families = handoffs
        .iter()
        .map(|handoff| {
            (
                handoff.from_stage_id.as_str(),
                handoff.to_stage_id.as_str(),
                handoff.artifact_family,
            )
        })
        .collect::<Vec<_>>();
    assert!(families.contains(&(
        "fastq.trim_reads",
        "bam.align",
        Some(bijux_dna_core::contract::ArtifactRoleFamily::Reads),
    )));
    assert!(families.contains(&(
        "bam.align",
        "vcf.call",
        Some(bijux_dna_core::contract::ArtifactRoleFamily::Alignment),
    )));
    Ok(())
}
