use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use anyhow::Result;
use bijux_dna_core::contract::{
    ArtifactRole, CompressionSupport, ReadLayoutMode, WorkflowInputArtifactV1, WorkflowManifestV1,
    WorkflowReferenceAssetV1,
};
use bijux_dna_pipelines::bam::{bam_workflow_template_by_id, bam_workflow_templates_for_pipeline};
use bijux_dna_pipelines::cross::{
    cross_workflow_template_by_id, cross_workflow_templates_for_pipeline,
};
use bijux_dna_pipelines::fastq::{
    fastq_workflow_template_by_id, fastq_workflow_templates_for_pipeline,
};
use bijux_dna_pipelines::{
    build_batch_workflow_graph, build_cross_domain_evidence_narrative,
    evaluate_batch_fan_semantics, evaluate_template_admission, parse_sample_sheet,
    plan_bam_to_vcf_minimal_workflow, plan_fastq_to_bam_ancient_workflow,
    plan_fastq_to_bam_modern_workflow, plan_fastq_to_vcf_minimal_workflow,
    resolve_batch_failure_policy, sample_sheet_to_workflow_manifests,
    summarize_cross_domain_evidence, validate_cross_domain_handoff,
    validate_sample_sheet_preflight, validate_template_overrides, CrossDomainHandoffRequestV1,
};

#[test]
fn sample_sheet_parser_builds_typed_records_and_manifests() -> Result<()> {
    let template = cross_workflow_template_by_id("cross.fastq_to_vcf_minimal")
        .expect("cross template must exist");
    let sheet = parse_sample_sheet(
        &template.template_id,
        "run_id,batch_id,sample_id,library_id,lane_id,layout_mode,reference_id,workflow_mode,r1,r2,expected_outputs\nRUN01,BATCH_A,S1,LIB1,L001,paired_end,GRCh38,minimal,reads/S1_R1.fastq.gz,reads/S1_R2.fastq.gz,bam;vcf;metrics_bundle\nRUN01,BATCH_A,S2,LIB2,L002,paired_end,GRCh38,minimal,reads/S2_R1.fastq.gz,reads/S2_R2.fastq.gz,bam;vcf",
    )?;

    assert_eq!(sheet.records.len(), 2);
    assert_eq!(sheet.records[0].run_id, "RUN01");
    assert_eq!(sheet.records[0].batch_id, "BATCH_A");
    assert_eq!(sheet.records[0].sample_id, "S1");
    assert_eq!(sheet.records[0].expected_outputs, vec!["bam", "vcf", "metrics_bundle"]);

    let manifests = sample_sheet_to_workflow_manifests(&template, &sheet)?;
    assert_eq!(manifests.len(), 2);
    assert_eq!(manifests[0].profile_id, "fastq-to-vcf__minimal__v1");
    assert_eq!(manifests[0].inputs[0].layout, Some(ReadLayoutMode::PairedEnd));
    assert!(manifests[0]
        .evidence_expectations
        .iter()
        .any(|entry| entry.artifact_role == ArtifactRole::Variant));
    Ok(())
}

#[test]
fn sample_sheet_parser_rejects_duplicate_sample_lane_pairs() {
    let error = parse_sample_sheet(
        "cross.fastq_to_vcf_minimal",
        "run_id,batch_id,sample_id,library_id,lane_id,layout_mode,reference_id,workflow_mode,r1,expected_outputs\nRUN01,BATCH_A,S1,LIB1,L001,single_end,GRCh38,minimal,reads/S1_R1.fastq.gz,vcf\nRUN01,BATCH_A,S1,LIB2,L001,single_end,GRCh38,minimal,reads/S1_R1_rep.fastq.gz,vcf",
    )
    .expect_err("duplicate sample/lane must fail");

    assert!(error.to_string().contains("repeats sample/lane pair"));
}

#[test]
fn batch_graph_makes_shared_sample_and_cohort_boundaries_explicit() -> Result<()> {
    let template = cross_workflow_template_by_id("cross.bam_to_vcf_default")
        .expect("cross template must exist");
    let sheet = parse_sample_sheet(
        &template.template_id,
        "run_id,batch_id,sample_id,library_id,lane_id,layout_mode,reference_id,workflow_mode,r1,expected_outputs\nRUN01,BATCH_A,S1,LIB1,L001,single_end,GRCh38,default,reads/S1.bam,vcf\nRUN01,BATCH_A,S2,LIB2,L002,single_end,GRCh38,default,reads/S2.bam,vcf",
    )?;
    let graph = build_batch_workflow_graph(&template, &sheet);

    assert!(graph.nodes.iter().any(|node| node.node_id == "shared::core.prepare_reference"));
    assert!(graph.nodes.iter().any(|node| node.node_id == "sample::S1::bam.genotyping"));
    assert!(graph.nodes.iter().any(|node| node.node_id == "cohort::vcf.stats"));
    assert!(graph
        .edges
        .iter()
        .any(|edge| edge.from == "sample::S1::bam.genotyping" && edge.to == "cohort::vcf.filter"));
    Ok(())
}

#[test]
fn template_override_policy_blocks_locked_parameters_without_expert_mode() {
    let template = cross_workflow_template_by_id("cross.bam_to_vcf_default")
        .expect("cross template must exist");
    let error = validate_template_overrides(
        &template,
        &BTreeMap::from([(
            "bam.genotyping".to_string(),
            serde_json::json!({ "reference_fasta": "alt.fa" }),
        )]),
        false,
    )
    .expect_err("locked override must fail outside expert mode");

    assert!(error.to_string().contains("locked by template policy"));
}

#[test]
fn template_override_policy_allows_safe_knobs() -> Result<()> {
    let template = cross_workflow_template_by_id("cross.fastq_to_vcf_minimal")
        .expect("cross template must exist");
    validate_template_overrides(
        &template,
        &BTreeMap::from([(
            "fastq.trim_reads".to_string(),
            serde_json::json!({ "min_length": 35 }),
        )]),
        false,
    )?;
    Ok(())
}

#[test]
fn template_admission_requires_layout_metadata_reference_and_index() {
    let template = cross_workflow_template_by_id("cross.bam_to_vcf_default")
        .expect("cross template must exist");
    let mut manifest = WorkflowManifestV1::new("cross", "bam-to-vcf__default__v1");
    manifest.inputs.push(WorkflowInputArtifactV1 {
        artifact_id: "reads".to_string(),
        role: ArtifactRole::Bam,
        path: "sample.bam".into(),
        layout: Some(ReadLayoutMode::SingleEnd),
        compression: Some(CompressionSupport::Bgzf),
        format_id: Some("bam".to_string()),
    });
    manifest.reference_assets.push(WorkflowReferenceAssetV1 {
        asset_id: "GRCh38".to_string(),
        role: ArtifactRole::Reference,
        path: "references/GRCh38.fa".into(),
        checksum_sha256: None,
        build_id: Some("GRCh38".to_string()),
        alias_group: Some("GRCh38".to_string()),
    });
    manifest.sample_metadata.insert("sample_id".to_string(), "S1".to_string());
    manifest.sample_metadata.insert("reference_id".to_string(), "GRCh38".to_string());

    let admitted = evaluate_template_admission(&template, &manifest, true);
    assert!(admitted.admitted);

    let blocked = evaluate_template_admission(&template, &manifest, false);
    assert!(!blocked.admitted);
    assert!(blocked.checks.iter().any(|check| check.name == "bam_index" && !check.passed));
}

#[test]
fn cross_evidence_summary_orders_sections_as_one_story() {
    let template = cross_workflow_template_by_id("cross.fastq_to_bam_modern")
        .expect("cross template must exist");
    let summary = summarize_cross_domain_evidence(
        &template,
        &BTreeMap::from([
            ("alignment_quality".to_string(), "alignment passed".to_string()),
            ("read_preprocessing".to_string(), "reads were validated and trimmed".to_string()),
            ("handoff_readiness".to_string(), "bam handoff accepted".to_string()),
        ]),
        &[],
    );

    let ordered_keys =
        summary.sections.iter().map(|section| section.section_id.clone()).collect::<Vec<_>>();
    assert_eq!(
        ordered_keys,
        vec![
            "read_preprocessing".to_string(),
            "alignment_quality".to_string(),
            "handoff_readiness".to_string(),
        ]
    );
    assert!(!summary.final_caveats.is_empty());
}

#[test]
fn templates_are_indexed_by_pipeline_id() {
    let templates = cross_workflow_templates_for_pipeline("fastq-to-vcf__minimal__v1");
    assert_eq!(templates.len(), 1);
    assert_eq!(templates[0].template_id, "cross.fastq_to_vcf_minimal");
}

#[test]
fn sample_sheet_preflight_rejects_refusal_conditions_before_planning() -> Result<()> {
    let template = cross_workflow_template_by_id("cross.fastq_to_vcf_minimal")
        .expect("cross template must exist");
    let sheet = parse_sample_sheet(
        &template.template_id,
        "run_id,batch_id,sample_id,library_id,lane_id,layout_mode,reference_id,workflow_mode,r1,r2,expected_outputs\nRUN01,BATCH_A,S1,LIB1,L01A,paired_end,GRCh38,minimal,reads/S1_R1.fastq.gz,reads/S1_R2.fastq.gz,bam;vcf\nRUN01,BATCH_A,S1,LIB1,L001,single_end,GRCh37,minimal,reads/S1_bad_R1.fastq.gz,,vcf",
    )?;
    let available_inputs = BTreeSet::from([
        PathBuf::from("reads/S1_R1.fastq.gz"),
        PathBuf::from("reads/S1_R2.fastq.gz"),
    ]);
    let known_references = BTreeSet::from(["GRCh38".to_string()]);

    let preflight =
        validate_sample_sheet_preflight(&template, &sheet, &available_inputs, &known_references);

    assert!(!preflight.valid);
    assert_eq!(preflight.records_evaluated, 2);
    assert!(preflight.refusal_codes.contains(&"duplicate_run_id".to_string()));
    assert!(preflight.refusal_codes.contains(&"invalid_lane_id".to_string()));
    assert!(preflight.refusal_codes.contains(&"missing_input_file".to_string()));
    assert!(preflight.refusal_codes.contains(&"reference_id_mismatch".to_string()));
    assert!(preflight.refusal_codes.contains(&"conflicting_layout_for_sample".to_string()));
    Ok(())
}

#[test]
fn fastq_to_bam_modern_plan_exposes_stage_and_handoff_order() -> Result<()> {
    let template = cross_workflow_template_by_id("cross.fastq_to_bam_modern")
        .expect("cross template must exist");
    let sheet = parse_sample_sheet(
        &template.template_id,
        "run_id,batch_id,sample_id,library_id,lane_id,layout_mode,reference_id,workflow_mode,r1,r2,expected_outputs\nRUN11,BATCH_M,S11,LIB11,L001,paired_end,GRCh38,modern,reads/S11_R1.fastq.gz,reads/S11_R2.fastq.gz,bam;metrics_bundle",
    )?;
    let plan = plan_fastq_to_bam_modern_workflow(&template, &sheet)?;

    assert_eq!(plan.sample_plans.len(), 1);
    assert_eq!(
        plan.sample_plans[0].stage_sequence,
        vec![
            "fastq.validate_reads".to_string(),
            "fastq.trim_reads".to_string(),
            "bam.align".to_string(),
            "bam.sort".to_string(),
            "bam.index".to_string(),
            "bam.qc_pre".to_string(),
            "bam.mapping_summary".to_string(),
            "bam.coverage".to_string(),
        ]
    );
    assert_eq!(
        plan.sample_plans[0].handoff_sequence,
        vec![
            "fastq.trim_reads->bam.align".to_string(),
            "bam.align->bam.sort".to_string(),
            "bam.index->bam.qc_pre".to_string(),
        ]
    );
    Ok(())
}

#[test]
fn fastq_to_bam_ancient_plan_captures_damage_and_contamination_flow() -> Result<()> {
    let sheet = parse_sample_sheet(
        "cross.fastq_to_bam_ancient",
        "run_id,batch_id,sample_id,library_id,lane_id,layout_mode,reference_id,workflow_mode,r1,r2,expected_outputs\nRUN21,BATCH_A,S21,LIB21,L001,paired_end,GRCh38,adna_shotgun,reads/S21_R1.fastq.gz,reads/S21_R2.fastq.gz,bam;metrics_bundle",
    )?;
    let plan = plan_fastq_to_bam_ancient_workflow(&sheet)?;

    assert_eq!(plan.template_id, "cross.fastq_to_bam_ancient");
    assert_eq!(
        plan.sample_plans[0].stage_sequence,
        vec![
            "fastq.validate_reads".to_string(),
            "fastq.merge_pairs".to_string(),
            "fastq.trim_terminal_damage".to_string(),
            "bam.align".to_string(),
            "bam.filter".to_string(),
            "bam.damage".to_string(),
            "bam.contamination".to_string(),
            "bam.mapping_summary".to_string(),
        ]
    );
    assert!(plan.caveats.iter().any(|value| value.contains("post-mortem-damage caveated")));
    Ok(())
}

#[test]
fn bam_to_vcf_minimal_plan_enforces_calling_and_filter_boundaries() -> Result<()> {
    let template = cross_workflow_template_by_id("cross.bam_to_vcf_default")
        .expect("cross template must exist");
    let sheet = parse_sample_sheet(
        &template.template_id,
        "run_id,batch_id,sample_id,library_id,lane_id,layout_mode,reference_id,workflow_mode,r1,expected_outputs\nRUN31,BATCH_V,S31,LIB31,L001,single_end,GRCh38,default,reads/S31.bam,vcf;metrics_bundle",
    )?;
    let plan = plan_bam_to_vcf_minimal_workflow(&template, &sheet)?;

    assert_eq!(
        plan.sample_plans[0].stage_sequence,
        vec![
            "bam.validate".to_string(),
            "bam.qc_pre".to_string(),
            "bam.mapping_summary".to_string(),
            "bam.coverage".to_string(),
            "bam.genotyping".to_string(),
        ]
    );
    assert_eq!(plan.cohort_stages, vec!["vcf.filter".to_string(), "vcf.stats".to_string()]);
    assert!(plan.caveats.iter().any(|value| value.contains("normalized output semantics")));
    Ok(())
}

#[test]
fn fastq_to_vcf_minimal_plan_connects_read_to_variant_evidence_path() -> Result<()> {
    let template = cross_workflow_template_by_id("cross.fastq_to_vcf_minimal")
        .expect("cross template must exist");
    let sheet = parse_sample_sheet(
        &template.template_id,
        "run_id,batch_id,sample_id,library_id,lane_id,layout_mode,reference_id,workflow_mode,r1,r2,expected_outputs\nRUN41,BATCH_X,S41,LIB41,L001,paired_end,GRCh38,minimal,reads/S41_R1.fastq.gz,reads/S41_R2.fastq.gz,bam;vcf",
    )?;
    let plan = plan_fastq_to_vcf_minimal_workflow(&template, &sheet)?;

    assert_eq!(
        plan.sample_plans[0].handoff_sequence,
        vec![
            "fastq.trim_reads->bam.align".to_string(),
            "bam.index->bam.qc_pre".to_string(),
            "bam.genotyping->vcf.filter".to_string(),
            "vcf.filter->vcf.stats".to_string(),
        ]
    );
    assert!(plan.caveats.iter().any(|value| value.contains("tiny-fixture proof")));
    Ok(())
}

#[test]
fn batch_fan_semantics_report_flags_overwrite_risk() -> Result<()> {
    let template = cross_workflow_template_by_id("cross.fastq_to_vcf_minimal")
        .expect("cross template must exist");
    let sheet = parse_sample_sheet(
        &template.template_id,
        "run_id,batch_id,sample_id,library_id,lane_id,layout_mode,reference_id,workflow_mode,r1,r2,expected_outputs\nRUN51,BATCH_F,S51,LIB51,L001,paired_end,GRCh38,minimal,reads/S51_R1.fastq.gz,reads/S51_R2.fastq.gz,bam;vcf\nRUN52,BATCH_F,S52,LIB52,L002,paired_end,GRCh38,minimal,reads/S52_R1.fastq.gz,reads/S52_R2.fastq.gz,bam;vcf",
    )?;
    let mut graph = build_batch_workflow_graph(&template, &sheet);
    graph.edges.push(bijux_dna_pipelines::WorkflowBatchEdgeV1 {
        from: "sample::S51::bam.genotyping".to_string(),
        to: "sample::S52::bam.genotyping".to_string(),
        fan_pattern: bijux_dna_pipelines::FanPatternV1::FanIn,
        artifact_scope: "sample_artifact".to_string(),
        lineage_fields: vec!["reference_id".to_string()],
    });
    let report = evaluate_batch_fan_semantics(&template, &graph);

    assert!(!report.valid);
    assert!(report.refusal_codes.contains(&"non_cohort_fanin_overwrite_risk".to_string()));
    Ok(())
}

#[test]
fn failure_policy_resolver_maps_stage_family_to_batch_action() {
    let template = cross_workflow_template_by_id("cross.fastq_to_vcf_minimal")
        .expect("cross template must exist");
    let all_samples = BTreeSet::from(["S1".to_string(), "S2".to_string()]);
    let failed = BTreeSet::from(["S1".to_string()]);

    let preprocessing =
        resolve_batch_failure_policy(&template, "preprocessing", &failed, &all_samples)
            .expect("preprocessing policy");
    assert_eq!(
        preprocessing.action,
        bijux_dna_pipelines::TemplateFailureActionV1::SkipFailedSample
    );
    assert_eq!(preprocessing.blocked_samples, vec!["S1".to_string()]);
    assert_eq!(preprocessing.continued_samples, vec!["S2".to_string()]);

    let alignment = resolve_batch_failure_policy(&template, "alignment", &failed, &all_samples)
        .expect("alignment policy");
    assert_eq!(alignment.action, bijux_dna_pipelines::TemplateFailureActionV1::BlockDownstream);
    assert_eq!(alignment.blocked_samples, vec!["S1".to_string(), "S2".to_string()]);

    let variant = resolve_batch_failure_policy(&template, "variant", &failed, &all_samples)
        .expect("variant policy");
    assert_eq!(variant.action, bijux_dna_pipelines::TemplateFailureActionV1::ContinueCohort);
    assert_eq!(variant.continued_samples, vec!["S1".to_string(), "S2".to_string()]);
}

#[test]
fn cross_domain_handoff_validator_enforces_identity_and_trust() {
    let accepted = validate_cross_domain_handoff(&CrossDomainHandoffRequestV1 {
        source_stage_id: "bam.genotyping".to_string(),
        target_stage_id: "vcf.filter".to_string(),
        source_domain: "bam".to_string(),
        target_domain: "vcf".to_string(),
        artifact_role: ArtifactRole::Variant,
        sample_id: "S10".to_string(),
        reference_id: "GRCh38".to_string(),
        trust_class: "production".to_string(),
    });
    assert!(accepted.accepted);

    let refused = validate_cross_domain_handoff(&CrossDomainHandoffRequestV1 {
        source_stage_id: "fastq.validate_reads".to_string(),
        target_stage_id: "bam.align".to_string(),
        source_domain: "fastq".to_string(),
        target_domain: "bam".to_string(),
        artifact_role: ArtifactRole::Reads,
        sample_id: "".to_string(),
        reference_id: "GRCh38".to_string(),
        trust_class: "simulated".to_string(),
    });
    assert!(!refused.accepted);
    assert!(refused.refusal_codes.contains(&"missing_sample_identity".to_string()));
    assert!(refused.refusal_codes.contains(&"simulated_artifact_not_allowed".to_string()));
}

#[test]
fn evidence_narrative_builder_connects_plan_handoff_and_failures() {
    let template = cross_workflow_template_by_id("cross.fastq_to_vcf_minimal")
        .expect("cross template must exist");
    let plan = bijux_dna_pipelines::CrossWorkflowExecutionPlanV1 {
        schema_version: "bijux.cross.workflow_execution_plan.v1".to_string(),
        template_id: template.template_id.clone(),
        pipeline_id: template.pipeline_id.clone(),
        shared_reference_stages: vec!["core.prepare_reference".to_string()],
        sample_plans: vec![bijux_dna_pipelines::CrossWorkflowSampleExecutionPlanV1 {
            sample_id: "S99".to_string(),
            stage_sequence: vec![
                "fastq.validate_reads".to_string(),
                "fastq.trim_reads".to_string(),
                "bam.align".to_string(),
                "bam.genotyping".to_string(),
            ],
            handoff_sequence: vec!["bam.genotyping->vcf.filter".to_string()],
        }],
        cohort_stages: vec!["vcf.filter".to_string(), "vcf.stats".to_string()],
        caveats: vec!["tiny fixture only".to_string()],
    };
    let handoff = validate_cross_domain_handoff(&CrossDomainHandoffRequestV1 {
        source_stage_id: "bam.genotyping".to_string(),
        target_stage_id: "vcf.filter".to_string(),
        source_domain: "bam".to_string(),
        target_domain: "vcf".to_string(),
        artifact_role: ArtifactRole::Variant,
        sample_id: "S99".to_string(),
        reference_id: "GRCh38".to_string(),
        trust_class: "production".to_string(),
    });
    let failure = bijux_dna_pipelines::BatchFailureDecisionV1 {
        schema_version: "bijux.cross.failure_decision.v1".to_string(),
        template_id: template.template_id.clone(),
        stage_family: "alignment".to_string(),
        action: bijux_dna_pipelines::TemplateFailureActionV1::BlockDownstream,
        blocked_samples: vec!["S99".to_string()],
        continued_samples: vec![],
        notes: vec!["alignment failure blocked downstream calling".to_string()],
    };
    let narrative = build_cross_domain_evidence_narrative(&template, &plan, &handoff, &[failure]);
    assert!(!narrative.paragraphs.is_empty());
    assert!(narrative
        .paragraphs
        .iter()
        .any(|value| value.contains("cross-domain handoff checks accepted")));
    assert!(narrative
        .caveats
        .iter()
        .any(|value| value.contains("alignment failure blocked downstream calling")));
}

#[test]
fn fastq_iteration_11_templates_expose_goal_stage_surfaces() {
    let cases = [
        (
            "fastq.qc_only_review",
            vec![
                "fastq.validate_reads",
                "fastq.profile_read_lengths",
                "fastq.profile_overrepresented_sequences",
                "fastq.report_qc",
            ],
        ),
        (
            "fastq.trim_qc",
            vec![
                "fastq.detect_adapters",
                "fastq.trim_reads",
                "fastq.filter_reads",
                "fastq.report_qc",
            ],
        ),
        (
            "fastq.umi_aware_preprocessing",
            vec![
                "fastq.extract_umis",
                "fastq.remove_duplicates",
                "fastq.trim_reads",
                "fastq.report_qc",
            ],
        ),
        (
            "fastq.host_depletion",
            vec!["core.prepare_reference", "fastq.deplete_host", "fastq.report_qc"],
        ),
        (
            "fastq.rrna_depletion",
            vec!["core.prepare_reference", "fastq.deplete_rrna", "fastq.report_qc"],
        ),
        (
            "fastq.contaminant_depletion",
            vec![
                "core.prepare_reference",
                "fastq.deplete_reference_contaminants",
                "fastq.report_qc",
            ],
        ),
        (
            "fastq.edna_metabarcoding",
            vec![
                "fastq.normalize_primers",
                "fastq.remove_chimeras",
                "fastq.cluster_otus",
                "fastq.infer_asvs",
                "fastq.screen_taxonomy",
                "fastq.report_qc",
            ],
        ),
        (
            "fastq.ancient_dna_preprocessing",
            vec![
                "fastq.merge_pairs",
                "fastq.trim_terminal_damage",
                "fastq.filter_low_complexity",
                "fastq.report_qc",
            ],
        ),
        (
            "fastq.adapter_primer_bank_review",
            vec![
                "fastq.detect_adapters",
                "fastq.normalize_primers",
                "fastq.profile_reads",
                "fastq.report_qc",
            ],
        ),
        (
            "fastq.preprocessing_policy_diff",
            vec![
                "fastq.trim_reads",
                "fastq.filter_reads",
                "fastq.profile_reads",
                "fastq.report_qc",
            ],
        ),
    ];

    for (template_id, stages) in cases {
        let template =
            fastq_workflow_template_by_id(template_id).expect("FASTQ workflow template must exist");
        for stage in stages {
            assert!(
                template.requested_stages.iter().any(|entry| entry == stage),
                "template {template_id} must include stage {stage}"
            );
        }
    }
}

#[test]
fn fastq_iteration_11_templates_are_indexed_by_pipeline_ids() {
    assert_eq!(
        fastq_workflow_templates_for_pipeline("fastq-to-fastq__trim_qc__v1")
            .iter()
            .map(|template| template.template_id.as_str())
            .collect::<Vec<_>>(),
        vec!["fastq.trim_qc", "fastq.preprocessing_policy_diff"],
    );
    assert_eq!(
        fastq_workflow_templates_for_pipeline("fastq-to-fastq__reference_adna__v1")
            .iter()
            .map(|template| template.template_id.as_str())
            .collect::<Vec<_>>(),
        vec!["fastq.ancient_dna_preprocessing"],
    );
}

#[test]
fn bam_iteration_12_templates_expose_goal_stage_surfaces() {
    let cases = [
        (
            "bam.modern_wgs_qc",
            vec!["bam.validate", "bam.mapping_summary", "bam.duplication_metrics", "bam.coverage"],
        ),
        (
            "bam.ancient_dna_qc",
            vec!["bam.damage", "bam.authenticity", "bam.contamination", "bam.coverage"],
        ),
        ("bam.low_pass_readiness", vec!["bam.validate", "bam.coverage", "bam.filter"]),
        (
            "bam.targeted_amplicon_qc",
            vec![
                "bam.mapping_summary",
                "bam.duplication_metrics",
                "bam.coverage",
                "bam.endogenous_content",
            ],
        ),
        (
            "bam.aligner_comparison_report",
            vec!["bam.mapping_summary", "bam.qc_pre", "bam.coverage"],
        ),
        (
            "bam.duplicate_method_comparison_report",
            vec!["bam.duplication_metrics", "bam.filter", "bam.coverage"],
        ),
        (
            "bam.contamination_method_comparison_report",
            vec!["bam.contamination", "bam.authenticity", "bam.coverage"],
        ),
        (
            "bam.batch_merge_workflow",
            vec!["bam.validate", "bam.mapping_summary", "bam.filter", "bam.coverage"],
        ),
        ("bam.coverage_review_report", vec!["bam.mapping_summary", "bam.coverage", "bam.qc_pre"]),
        (
            "bam.large_file_performance_profile",
            vec!["bam.filter", "bam.mapping_summary", "bam.coverage"],
        ),
    ];

    for (template_id, stages) in cases {
        let template =
            bam_workflow_template_by_id(template_id).expect("BAM workflow template must exist");
        for stage in stages {
            assert!(
                template.requested_stages.iter().any(|entry| entry == stage),
                "template {template_id} must include stage {stage}"
            );
        }
    }
}

#[test]
fn bam_iteration_12_templates_are_indexed_by_pipeline_ids() {
    assert_eq!(
        bam_workflow_templates_for_pipeline("bam-to-bam__adna_shotgun__v1")
            .iter()
            .map(|template| template.template_id.as_str())
            .collect::<Vec<_>>(),
        vec!["bam.ancient_dna_qc"],
    );
    assert_eq!(
        bam_workflow_templates_for_pipeline("bam-to-bam__adna_capture__v1")
            .iter()
            .map(|template| template.template_id.as_str())
            .collect::<Vec<_>>(),
        vec!["bam.targeted_amplicon_qc"],
    );
    assert_eq!(
        bam_workflow_templates_for_pipeline("bam-to-bam__reference_adna__v1")
            .iter()
            .map(|template| template.template_id.as_str())
            .collect::<Vec<_>>(),
        vec!["bam.contamination_method_comparison_report"],
    );
}
