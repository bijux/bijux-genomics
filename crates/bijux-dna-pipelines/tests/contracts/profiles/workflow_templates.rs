use std::collections::BTreeMap;

use anyhow::Result;
use bijux_dna_core::contract::{
    ArtifactRole, CompressionSupport, ReadLayoutMode, WorkflowInputArtifactV1, WorkflowManifestV1,
    WorkflowReferenceAssetV1,
};
use bijux_dna_pipelines::cross::{
    cross_workflow_template_by_id, cross_workflow_templates_for_pipeline,
};
use bijux_dna_pipelines::{
    build_batch_workflow_graph, evaluate_template_admission, parse_sample_sheet,
    sample_sheet_to_workflow_manifests, summarize_cross_domain_evidence,
    validate_template_overrides,
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
