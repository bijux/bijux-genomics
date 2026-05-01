#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::path::PathBuf;

use bijux_dna_core::contract::{ArtifactRole, StageIO, ToolConstraints};
use bijux_dna_core::ids::*;
use bijux_dna_domain_fastq::params::{
    validate::{PairSyncPolicy, ValidationMode},
    PairedMode,
};
use bijux_dna_domain_fastq::{
    ValidateFailureClass, ValidatedReadsManifestV1, ValidationReportV1,
    VALIDATED_READS_MANIFEST_SCHEMA_VERSION, VALIDATION_REPORT_SCHEMA_VERSION,
};
use bijux_dna_stage_contract::{ArtifactRef, PlanDecisionReason, StagePlugin};

use super::{semantic::validate_semantic_metrics, FastqStagePlugin};

fn write_fastq(path: &std::path::Path, read_id: &str, sequence: &str) {
    let quality = "#".repeat(sequence.len());
    bijux_dna_infra::write_bytes(path, format!("@{read_id}\n{sequence}\n+\n{quality}\n"))
        .expect("write fastq");
}

fn plan(stage_id: &'static str) -> bijux_dna_stage_contract::StagePlanV1 {
    bijux_dna_stage_contract::StagePlanV1 {
        stage_id: StageId::from_static(stage_id),
        stage_instance_id: None,
        stage_version: StageVersion(1),
        tool_id: ToolId::from_static("fastqc"),
        tool_version: "test".to_string(),
        image: serde_json::from_value(serde_json::json!({
            "image": "bijuxdna/test",
            "digest": null,
        }))
        .expect("image"),
        command: serde_json::from_value(serde_json::json!({
            "template": ["echo", "ok"],
        }))
        .expect("command"),
        resources: ToolConstraints::default(),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::new("reads_r1"),
                PathBuf::from("reads.fastq.gz"),
                ArtifactRole::Reads,
            )],
            outputs: vec![ArtifactRef::required(
                ArtifactId::new("report_json"),
                PathBuf::from("report.json"),
                ArtifactRole::ReportJson,
            )],
        },
        out_dir: PathBuf::from("out"),
        params: serde_json::json!({}),
        effective_params: serde_json::json!({}),
        operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
        aux_images: std::collections::BTreeMap::new(),
        canonical_contract: None,
        provenance: None,
        reason: PlanDecisionReason::default(),
    }
}

#[test]
fn fastq_stage_plugin_handles_only_registered_fastq_stage_ids() {
    let plugin = FastqStagePlugin;

    assert!(plugin.handles_stage("fastq.detect_adapters"));
    assert!(!plugin.handles_stage("fastq.not_registered"));
    assert!(!plugin.handles_stage("bam.align"));
}

#[test]
fn fastq_stage_plugin_rejects_materializing_unsupported_stage_ids() {
    let plugin = FastqStagePlugin;
    let plan = plan("fastq.not_registered");

    let error = match plugin.materialize(&plan) {
        Ok(_) => panic!("unsupported FASTQ stages must fail"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("unsupported FASTQ stage fastq.not_registered"));
}

#[test]
fn fastq_stage_plugin_rejects_parsing_unsupported_stage_ids() {
    let plugin = FastqStagePlugin;
    let plan = plan("fastq.not_registered");

    let error = match plugin.parse_outputs(&plan, &[]) {
        Ok(_) => panic!("unsupported FASTQ stages must fail"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("unsupported FASTQ stage fastq.not_registered"));
}

#[test]
fn fastq_stage_plugin_rejects_empty_command_templates() {
    let plugin = FastqStagePlugin;
    let mut plan = plan("fastq.detect_adapters");
    plan.command.template.clear();

    let error = match plugin.materialize(&plan) {
        Ok(_) => panic!("empty command templates must fail"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("empty command template"));
}

#[test]
fn fastq_stage_plugin_rejects_blank_command_template_arguments() {
    let plugin = FastqStagePlugin;
    let mut plan = plan("fastq.detect_adapters");
    plan.command.template = vec!["fastqc".to_string(), "   ".to_string()];

    let error = match plugin.materialize(&plan) {
        Ok(_) => panic!("blank command template arguments must fail"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("blank command template argument"));
}

#[test]
fn fastq_stage_plugin_input_fingerprint_is_stable_for_reordered_inputs() -> anyhow::Result<()> {
    let plugin = FastqStagePlugin;
    let temp = bijux_dna_infra::temp_dir("bijux-fastq-plugin-input-order")?;
    let input_a = temp.path().join("a.fastq");
    let input_b = temp.path().join("b.fastq");
    bijux_dna_infra::write_bytes(&input_a, b"@a\nAC\n+\n##\n")?;
    bijux_dna_infra::write_bytes(&input_b, b"@b\nGT\n+\n##\n")?;

    let input_ref_a =
        ArtifactRef::required(ArtifactId::new("reads_a"), input_a, ArtifactRole::Reads);
    let input_ref_b =
        ArtifactRef::required(ArtifactId::new("reads_b"), input_b, ArtifactRole::Reads);
    let mut first_plan = plan("fastq.detect_adapters");
    first_plan.io.inputs = vec![input_ref_a.clone(), input_ref_b.clone()];
    let mut second_plan = plan("fastq.detect_adapters");
    second_plan.io.inputs = vec![input_ref_b, input_ref_a];

    let first = plugin.parse_outputs(&first_plan, &[])?;
    let second = plugin.parse_outputs(&second_plan, &[])?;

    assert_eq!(first.metrics.input_hashes, second.metrics.input_hashes);
    assert_eq!(first.metrics.input_fingerprint, second.metrics.input_fingerprint);
    Ok(())
}

#[test]
fn fastq_stage_plugin_trims_tool_version_in_metrics_envelope() -> anyhow::Result<()> {
    let plugin = FastqStagePlugin;
    let mut plan = plan("fastq.detect_adapters");
    plan.tool_version = " test ".to_string();

    let output = plugin.parse_outputs(&plan, &[])?;

    assert_eq!(output.metrics.tool_version, "test");
    assert_eq!(
        output
            .metrics
            .metric_provenance
            .as_ref()
            .map(|provenance| provenance.tool_version.as_str()),
        Some("test")
    );
    Ok(())
}

#[test]
fn fastq_stage_plugin_uses_reported_outputs_for_metrics() -> anyhow::Result<()> {
    let plugin = FastqStagePlugin;
    let temp = bijux_dna_infra::temp_dir("bijux-fastq-plugin-output-metrics")?;
    let input_path = temp.path().join("input.fastq");
    let actual_output_path = temp.path().join("actual-output.fastq");
    bijux_dna_infra::write_bytes(&input_path, b"@a\nACGT\n+\n####\n")?;
    bijux_dna_infra::write_bytes(&actual_output_path, b"@a\nAC\n+\n##\n")?;

    let mut plan = plan("fastq.trim_reads");
    plan.io.inputs =
        vec![ArtifactRef::required(ArtifactId::new("reads_r1"), input_path, ArtifactRole::Reads)];
    plan.io.outputs = vec![ArtifactRef::required(
        ArtifactId::new("trimmed_reads_r1"),
        temp.path().join("planned-output.fastq"),
        ArtifactRole::Reads,
    )];
    let outputs = vec![ArtifactRef::required(
        ArtifactId::new("trimmed_reads_r1"),
        actual_output_path,
        ArtifactRole::Reads,
    )];

    let output = plugin.parse_outputs(&plan, &outputs)?;

    assert_eq!(output.metrics.metrics["reads_in"], 1);
    assert_eq!(output.metrics.metrics["reads_out"], 1);
    assert_eq!(output.artifacts, outputs);
    Ok(())
}

#[test]
fn parse_outputs_emits_artifacts_report_parts_and_event_hints() {
    let plugin = FastqStagePlugin;
    let plan = plan("fastq.detect_adapters");
    let output = plugin.parse_outputs(&plan, &plan.io.outputs).expect("parse outputs");
    assert_eq!(output.artifacts.len(), 1);
    assert_eq!(output.report_parts.len(), 2);
    assert_eq!(output.event_hints.len(), 1);
    assert!(output.warnings.is_empty());
    assert_eq!(output.invariants.len(), 3);
    assert_eq!(
        output.report_parts[0].payload["runtime_interpretation"],
        serde_json::json!("ObserverSpecialized")
    );
    assert_eq!(
        output.report_parts[1].payload["benchmark_scenarios"][0]["scenario_id"],
        serde_json::json!("detect_adapters_fairness")
    );
    assert_eq!(
        output.verdict.as_ref().map(|verdict| verdict.verdict.clone()),
        Some(bijux_dna_core::prelude::invariants::InvariantStatusV1::Pass)
    );
}

#[test]
fn parse_outputs_warns_when_no_observer_parser_exists() {
    let plugin = FastqStagePlugin;
    let plan = plan("fastq.trim_reads");
    let output = plugin.parse_outputs(&plan, &plan.io.outputs).expect("parse outputs");
    assert_eq!(output.artifacts.len(), 1);
    assert_eq!(output.report_parts.len(), 2);
    assert_eq!(output.warnings.len(), 2);
    assert_eq!(
        output.report_parts[0].payload["runtime_interpretation"],
        serde_json::json!("GenericEnvelope")
    );
    assert_eq!(
        output.report_parts[1].payload["comparison_artifact_ids"],
        serde_json::json!([
            "trim_tool_benchmark_cohort_json",
            "trim_tool_comparison_json",
            "trim_tool_normalization_json"
        ])
    );
    assert_eq!(
        output.report_parts[1].payload["benchmark_scenarios"][0]["scenario_id"],
        serde_json::json!("trim_fairness")
    );
    assert_eq!(
        output.report_parts[1].payload["semantic_loss"],
        serde_json::json!(["observer_specialized_parser_missing"])
    );
    assert!(output.warnings[0].contains("fastq.trim_reads"));
    assert!(output.warnings[1].contains("semantic loss tags"));
    assert_eq!(output.invariants.len(), 3);
    assert_eq!(
        output.verdict.as_ref().map(|verdict| verdict.verdict.clone()),
        Some(bijux_dna_core::prelude::invariants::InvariantStatusV1::Warn)
    );
}

#[test]
fn parse_outputs_surfaces_detect_adapter_semantics() {
    let plugin = FastqStagePlugin;
    let temp = tempfile::tempdir().expect("tempdir");
    let report_path = temp.path().join("adapter_report.json");
    let evidence_dir = temp.path().join("fastqc");
    bijux_dna_infra::ensure_dir(&evidence_dir).expect("create evidence dir");
    bijux_dna_infra::write_bytes(
        &report_path,
        serde_json::json!({
            "schema_version": "bijux.fastq.detect_adapters.report.v2",
            "stage": "fastq.detect_adapters",
            "stage_id": "fastq.detect_adapters",
            "tool_id": "fastqc",
            "paired_mode": "paired_end",
            "threads": 4,
            "inspection_mode": "evidence_only",
            "report_only": true,
            "evidence_engine": "fastqc",
            "evidence_scope": "full_input",
            "evidence_format": "fastqc_summary",
            "evidence_artifact_id": "report_json",
            "detected_adapter_source": "normalized_fastqc_evidence",
            "input_r1": "reads_R1.fastq.gz",
            "input_r2": "reads_R2.fastq.gz",
            "report_json": report_path,
            "adapter_evidence_dir": evidence_dir,
            "reads_in": 200_u64,
            "reads_out": 200_u64,
            "bases_in": 20_000_u64,
            "bases_out": 20_000_u64,
            "pairs_in": 100_u64,
            "pairs_out": 100_u64,
            "mean_q": 31.2,
            "candidate_adapter_count": 2_u64,
            "adapter_trimmed_fraction": 0.08,
            "adapter_content_max": 12.5,
            "adapter_content_mean": 3.2,
            "duplication_rate": 0.15,
            "n_rate": 0.001,
            "kmer_warning_count": 4_u64,
            "overrepresented_sequence_count": 3_u64,
            "runtime_s": 4.0,
            "memory_mb": 64.0,
            "exit_code": 0,
            "raw_backend_report": null,
            "raw_backend_report_format": null
        })
        .to_string(),
    )
    .expect("write report");
    let mut plan = plan("fastq.detect_adapters");
    plan.io.outputs = vec![
        ArtifactRef::required(
            ArtifactId::new("report_json"),
            report_path,
            ArtifactRole::ReportJson,
        ),
        ArtifactRef::optional(
            ArtifactId::new("adapter_evidence_dir"),
            evidence_dir,
            ArtifactRole::StageReport,
        ),
    ];
    let output = plugin.parse_outputs(&plan, &plan.io.outputs).expect("parse outputs");
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["candidate_adapter_count"],
        serde_json::json!(2_u64)
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["evidence_scope"],
        serde_json::json!("full_input")
    );
}

#[test]
fn parse_outputs_surfaces_observed_deduplicate_semantics() {
    let plugin = FastqStagePlugin;
    let temp = tempfile::tempdir().expect("tempdir");
    let reads_path = temp.path().join("reads.fastq");
    let dedup_reads_path = temp.path().join("dedup.fastq");
    bijux_dna_infra::write_bytes(&reads_path, b"@r1\nACGT\n+\n####\n").expect("write reads");
    bijux_dna_infra::write_bytes(&dedup_reads_path, b"@r1\nACGT\n+\n####\n")
        .expect("write dedup reads");
    let report_path = temp.path().join("deduplicate_report.json");
    bijux_dna_infra::write_bytes(
        &report_path,
        serde_json::json!({
            "reads_in": 12_u64,
            "reads_out": 9_u64
        })
        .to_string(),
    )
    .expect("write report");
    let plan = bijux_dna_stage_contract::StagePlanV1 {
        stage_id: StageId::from_static("fastq.remove_duplicates"),
        tool_id: ToolId::from_static("clumpify"),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::new("reads_r1"),
                reads_path,
                ArtifactRole::Reads,
            )],
            outputs: vec![ArtifactRef::required(
                ArtifactId::new("dedup_reads_r1"),
                dedup_reads_path,
                ArtifactRole::Reads,
            )],
        },
        ..plan("fastq.remove_duplicates")
    };

    let output = plugin
        .parse_outputs(
            &plan,
            &[
                plan.io.outputs[0].clone(),
                ArtifactRef::required(
                    ArtifactId::new("report_json"),
                    report_path.clone(),
                    ArtifactRole::ReportJson,
                ),
            ],
        )
        .expect("parse outputs");

    assert_eq!(
        output.report_parts[0].payload["semantic_metrics"]["duplicates_removed"],
        serde_json::json!(3_u64)
    );
    assert!(output.report_parts.iter().any(|part| part.name == "observed_semantic_metrics"));
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["dedup_rate"],
        serde_json::json!(0.25)
    );
}

#[test]
fn parse_outputs_surfaces_observed_merge_semantics() {
    let plugin = FastqStagePlugin;
    let temp = tempfile::tempdir().expect("tempdir");
    let reads_r1_path = temp.path().join("reads_R1.fastq");
    let reads_r2_path = temp.path().join("reads_R2.fastq");
    let merged_reads_path = temp.path().join("pear.assembled.fastq");
    let unmerged_r1_path = temp.path().join("pear.unassembled.forward.fastq");
    let unmerged_r2_path = temp.path().join("pear.unassembled.reverse.fastq");
    let report_path = temp.path().join("merge_report.json");
    write_fastq(&reads_r1_path, "r1", "ACGT");
    write_fastq(&reads_r2_path, "r1", "TGCA");
    write_fastq(&merged_reads_path, "merged", "ACGTTGCA");
    bijux_dna_infra::write_bytes(&unmerged_r1_path, b"").expect("write empty unmerged r1");
    bijux_dna_infra::write_bytes(&unmerged_r2_path, b"").expect("write empty unmerged r2");
    bijux_dna_infra::write_bytes(
        &report_path,
        serde_json::json!({
            "schema_version": "bijux.fastq.merge_pairs.report.v2",
            "stage": "fastq.merge_pairs",
            "stage_id": "fastq.merge_pairs",
            "tool_id": "pear",
            "paired_mode": "paired_end",
            "merge_engine": "pear",
            "threads": 4,
            "merge_overlap": 20,
            "min_len": 120,
            "unmerged_read_policy": "omit_unmerged_pairs",
            "input_r1": "reads_R1.fastq.gz",
            "input_r2": "reads_R2.fastq.gz",
            "merged_reads": "pear.assembled.fastq",
            "unmerged_reads_r1": null,
            "unmerged_reads_r2": null,
            "reads_r1": 100,
            "reads_r2": 100,
            "reads_merged": 87,
            "reads_unmerged": 13,
            "merge_rate": 0.87,
            "runtime_s": 2.2,
            "memory_mb": 32.0,
            "raw_backend_report": null,
            "raw_backend_report_format": null
        })
        .to_string(),
    )
    .expect("write report");
    let plan = bijux_dna_stage_contract::StagePlanV1 {
        stage_id: StageId::from_static("fastq.merge_pairs"),
        tool_id: ToolId::from_static("pear"),
        io: StageIO {
            inputs: vec![
                ArtifactRef::required(
                    ArtifactId::new("reads_r1"),
                    reads_r1_path,
                    ArtifactRole::Reads,
                ),
                ArtifactRef::required(
                    ArtifactId::new("reads_r2"),
                    reads_r2_path,
                    ArtifactRole::Reads,
                ),
            ],
            outputs: vec![
                ArtifactRef::required(
                    ArtifactId::new("merged_reads"),
                    merged_reads_path,
                    ArtifactRole::Reads,
                ),
                ArtifactRef::required(
                    ArtifactId::new("unmerged_reads_r1"),
                    unmerged_r1_path,
                    ArtifactRole::Reads,
                ),
                ArtifactRef::required(
                    ArtifactId::new("unmerged_reads_r2"),
                    unmerged_r2_path,
                    ArtifactRole::Reads,
                ),
                ArtifactRef::required(
                    ArtifactId::new("report_json"),
                    report_path,
                    ArtifactRole::ReportJson,
                ),
            ],
        },
        ..plan("fastq.merge_pairs")
    };

    let output = plugin.parse_outputs(&plan, &plan.io.outputs).expect("parse outputs");

    assert_eq!(
        output.report_parts[0].payload["semantic_metrics"]["merge_engine"],
        serde_json::json!("pear")
    );
    assert_eq!(
        output.report_parts[0].payload["semantic_metrics"]["reads_merged"],
        serde_json::json!(87)
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["merge_rate"],
        serde_json::json!(0.87)
    );
}

#[test]
fn validate_semantic_metrics_surface_pair_lineage_contract() {
    let temp = tempfile::tempdir().expect("tempdir");
    let report_path = temp.path().join("validation_report.json");
    let manifest_path = temp.path().join("validated_reads_manifest.json");
    bijux_dna_infra::write_bytes(
        &report_path,
        serde_json::to_string(&ValidationReportV1 {
            schema_version: VALIDATION_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.validate_reads".to_string(),
            stage_id: "fastq.validate_reads".to_string(),
            tool_id: "seqtk".to_string(),
            validation_mode: ValidationMode::Strict,
            pair_sync_policy: PairSyncPolicy::RequireHeaderSync,
            input_r1: "reads_R1.fastq.gz".to_string(),
            input_r2: Some("reads_R2.fastq.gz".to_string()),
            validation_log_r1: "validation_r1.log".to_string(),
            validation_log_r2: Some("validation_r2.log".to_string()),
            validated_inputs: 2,
            validated_reads_r1: 1,
            validated_reads_r2: Some(1),
            validated_pairs: Some(1),
            status_r1: 0,
            status_r2: 0,
            pair_sync_checked: true,
            pair_sync_pass: Some(false),
            pair_count_match: Some(false),
            failure_class: ValidateFailureClass::HeaderSyncMismatch,
            strict_pass: false,
            exit_code: 97,
        })
        .expect("serialize report"),
    )
    .expect("write report");
    bijux_dna_infra::write_bytes(
        &manifest_path,
        serde_json::to_string(&ValidatedReadsManifestV1 {
            schema_version: VALIDATED_READS_MANIFEST_SCHEMA_VERSION.to_string(),
            stage_id: "fastq.validate_reads".to_string(),
            tool_id: "seqtk".to_string(),
            validation_mode: ValidationMode::Strict,
            pair_sync_policy: PairSyncPolicy::RequireHeaderSync,
            input_r1: "reads_R1.fastq.gz".to_string(),
            input_r2: Some("reads_R2.fastq.gz".to_string()),
            validation_report: "validation_report.json".to_string(),
            paired_mode: PairedMode::PairedEnd,
            validated_stream_ids: vec!["reads_r1".to_string(), "reads_r2".to_string()],
            pair_sync_checked: true,
            pair_sync_pass: Some(false),
            validated_pairs: Some(1),
        })
        .expect("serialize manifest"),
    )
    .expect("write manifest");
    let semantics = validate_semantic_metrics(&[
        ArtifactRef::required(
            ArtifactId::new("validation_report"),
            report_path,
            ArtifactRole::SummaryJson,
        ),
        ArtifactRef::required(
            ArtifactId::new("validated_reads_manifest"),
            manifest_path,
            ArtifactRole::StageReport,
        ),
    ])
    .expect("validate semantics");

    assert_eq!(semantics["validated_pairs"], serde_json::json!(1_u64));
    assert_eq!(semantics["validation_mode"], serde_json::json!("strict"));
    assert_eq!(semantics["failure_class"], serde_json::json!("header_sync_mismatch"));
    assert_eq!(semantics["pair_sync_pass"], serde_json::json!(false));
    assert_eq!(semantics["pair_sync_policy"], serde_json::json!("require_header_sync"));
    assert_eq!(semantics["paired_mode"], serde_json::json!("paired_end"));
    assert_eq!(semantics["validated_stream_ids"], serde_json::json!(["reads_r1", "reads_r2"]));
    assert_eq!(semantics["validated_reads_r1"], serde_json::json!(1_u64));
}

#[test]
fn parse_outputs_surfaces_terminal_damage_semantics() {
    let plugin = FastqStagePlugin;
    let temp = tempfile::tempdir().expect("tempdir");
    let reads_path = temp.path().join("reads.fastq");
    let trimmed_reads_path = temp.path().join("trimmed.fastq");
    let report_path = temp.path().join("trim_terminal_damage_report.json");
    bijux_dna_infra::write_bytes(&reads_path, b"@r1\nACGT\n+\n####\n").expect("write reads");
    bijux_dna_infra::write_bytes(&trimmed_reads_path, b"@r1\nCG\n+\n##\n")
        .expect("write trimmed reads");
    bijux_dna_infra::write_bytes(
        &report_path,
        serde_json::json!({
            "schema_version": "bijux.fastq.trim_terminal_damage.report.v2",
            "stage": "fastq.trim_terminal_damage",
            "stage_id": "fastq.trim_terminal_damage",
            "tool_id": id_catalog::TOOL_CUTADAPT,
            "paired_mode": "single_end",
            "threads": 4,
            "damage_mode": "ancient",
            "execution_policy": "explicit_terminal_trim",
            "trim_5p_bases": 2_u64,
            "trim_3p_bases": 1_u64,
            "requested_trim_5p_bases": 2_u64,
            "requested_trim_3p_bases": 1_u64,
            "udg_classification": "non_udg",
            "input_r1": "reads.fastq",
            "input_r2": null,
            "output_r1": "trimmed.fastq",
            "output_r2": null,
            "reads_in": null,
            "reads_out": null,
            "bases_in": null,
            "bases_out": null,
            "mean_q_before": null,
            "mean_q_after": null,
            "ct_ga_asymmetry_pre": null,
            "ct_ga_asymmetry_post": null,
            "ct_ga_asymmetry_pre_r1": null,
            "ct_ga_asymmetry_post_r1": null,
            "ct_ga_asymmetry_pre_r2": null,
            "ct_ga_asymmetry_post_r2": null,
            "terminal_base_composition_pre_r1": null,
            "terminal_base_composition_post_r1": null,
            "terminal_base_composition_pre_r2": null,
            "terminal_base_composition_post_r2": null,
            "raw_backend_report": "cutadapt.damage.json",
            "raw_backend_report_format": "cutadapt_json",
            "runtime_s": null,
            "memory_mb": null,
            "used_fallback": false,
            "backend_metrics": {"reads_profiled_r1": 1}
        })
        .to_string(),
    )
    .expect("write report");
    let plan = bijux_dna_stage_contract::StagePlanV1 {
        stage_id: StageId::from_static("fastq.trim_terminal_damage"),
        tool_id: ToolId::from_static(id_catalog::TOOL_CUTADAPT),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::new("reads_r1"),
                reads_path,
                ArtifactRole::Reads,
            )],
            outputs: vec![ArtifactRef::required(
                ArtifactId::new("trimmed_reads_r1"),
                trimmed_reads_path,
                ArtifactRole::Reads,
            )],
        },
        ..plan("fastq.trim_terminal_damage")
    };

    let output = plugin
        .parse_outputs(
            &plan,
            &[
                plan.io.outputs[0].clone(),
                ArtifactRef::required(
                    ArtifactId::new("report_json"),
                    report_path.clone(),
                    ArtifactRole::ReportJson,
                ),
            ],
        )
        .expect("parse outputs");

    assert!(output.warnings.is_empty());
    assert_eq!(
        output.report_parts[0].payload["runtime_interpretation"],
        serde_json::json!("ObserverSpecialized")
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["execution_policy"],
        serde_json::json!("explicit_terminal_trim")
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["trim_5p_bases"],
        serde_json::json!(2_u64)
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["threads"],
        serde_json::json!(4_u64)
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["udg_classification"],
        serde_json::json!("non_udg")
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["used_fallback"],
        serde_json::json!(false)
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["raw_backend_report_format"],
        serde_json::json!("cutadapt_json")
    );
}

#[test]
fn parse_outputs_surfaces_trim_read_semantics() {
    let plugin = FastqStagePlugin;
    let temp = tempfile::tempdir().expect("tempdir");
    let reads_path = temp.path().join("reads.fastq");
    let trimmed_reads_path = temp.path().join("trimmed.fastq");
    let report_path = temp.path().join("trim_report.json");
    let raw_backend_report_path = temp.path().join("trim_report.fastp.json");
    bijux_dna_infra::write_bytes(&reads_path, b"@r1\nACGTGGGG\n+\n########\n")
        .expect("write reads");
    bijux_dna_infra::write_bytes(&trimmed_reads_path, b"@r1\nACGT\n+\n####\n")
        .expect("write trimmed reads");
    bijux_dna_infra::write_bytes(
        &raw_backend_report_path,
        serde_json::json!({
            "filtering_result": {
                "passed_filter_reads": 96_u64,
                "low_quality_reads": 3_u64,
                "too_many_N_reads": 1_u64,
                "too_short_reads": 4_u64
            }
        })
        .to_string(),
    )
    .expect("write raw backend report");
    bijux_dna_infra::write_bytes(
        &report_path,
        format!(
            r#"{{
                "schema_version": "bijux.fastq.trim_reads.report.v2",
                "stage": "fastq.trim_reads",
                "stage_id": "fastq.trim_reads",
                "tool_id": "{tool_id}",
                "paired_mode": "single_end",
                "threads": 4,
                "trimming_backend": "fastp",
                "backend_mode": "enforced",
                "input_r1": "reads.fastq",
                "input_r2": null,
                "output_r1": "trimmed.fastq",
                "output_r2": null,
                "min_length": 30,
                "quality_cutoff": 20,
                "adapter_policy": "bank",
                "polyx_policy": "trim",
                "n_policy": "drop",
                "contaminant_policy": "none",
                "adapter_bank_id": "illumina",
                "adapter_bank_hash": "sha256:adapter",
                "adapter_preset": "default",
                "detected_adapter_source": "governed_pattern_scan",
                "adapter_overrides": {{
                    "enable": ["AGATCGGAAGAGC"],
                    "disable": ["polyA"]
                }},
                "prepared_adapter_bank": null,
                "polyx_bank_id": "polyx",
                "polyx_bank_hash": "sha256:polyx",
                "polyx_preset": "illumina_twocolor",
                "contaminant_bank_id": null,
                "contaminant_bank_hash": null,
                "contaminant_preset": null,
                "reads_in": 100,
                "reads_out": 96,
                "bases_in": 1000,
                "bases_out": 840,
                "pairs_in": null,
                "pairs_out": null,
                "mean_q_before": 28.0,
                "mean_q_after": 30.0,
                "effective_trim_params": {{
                    "adapter_policy": "bank",
                    "min_length": 30,
                    "quality_cutoff": 20
                }},
                "runtime_s": 4.2,
                "memory_mb": 128.0,
                "raw_backend_report": "{raw_backend_report}",
                "raw_backend_report_format": "fastp_json"
            }}"#,
            tool_id = id_catalog::TOOL_FASTP,
            raw_backend_report = raw_backend_report_path.display(),
        ),
    )
    .expect("write report");
    let plan = bijux_dna_stage_contract::StagePlanV1 {
        stage_id: StageId::from_static("fastq.trim_reads"),
        tool_id: ToolId::from_static(id_catalog::TOOL_FASTP),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::new("reads_r1"),
                reads_path,
                ArtifactRole::Reads,
            )],
            outputs: vec![ArtifactRef::required(
                ArtifactId::new("trimmed_reads_r1"),
                trimmed_reads_path,
                ArtifactRole::Reads,
            )],
        },
        ..plan("fastq.trim_reads")
    };

    let output = plugin
        .parse_outputs(
            &plan,
            &[
                plan.io.outputs[0].clone(),
                ArtifactRef::required(
                    ArtifactId::new("report_json"),
                    report_path.clone(),
                    ArtifactRole::ReportJson,
                ),
            ],
        )
        .expect("parse outputs");

    assert!(output.warnings.is_empty());
    assert_eq!(
        output.report_parts[0].payload["runtime_interpretation"],
        serde_json::json!("ObserverSpecialized")
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["adapter_policy"],
        serde_json::json!("bank")
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["reads_out"],
        serde_json::json!(96_u64)
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["passed_filter_reads"],
        serde_json::json!(96_u64)
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["threads"],
        serde_json::json!(4)
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["adapter_overrides"],
        serde_json::json!({
            "enable": ["AGATCGGAAGAGC"],
            "disable": ["polyA"]
        })
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["raw_backend_report_format"],
        serde_json::json!("fastp_json")
    );
}

#[test]
fn parse_outputs_surfaces_filter_read_semantics() {
    let plugin = FastqStagePlugin;
    let temp = tempfile::tempdir().expect("tempdir");
    let reads_path = temp.path().join("reads.fastq");
    let filtered_reads_path = temp.path().join("filtered.fastq");
    let report_path = temp.path().join("filter_report.json");
    bijux_dna_infra::write_bytes(&reads_path, b"@r1\nACGTNNNN\n+\n########\n")
        .expect("write reads");
    bijux_dna_infra::write_bytes(&filtered_reads_path, b"@r1\nACGT\n+\n####\n")
        .expect("write filtered");
    bijux_dna_infra::write_bytes(
        &report_path,
        format!(
            r#"{{
            "schema_version": "bijux.fastq.filter_reads.report.v3",
            "stage": "fastq.filter_reads",
            "stage_id": "fastq.filter_reads",
            "tool_id": "{tool_id}",
            "paired_mode": "single_end",
            "threads": 8,
            "input_r1": "reads.fastq.gz",
            "input_r2": null,
            "output_r1": "filtered.fastq.gz",
            "output_r2": null,
            "report_json": "filter_report.json",
            "max_n": 0,
            "max_n_fraction": 0.05,
            "max_n_count": 3,
            "low_complexity_threshold": 20.0,
            "entropy_threshold": 18.0,
            "n_policy": "drop",
            "polyx_policy": "trim",
            "contaminant_db": null,
            "reads_in": 100,
            "reads_out": 95,
            "reads_dropped": 5,
            "reads_removed_by_n": 2,
            "reads_removed_by_entropy": 1,
            "reads_removed_low_complexity": 1,
            "reads_removed_by_kmer": 0,
            "reads_removed_contaminant_kmer": 0,
            "reads_removed_by_length": 1,
            "bases_in": 1000,
            "bases_out": 920,
            "pairs_in": null,
            "pairs_out": null,
            "mean_q_before": 28.0,
            "mean_q_after": 30.0,
            "runtime_s": 1.2,
            "memory_mb": 32.0,
            "exit_code": 0,
            "raw_backend_report": null,
            "raw_backend_report_format": "fastp_json",
            "backend_metrics": {{
                "schema_version": "bijux.fastp.metrics.v1",
                "passed_filter_reads": 95,
                "too_many_n_reads": 2,
                "too_short_reads": 1
            }}
        }}"#,
            tool_id = id_catalog::TOOL_FASTP
        ),
    )
    .expect("write report");
    let plan = bijux_dna_stage_contract::StagePlanV1 {
        stage_id: StageId::from_static("fastq.filter_reads"),
        tool_id: ToolId::from_static(id_catalog::TOOL_FASTP),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::new("reads_r1"),
                reads_path,
                ArtifactRole::Reads,
            )],
            outputs: vec![ArtifactRef::required(
                ArtifactId::new("filtered_reads_r1"),
                filtered_reads_path,
                ArtifactRole::Reads,
            )],
        },
        ..plan("fastq.filter_reads")
    };

    let output = plugin
        .parse_outputs(
            &plan,
            &[
                plan.io.outputs[0].clone(),
                ArtifactRef::required(
                    ArtifactId::new("report_json"),
                    report_path.clone(),
                    ArtifactRole::ReportJson,
                ),
            ],
        )
        .expect("parse outputs");

    assert!(output.warnings.is_empty());
    assert_eq!(
        output.report_parts[0].payload["runtime_interpretation"],
        serde_json::json!("ObserverSpecialized")
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["threads"],
        serde_json::json!(8_u32)
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["max_n_fraction"],
        serde_json::json!(0.05)
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["reads_removed_by_n"],
        serde_json::json!(2_u64)
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["passed_filter_reads"],
        serde_json::json!(95_u64)
    );
}

#[test]
fn parse_outputs_surfaces_low_complexity_semantics() {
    let plugin = FastqStagePlugin;
    let temp = tempfile::tempdir().expect("tempdir");
    let reads_path = temp.path().join("reads.fastq");
    let filtered_reads_path = temp.path().join("filtered.fastq");
    let report_path = temp.path().join("low_complexity_report.json");
    bijux_dna_infra::write_bytes(&reads_path, b"@r1\nACGTNNNN\n+\n########\n")
        .expect("write reads");
    bijux_dna_infra::write_bytes(&filtered_reads_path, b"@r1\nACGT\n+\n####\n")
        .expect("write filtered");
    bijux_dna_infra::write_bytes(
        &report_path,
        serde_json::json!({
            "schema_version": "bijux.fastq.filter_low_complexity.report.v2",
            "stage": "fastq.filter_low_complexity",
            "stage_id": "fastq.filter_low_complexity",
            "tool_id": "bbduk",
            "paired_mode": "single_end",
            "threads": 8,
            "input_r1": "reads.fastq.gz",
            "input_r2": null,
            "output_r1": "filtered.fastq.gz",
            "output_r2": null,
            "report_json": "low_complexity_report.json",
            "entropy_threshold": 0.5,
            "polyx_threshold": 20,
            "reads_in": 100,
            "reads_out": 92,
            "reads_removed_low_complexity": 8,
            "bases_in": 1000,
            "bases_out": 910,
            "pairs_in": null,
            "pairs_out": null,
            "mean_q_before": 28.0,
            "mean_q_after": 29.0,
            "runtime_s": 1.1,
            "memory_mb": 64.0,
            "exit_code": 0,
            "raw_backend_report": "bbduk.low_complexity.stats",
            "raw_backend_report_format": "bbduk_stats",
            "backend_metrics": {
                "reads_removed_reported": 8
            }
        })
        .to_string(),
    )
    .expect("write report");
    let plan = bijux_dna_stage_contract::StagePlanV1 {
        stage_id: StageId::from_static("fastq.filter_low_complexity"),
        stage_instance_id: None,
        stage_version: StageVersion(1),
        tool_id: ToolId::from_static("bbduk"),
        tool_version: "test".to_string(),
        image: serde_json::from_value(serde_json::json!({
            "image": "bijuxdna/test",
            "digest": null,
        }))
        .expect("image"),
        command: serde_json::from_value(serde_json::json!({
            "template": ["echo", "ok"],
        }))
        .expect("command"),
        resources: ToolConstraints::default(),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::new("reads_r1"),
                reads_path.clone(),
                ArtifactRole::Reads,
            )],
            outputs: vec![
                ArtifactRef::required(
                    ArtifactId::new("filtered_fastq_r1"),
                    filtered_reads_path.clone(),
                    ArtifactRole::Reads,
                ),
                ArtifactRef::required(
                    ArtifactId::new("filter_report_json"),
                    report_path.clone(),
                    ArtifactRole::ReportJson,
                ),
            ],
        },
        out_dir: temp.path().to_path_buf(),
        params: serde_json::json!({}),
        effective_params: serde_json::json!({}),
        operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
        aux_images: std::collections::BTreeMap::new(),
        canonical_contract: None,
        provenance: None,
        reason: PlanDecisionReason::default(),
    };
    let outputs = plan.io.outputs.clone();

    let output = plugin.parse_outputs(&plan, &outputs).expect("parse outputs");
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["reads_removed_low_complexity"],
        serde_json::json!(8_u64)
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["polyx_threshold"],
        serde_json::json!(20)
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["reads_removed_reported"],
        serde_json::json!(8)
    );
}

#[test]
fn parse_outputs_surfaces_extract_umis_semantics() {
    let plugin = FastqStagePlugin;
    let temp = tempfile::tempdir().expect("tempdir");
    let reads_r1 = temp.path().join("reads_R1.fastq");
    let reads_r2 = temp.path().join("reads_R2.fastq");
    let umi_r1 = temp.path().join("umi_reads_R1.fastq");
    let umi_r2 = temp.path().join("umi_reads_R2.fastq");
    let report_path = temp.path().join("umi_report.json");
    bijux_dna_infra::write_bytes(&reads_r1, b"@r1\nACGT\n+\n####\n").expect("write reads");
    bijux_dna_infra::write_bytes(&reads_r2, b"@r1\nTGCA\n+\n####\n").expect("write reads");
    bijux_dna_infra::write_bytes(&umi_r1, b"@r1_UMI:AAAA\nACGT\n+\n####\n")
        .expect("write umi reads");
    bijux_dna_infra::write_bytes(&umi_r2, b"@r1_UMI:AAAA\nTGCA\n+\n####\n")
        .expect("write umi reads");
    bijux_dna_infra::write_bytes(
        &report_path,
        serde_json::json!({
            "schema_version": "bijux.fastq.extract_umis.report.v2",
            "stage": "fastq.extract_umis",
            "stage_id": "fastq.extract_umis",
            "tool_id": "umi_tools",
            "paired_mode": "paired_end",
            "threads": 2,
            "umi_pattern": "NNNNNNNN",
            "extraction_location": "read1_prefix",
            "read_name_transform": "append_to_header",
            "failed_extraction_policy": "refuse_stage",
            "grouping_policy": "pair_aware",
            "downstream_dedup_policy": "sequence_identity_recommended",
            "downstream_propagation": "header_and_report",
            "input_r1": "reads_R1.fastq.gz",
            "input_r2": "reads_R2.fastq.gz",
            "output_r1": "umi_reads_R1.fastq.gz",
            "output_r2": "umi_reads_R2.fastq.gz",
            "report_json": "umi_report.json",
            "reads_in": 2,
            "reads_out": 2,
            "bases_in": 8,
            "bases_out": 8,
            "pairs_in": 1,
            "pairs_out": 1,
            "reads_with_umi": 2,
            "mean_q_before": 30.0,
            "mean_q_after": 30.0,
            "runtime_s": 1.0,
            "memory_mb": 32.0,
            "exit_code": 0,
            "raw_backend_report": "umi_tools.extract.log",
            "raw_backend_report_format": "umi_tools_log",
            "backend_metrics": {
                "reads_with_umi_fraction": 1.0
            }
        })
        .to_string(),
    )
    .expect("write report");
    let plan = bijux_dna_stage_contract::StagePlanV1 {
        stage_id: StageId::from_static("fastq.extract_umis"),
        stage_instance_id: None,
        stage_version: StageVersion(1),
        tool_id: ToolId::from_static("umi_tools"),
        tool_version: "test".to_string(),
        image: serde_json::from_value(serde_json::json!({
            "image": "bijuxdna/test",
            "digest": null,
        }))
        .expect("image"),
        command: serde_json::from_value(serde_json::json!({
            "template": ["echo", "ok"],
        }))
        .expect("command"),
        resources: ToolConstraints::default(),
        io: StageIO {
            inputs: vec![
                ArtifactRef::required(
                    ArtifactId::new("reads_r1"),
                    reads_r1.clone(),
                    ArtifactRole::Reads,
                ),
                ArtifactRef::required(
                    ArtifactId::new("reads_r2"),
                    reads_r2.clone(),
                    ArtifactRole::Reads,
                ),
            ],
            outputs: vec![
                ArtifactRef::required(
                    ArtifactId::new("umi_reads_r1"),
                    umi_r1.clone(),
                    ArtifactRole::Reads,
                ),
                ArtifactRef::required(
                    ArtifactId::new("umi_reads_r2"),
                    umi_r2.clone(),
                    ArtifactRole::Reads,
                ),
                ArtifactRef::required(
                    ArtifactId::new("report_json"),
                    report_path.clone(),
                    ArtifactRole::ReportJson,
                ),
            ],
        },
        out_dir: temp.path().to_path_buf(),
        params: serde_json::json!({}),
        effective_params: serde_json::json!({}),
        operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
        aux_images: std::collections::BTreeMap::new(),
        canonical_contract: None,
        provenance: None,
        reason: PlanDecisionReason::default(),
    };
    let outputs = plan.io.outputs.clone();

    let output = plugin.parse_outputs(&plan, &outputs).expect("parse outputs");
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["umi_pattern"],
        serde_json::json!("NNNNNNNN")
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["reads_with_umi"],
        serde_json::json!(2_u64)
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["reads_with_umi_fraction"],
        serde_json::json!(1.0)
    );
}

#[test]
fn parse_outputs_surfaces_polyg_trim_semantics() {
    let plugin = FastqStagePlugin;
    let temp = tempfile::tempdir().expect("tempdir");
    let reads_path = temp.path().join("reads.fastq");
    let trimmed_reads_path = temp.path().join("trimmed.fastq");
    let report_path = temp.path().join("trim_polyg_tails_report.json");
    bijux_dna_infra::write_bytes(&reads_path, b"@r1\nACGTGGGG\n+\n########\n")
        .expect("write reads");
    bijux_dna_infra::write_bytes(&trimmed_reads_path, b"@r1\nACGT\n+\n####\n")
        .expect("write trimmed reads");
    bijux_dna_infra::write_bytes(
        &report_path,
        serde_json::json!({
            "schema_version": "bijux.fastq.trim_polyg_tails.report.v2",
            "stage": "fastq.trim_polyg_tails",
            "stage_id": "fastq.trim_polyg_tails",
            "tool_id": id_catalog::TOOL_FASTP,
            "paired_mode": "single_end",
            "threads": 4_u64,
            "trim_polyg": true,
            "min_polyg_run": 10_u64,
            "input_r1": "reads.fastq.gz",
            "input_r2": null,
            "output_r1": "trimmed.fastq.gz",
            "output_r2": null,
            "reads_in": 1_u64,
            "reads_out": 1_u64,
            "bases_in": 8_u64,
            "bases_out": 4_u64,
            "pairs_in": null,
            "pairs_out": null,
            "mean_q_before": 30.0,
            "mean_q_after": 31.0,
            "bases_trimmed_polyg": 4_u64,
            "polyx_bank_id": "polyx",
            "polyx_bank_hash": "sha256:polyx",
            "polyx_preset": "illumina_twocolor",
            "runtime_s": 1.0,
            "memory_mb": 16.0,
            "raw_backend_report": null,
            "raw_backend_report_format": "fastp_json",
            "backend_metrics": {
                "schema_version": "bijux.fastp.metrics.v1",
                "passed_filter_reads": 960_u64,
                "low_quality_reads": 18_u64,
                "too_many_n_reads": 4_u64,
                "too_short_reads": 12_u64
            }
        })
        .to_string(),
    )
    .expect("write report");
    let plan = bijux_dna_stage_contract::StagePlanV1 {
        stage_id: StageId::from_static("fastq.trim_polyg_tails"),
        tool_id: ToolId::from_static(id_catalog::TOOL_FASTP),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::new("reads_r1"),
                reads_path,
                ArtifactRole::Reads,
            )],
            outputs: vec![ArtifactRef::required(
                ArtifactId::new("trimmed_reads_r1"),
                trimmed_reads_path,
                ArtifactRole::Reads,
            )],
        },
        ..plan("fastq.trim_polyg_tails")
    };

    let output = plugin
        .parse_outputs(
            &plan,
            &[
                plan.io.outputs[0].clone(),
                ArtifactRef::required(
                    ArtifactId::new("report_json"),
                    report_path.clone(),
                    ArtifactRole::ReportJson,
                ),
            ],
        )
        .expect("parse outputs");

    assert!(output.warnings.is_empty());
    assert_eq!(
        output.report_parts[0].payload["runtime_interpretation"],
        serde_json::json!("ObserverSpecialized")
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["threads"],
        serde_json::json!(4_u64)
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["min_polyg_run"],
        serde_json::json!(10_u64)
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["raw_backend_report_format"],
        serde_json::json!("fastp_json")
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["polyx_preset"],
        serde_json::json!("illumina_twocolor")
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["passed_filter_reads"],
        serde_json::json!(960_u64)
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["too_short_reads"],
        serde_json::json!(12_u64)
    );
}

#[test]
fn parse_outputs_surfaces_bbduk_polyg_trim_semantics() {
    let plugin = FastqStagePlugin;
    let temp = tempfile::tempdir().expect("tempdir");
    let reads_path = temp.path().join("reads.fastq");
    let trimmed_reads_path = temp.path().join("trimmed.fastq");
    let report_path = temp.path().join("trim_polyg_tails_report.json");
    bijux_dna_infra::write_bytes(&reads_path, b"@r1\nACGTGGGG\n+\n########\n")
        .expect("write reads");
    bijux_dna_infra::write_bytes(&trimmed_reads_path, b"@r1\nACGT\n+\n####\n")
        .expect("write trimmed reads");
    bijux_dna_infra::write_bytes(
        &report_path,
        serde_json::json!({
            "schema_version": "bijux.fastq.trim_polyg_tails.report.v2",
            "stage": "fastq.trim_polyg_tails",
            "stage_id": "fastq.trim_polyg_tails",
            "tool_id": "bbduk",
            "paired_mode": "single_end",
            "threads": 4_u64,
            "trim_polyg": true,
            "min_polyg_run": 10_u64,
            "input_r1": "reads.fastq.gz",
            "input_r2": null,
            "output_r1": "trimmed.fastq.gz",
            "output_r2": null,
            "reads_in": 1_u64,
            "reads_out": 1_u64,
            "bases_in": 8_u64,
            "bases_out": 4_u64,
            "pairs_in": null,
            "pairs_out": null,
            "mean_q_before": 30.0,
            "mean_q_after": 30.5,
            "bases_trimmed_polyg": 4_u64,
            "polyx_bank_id": null,
            "polyx_bank_hash": null,
            "polyx_preset": null,
            "runtime_s": 1.0,
            "memory_mb": 16.0,
            "raw_backend_report": null,
            "raw_backend_report_format": "bbduk_stats",
            "backend_metrics": {
                "schema_version": "bijux.bbduk.trim_polyg.metrics.v1",
                "reads_removed": 137_u64
            }
        })
        .to_string(),
    )
    .expect("write report");
    let plan = bijux_dna_stage_contract::StagePlanV1 {
        stage_id: StageId::from_static("fastq.trim_polyg_tails"),
        tool_id: ToolId::from_static("bbduk"),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::new("reads_r1"),
                reads_path,
                ArtifactRole::Reads,
            )],
            outputs: vec![ArtifactRef::required(
                ArtifactId::new("trimmed_reads_r1"),
                trimmed_reads_path,
                ArtifactRole::Reads,
            )],
        },
        ..plan("fastq.trim_polyg_tails")
    };

    let output = plugin
        .parse_outputs(
            &plan,
            &[
                plan.io.outputs[0].clone(),
                ArtifactRef::required(
                    ArtifactId::new("report_json"),
                    report_path.clone(),
                    ArtifactRole::ReportJson,
                ),
            ],
        )
        .expect("parse outputs");

    assert!(output.warnings.is_empty());
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["threads"],
        serde_json::json!(4_u64)
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["raw_backend_report_format"],
        serde_json::json!("bbduk_stats")
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["reads_removed"],
        serde_json::json!(137_u64)
    );
}

#[test]
fn parse_outputs_surfaces_screen_taxonomy_semantics() {
    let plugin = FastqStagePlugin;
    let temp = tempfile::tempdir().expect("tempdir");
    let reads_path = temp.path().join("reads.fastq");
    let summary_path = temp.path().join("kraken2.report.tsv");
    let report_path = temp.path().join("kraken2.classifications.json");
    bijux_dna_infra::write_bytes(&reads_path, b"@r1\nACGT\n+\n####\n").expect("write reads");
    bijux_dna_infra::write_bytes(&summary_path, b"unclassified\t23\t23.0%\nbacteria\t77\t77.0%\n")
        .expect("write summary");
    bijux_dna_infra::write_bytes(
        &report_path,
        serde_json::json!({
            "schema_version": "bijux.fastq.screen_taxonomy.report.v2",
            "stage": "fastq.screen_taxonomy",
            "stage_id": "fastq.screen_taxonomy",
            "tool_id": id_catalog::TOOL_KRAKEN2,
            "paired_mode": "single_end",
            "threads": 8,
            "classifier": id_catalog::TOOL_KRAKEN2,
            "report_format": "kraken_report",
            "assignment_format": "kraken_assignments",
            "database_catalog_id": "taxonomy_reference",
            "database_artifact_id": "taxonomy_db",
            "database_build_id": "build-2026-03",
            "database_digest": "sha256:taxonomy-db",
            "database_namespace": "read_screening",
            "database_scope": "read_screening",
            "minimum_confidence": 0.05,
            "emit_unclassified": true,
            "interpretation_boundary": "screening_only",
            "truth_conditions": [],
            "input_r1": "reads.fastq.gz",
            "input_r2": null,
            "screen_report_tsv": "kraken2.report.tsv",
            "classification_report_json": "kraken2.classifications.json",
            "reads_in": 1_u64,
            "reads_out": 1_u64,
            "bases_in": 4_u64,
            "bases_out": 4_u64,
            "pairs_in": 0_u64,
            "pairs_out": 0_u64,
            "contamination_rate": 0.77,
            "classified_fraction": 0.77,
            "unclassified_fraction": 0.23,
            "summary_entries": [
                {"label": "unclassified", "percent": 23.0},
                {"label": "bacteria", "percent": 77.0}
            ],
            "top_taxa": [
                {"label": "bacteria", "percent": 77.0}
            ],
            "runtime_s": 1.0,
            "memory_mb": 16.0
        })
        .to_string(),
    )
    .expect("write report");
    let plan = bijux_dna_stage_contract::StagePlanV1 {
        stage_id: StageId::from_static("fastq.screen_taxonomy"),
        tool_id: ToolId::from_static(id_catalog::TOOL_KRAKEN2),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::new("reads_r1"),
                reads_path,
                ArtifactRole::Reads,
            )],
            outputs: vec![
                ArtifactRef::required(
                    ArtifactId::new("screen_report_tsv"),
                    summary_path.clone(),
                    ArtifactRole::SummaryTsv,
                ),
                ArtifactRef::required(
                    ArtifactId::new("classification_report_json"),
                    report_path.clone(),
                    ArtifactRole::ReportJson,
                ),
            ],
        },
        ..plan("fastq.screen_taxonomy")
    };

    let output = plugin.parse_outputs(&plan, &plan.io.outputs).expect("parse outputs");

    assert!(output.warnings.is_empty());
    assert_eq!(
        output.report_parts[0].payload["runtime_interpretation"],
        serde_json::json!("ObserverSpecialized")
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["classifier"],
        serde_json::json!(id_catalog::TOOL_KRAKEN2)
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["database_digest"],
        serde_json::json!("sha256:taxonomy-db")
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["top_taxa"],
        serde_json::json!(["bacteria"])
    );
}

#[test]
fn parse_outputs_surfaces_deplete_rrna_semantics() {
    let plugin = FastqStagePlugin;
    let temp = tempfile::tempdir().expect("tempdir");
    let reads_path = temp.path().join("reads.fastq");
    let output_path = temp.path().join("rrna_filtered.fastq");
    let report_tsv = temp.path().join("rrna_report.tsv");
    let report_json = temp.path().join("rrna_report.json");
    bijux_dna_infra::write_bytes(&reads_path, b"@r1\nACGT\n+\n####\n").expect("write reads");
    bijux_dna_infra::write_bytes(&output_path, b"@r1\nAC\n+\n##\n").expect("write filtered reads");
    bijux_dna_infra::write_bytes(&report_tsv, b"sample\treads_removed\tfraction\n")
        .expect("write tsv");
    bijux_dna_infra::write_bytes(
        &report_json,
        serde_json::json!({
            "schema_version": "bijux.fastq.deplete_rrna.report.v2",
            "stage": "fastq.deplete_rrna",
            "stage_id": "fastq.deplete_rrna",
            "tool_id": "sortmerna",
            "paired_mode": "single_end",
            "threads": 4,
            "rrna_db": "/refs/silva",
            "database_artifact_id": "silva_nr99",
            "database_build_id": "2026.03",
            "screening_engine": "sortmerna",
            "report_format": "summary_tsv_and_json",
            "emit_removed_reads": false,
            "min_identity": 0.95,
            "retained_read_role": "rrna_filtered_reads",
            "rejected_read_role": "removed_rrna_reads",
            "input_r1": "reads.fastq.gz",
            "input_r2": null,
            "output_r1": "rrna_filtered.fastq",
            "output_r2": null,
            "rrna_report_tsv": "rrna_report.tsv",
            "rrna_report_json": "rrna_report.json",
            "reads_in": 100_u64,
            "reads_out": 64_u64,
            "reads_removed": 36_u64,
            "bases_in": 1000_u64,
            "bases_out": 620_u64,
            "bases_removed": 380_u64,
            "pairs_in": null,
            "pairs_out": null,
            "rrna_fraction_removed": 0.36,
            "runtime_s": 5.0,
            "memory_mb": 64.0,
            "exit_code": 0,
            "raw_backend_report": null,
            "raw_backend_report_format": null,
            "backend_metrics": {"reads_removed": 36}
        })
        .to_string(),
    )
    .expect("write report");
    let plan = bijux_dna_stage_contract::StagePlanV1 {
        stage_id: StageId::from_static("fastq.deplete_rrna"),
        tool_id: ToolId::from_static("sortmerna"),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::new("reads_r1"),
                reads_path,
                ArtifactRole::Reads,
            )],
            outputs: vec![
                ArtifactRef::required(
                    ArtifactId::new("rrna_filtered_reads_r1"),
                    output_path,
                    ArtifactRole::Reads,
                ),
                ArtifactRef::required(
                    ArtifactId::new("rrna_report_tsv"),
                    report_tsv,
                    ArtifactRole::SummaryTsv,
                ),
                ArtifactRef::required(
                    ArtifactId::new("rrna_report_json"),
                    report_json,
                    ArtifactRole::ReportJson,
                ),
            ],
        },
        ..plan("fastq.deplete_rrna")
    };

    let output = plugin.parse_outputs(&plan, &plan.io.outputs).expect("parse outputs");

    assert!(output.warnings.is_empty());
    assert_eq!(
        output.report_parts[0].payload["runtime_interpretation"],
        serde_json::json!("ObserverSpecialized")
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["database_artifact_id"],
        serde_json::json!("silva_nr99")
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["reads_removed"],
        serde_json::json!(36_u64)
    );
}

#[test]
fn parse_outputs_surfaces_deplete_reference_contaminants_semantics() {
    let plugin = FastqStagePlugin;
    let temp = tempfile::tempdir().expect("tempdir");
    let reads_path = temp.path().join("reads.fastq");
    let output_path = temp.path().join("contaminant_screened.fastq.gz");
    let report_json = temp.path().join("contaminant_screen_report.json");
    bijux_dna_infra::write_bytes(&reads_path, b"@r1\nACGT\n+\n####\n").expect("write reads");
    bijux_dna_infra::write_bytes(&output_path, b"@r1\nAC\n+\n##\n").expect("write filtered reads");
    bijux_dna_infra::write_bytes(
        &report_json,
        serde_json::json!({
            "schema_version": "bijux.fastq.deplete_reference_contaminants.report.v2",
            "stage": "fastq.deplete_reference_contaminants",
            "stage_id": "fastq.deplete_reference_contaminants",
            "tool_id": "bowtie2",
            "paired_mode": "single_end",
            "threads": 4,
            "reference_catalog_id": "contaminant_reference",
            "contaminant_reference": "phix_and_spikeins",
            "reference_index_artifact_id": "reference_index",
            "reference_index_backend": "bowtie2_build",
            "reference_build_id": "2026.03",
            "reference_digest": "sha256:contaminants",
            "match_threshold": 0.95,
            "retained_read_role": "contaminant_screened_reads",
            "rejected_read_role": "removed_contaminant_reads",
            "retain_unmapped_pairs": false,
            "input_r1": "reads.fastq.gz",
            "input_r2": null,
            "output_r1": "contaminant_screened.fastq.gz",
            "output_r2": null,
            "report_json": "contaminant_screen_report.json",
            "reads_in": 100_u64,
            "reads_out": 72_u64,
            "reads_removed": 28_u64,
            "bases_in": 1000_u64,
            "bases_out": 700_u64,
            "bases_removed": 300_u64,
            "pairs_in": null,
            "pairs_out": null,
            "contaminant_fraction_removed": 0.28,
            "runtime_s": 5.0,
            "memory_mb": 64.0,
            "exit_code": 0,
            "raw_backend_report": "bowtie2.contaminant.metrics.txt",
            "raw_backend_report_format": "bowtie2_met_file",
            "backend_metrics": {"reads_removed": 28}
        })
        .to_string(),
    )
    .expect("write report");
    let plan = bijux_dna_stage_contract::StagePlanV1 {
        stage_id: StageId::from_static("fastq.deplete_reference_contaminants"),
        tool_id: ToolId::from_static("bowtie2"),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::new("reads_r1"),
                reads_path,
                ArtifactRole::Reads,
            )],
            outputs: vec![
                ArtifactRef::required(
                    ArtifactId::new("contaminant_screened_reads_r1"),
                    output_path,
                    ArtifactRole::Reads,
                ),
                ArtifactRef::required(
                    ArtifactId::new("contaminant_screen_report_json"),
                    report_json,
                    ArtifactRole::ReportJson,
                ),
            ],
        },
        ..plan("fastq.deplete_reference_contaminants")
    };

    let output = plugin.parse_outputs(&plan, &plan.io.outputs).expect("parse outputs");

    assert!(output.warnings.is_empty());
    assert_eq!(
        output.report_parts[0].payload["runtime_interpretation"],
        serde_json::json!("ObserverSpecialized")
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["contaminant_reference"],
        serde_json::json!("phix_and_spikeins")
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["reads_removed"],
        serde_json::json!(28_u64)
    );
}

#[test]
fn parse_outputs_surfaces_deplete_host_semantics() {
    let plugin = FastqStagePlugin;
    let temp = tempfile::tempdir().expect("tempdir");
    let reads_path = temp.path().join("reads.fastq");
    let output_path = temp.path().join("host_depleted.fastq.gz");
    let report_json = temp.path().join("host_depletion_report.json");
    bijux_dna_infra::write_bytes(&reads_path, b"@r1\nACGT\n+\n####\n").expect("write reads");
    bijux_dna_infra::write_bytes(&output_path, b"@r1\nAC\n+\n##\n").expect("write filtered reads");
    bijux_dna_infra::write_bytes(
        &report_json,
        r#"{
            "schema_version": "bijux.fastq.deplete_host.report.v2",
            "stage": "fastq.deplete_host",
            "stage_id": "fastq.deplete_host",
            "tool_id": "bowtie2",
            "paired_mode": "single_end",
            "threads": 4,
            "reference_scope": "host",
            "reference_catalog_id": "host_reference",
            "reference_index_artifact_id": "reference_index",
            "reference_index_backend": "bowtie2_build",
            "reference_build_id": "2026.03",
            "reference_digest": "sha256:host",
            "masking_policy": "unmasked",
            "decoy_policy": "none",
            "decoy_catalog_id": null,
            "identity_threshold": 0.95,
            "retained_read_policy": "keep_non_host_reads",
            "emit_removed_reads": true,
            "report_format": "bowtie2_metrics_file",
            "retain_unmapped_pairs": false,
            "input_r1": "reads.fastq.gz",
            "input_r2": null,
            "output_r1": "host_depleted.fastq.gz",
            "output_r2": null,
            "removed_host_r1": "removed_host.fastq.gz",
            "removed_host_r2": null,
            "report_json": "host_depletion_report.json",
            "reads_in": 100,
            "reads_out": 70,
            "reads_removed": 30,
            "bases_in": 1000,
            "bases_out": 680,
            "bases_removed": 320,
            "pairs_in": null,
            "pairs_out": null,
            "host_fraction_removed": 0.30,
            "runtime_s": 5.0,
            "memory_mb": 64.0,
            "exit_code": 0,
            "raw_backend_report": "bowtie2.host.metrics.txt",
            "raw_backend_report_format": "bowtie2_met_file",
            "backend_metrics": {"reads_removed": 30}
        }"#,
    )
    .expect("write report");
    let plan = bijux_dna_stage_contract::StagePlanV1 {
        stage_id: StageId::from_static("fastq.deplete_host"),
        tool_id: ToolId::from_static("bowtie2"),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::new("reads_r1"),
                reads_path,
                ArtifactRole::Reads,
            )],
            outputs: vec![
                ArtifactRef::required(
                    ArtifactId::new("host_depleted_reads_r1"),
                    output_path,
                    ArtifactRole::Reads,
                ),
                ArtifactRef::required(
                    ArtifactId::new("host_depletion_report_json"),
                    report_json,
                    ArtifactRole::ReportJson,
                ),
            ],
        },
        ..plan("fastq.deplete_host")
    };

    let output = plugin.parse_outputs(&plan, &plan.io.outputs).expect("parse outputs");

    assert!(output.warnings.is_empty());
    assert_eq!(
        output.report_parts[0].payload["runtime_interpretation"],
        serde_json::json!("ObserverSpecialized")
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["reference_catalog_id"],
        serde_json::json!("host_reference")
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["host_fraction_removed"],
        serde_json::json!(0.30)
    );
}

#[test]
fn parse_outputs_surfaces_correction_semantics() {
    let plugin = FastqStagePlugin;
    let temp = tempfile::tempdir().expect("tempdir");
    let reads_r1_path = temp.path().join("reads_R1.fastq");
    let reads_r2_path = temp.path().join("reads_R2.fastq");
    let corrected_r1_path = temp.path().join("corrected_R1.fastq");
    let corrected_r2_path = temp.path().join("corrected_R2.fastq");
    let report_path = temp.path().join("correct_report.json");
    bijux_dna_infra::write_bytes(&reads_r1_path, b"@r1\nACGT\n+\n####\n").expect("write reads r1");
    bijux_dna_infra::write_bytes(&reads_r2_path, b"@r1\nTGCA\n+\n####\n").expect("write reads r2");
    bijux_dna_infra::write_bytes(&corrected_r1_path, b"@r1\nACGT\n+\n####\n")
        .expect("write corrected r1");
    bijux_dna_infra::write_bytes(&corrected_r2_path, b"@r1\nTGCA\n+\n####\n")
        .expect("write corrected r2");
    bijux_dna_infra::write_bytes(
        &report_path,
        serde_json::json!({
            "schema_version": "bijux.fastq.correct_errors.report.v2",
            "stage": "fastq.correct_errors",
            "stage_id": "fastq.correct_errors",
            "tool_id": "rcorrector",
            "paired_mode": "paired_end",
            "threads": 4,
            "correction_engine": "rcorrector",
            "quality_encoding": "phred33",
            "kmer_size": 31_u64,
            "genome_size": null,
            "max_memory_gb": null,
            "trusted_kmer_artifact": "trusted.kmers",
            "conservative_mode": false,
            "report_json": "correct_report.json",
            "corrected_reads": 2_u64,
            "reads_in": 2_u64,
            "reads_out": 2_u64,
            "bases_in": 8_u64,
            "bases_out": 8_u64,
            "pairs_in": 1_u64,
            "pairs_out": 1_u64,
            "mean_q_before": 30.0_f64,
            "mean_q_after": 32.5_f64,
            "kmer_fix_rate": 0.125_f64,
            "correction_effect": {
                "outputs_changed": true,
                "bases_delta": -300_i64,
                "mean_q_delta": 2.5_f64
            },
            "runtime_s": 1.0_f64,
            "memory_mb": 64.0_f64,
            "exit_code": 0,
            "raw_backend_report": null,
            "raw_backend_report_format": null,
            "backend_metrics": null,
            "input_r1": reads_r1_path,
            "input_r2": reads_r2_path,
            "output_r1": corrected_r1_path,
            "output_r2": corrected_r2_path
        })
        .to_string(),
    )
    .expect("write report");
    let plan = bijux_dna_stage_contract::StagePlanV1 {
        stage_id: StageId::from_static("fastq.correct_errors"),
        tool_id: ToolId::from_static("rcorrector"),
        io: StageIO {
            inputs: vec![
                ArtifactRef::required(
                    ArtifactId::new("reads_r1"),
                    reads_r1_path,
                    ArtifactRole::Reads,
                ),
                ArtifactRef::required(
                    ArtifactId::new("reads_r2"),
                    reads_r2_path,
                    ArtifactRole::Reads,
                ),
            ],
            outputs: vec![
                ArtifactRef::required(
                    ArtifactId::new("corrected_reads_r1"),
                    corrected_r1_path,
                    ArtifactRole::Reads,
                ),
                ArtifactRef::required(
                    ArtifactId::new("corrected_reads_r2"),
                    corrected_r2_path,
                    ArtifactRole::Reads,
                ),
            ],
        },
        ..plan("fastq.correct_errors")
    };

    let output = plugin
        .parse_outputs(
            &plan,
            &[
                plan.io.outputs[0].clone(),
                plan.io.outputs[1].clone(),
                ArtifactRef::required(
                    ArtifactId::new("report_json"),
                    report_path.clone(),
                    ArtifactRole::ReportJson,
                ),
            ],
        )
        .expect("parse outputs");

    assert!(output.warnings.is_empty());
    assert_eq!(
        output.report_parts[0].payload["runtime_interpretation"],
        serde_json::json!("ObserverSpecialized")
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["correction_engine"],
        serde_json::json!("rcorrector")
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["threads"],
        serde_json::json!(4)
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["quality_encoding"],
        serde_json::json!("phred33")
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["kmer_size"],
        serde_json::json!(31_u64)
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["trusted_kmer_artifact"],
        serde_json::json!("trusted.kmers")
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["correction_effect"]["outputs_changed"],
        serde_json::json!(true)
    );
}

#[test]
fn parse_outputs_surfaces_qc_contributor_lineage_semantics() {
    let plugin = FastqStagePlugin;
    let temp = tempfile::tempdir().expect("tempdir");
    let qc_input_path = temp.path().join("qc_input.fastq");
    let report_json_path = temp.path().join("report_qc_report.json");
    let report_path = temp.path().join("multiqc_report.html");
    let data_dir = temp.path().join("multiqc_data");
    let manifest_path = temp.path().join("governed_qc_inputs_manifest.json");
    bijux_dna_infra::write_bytes(&qc_input_path, b"@r1\nACGT\n+\n####\n").expect("write qc input");
    bijux_dna_infra::ensure_dir(&data_dir).expect("multiqc data dir");
    bijux_dna_infra::write_bytes(
        data_dir.join("multiqc_general_stats.json"),
        include_str!("../../tests/fixtures/tool_metrics/default/multiqc_general_stats.json"),
    )
    .expect("write multiqc general stats");
    bijux_dna_infra::write_bytes(
        &report_json_path,
        serde_json::json!({
            "schema_version": "bijux.fastq.report_qc.report.v2",
            "stage": "fastq.report_qc",
            "stage_id": "fastq.report_qc",
            "tool_id": "multiqc",
            "paired_mode": "single_end",
            "aggregation_engine": "multiqc",
            "aggregation_scope": "governed_qc_artifacts",
            "reads_in": 100,
            "reads_out": 100,
            "bases_in": 400,
            "bases_out": 400,
            "pairs_in": 0,
            "pairs_out": 0,
            "mean_q": 31.2,
            "contamination_rate": 0.0,
            "multiqc_sample_count": 2,
            "multiqc_module_count": 2,
            "raw_fastqc_dir": "/tmp/raw_fastqc",
            "trimmed_fastqc_dir": "/tmp/trimmed_fastqc",
            "multiqc_report": report_path,
            "multiqc_data": data_dir,
            "governed_qc_input_count": 2,
            "governed_qc_contributor_stage_ids": ["fastq.trim_reads", "fastq.validate_reads"],
            "governed_qc_contributor_tool_ids": [
                id_catalog::TOOL_FASTP,
                "fastqvalidator"
            ],
            "governed_qc_contributors": [],
            "governed_qc_lineage_hash": "fastq.trim_reads.fastp=report_json",
            "governed_qc_inputs_manifest": manifest_path,
            "runtime_s": 3.0,
            "memory_mb": 128.0,
            "exit_code": 0
        })
        .to_string(),
    )
    .expect("write governed report");
    bijux_dna_infra::write_bytes(
        &manifest_path,
        serde_json::json!({
            "schema_version": "bijux.fastq.report_qc.inputs.v1",
            "lineage_hash": "fastq.trim_reads.fastp=report_json",
            "raw_fastqc_dir": "/tmp/raw_fastqc",
            "contributors": [
                {
                    "contributor_id": "fastq.trim_reads.fastp",
                    "stage_id": "fastq.trim_reads",
                    "artifact_id": "report_json",
                    "artifact_role": "report_json",
                    "path": "/tmp/fastp/report.json"
                },
                {
                    "contributor_id": "fastq.validate_reads.fastqvalidator",
                    "stage_id": "fastq.validate_reads",
                    "artifact_id": "validation_report",
                    "artifact_role": "validation_report",
                    "path": "/tmp/validate/report.json"
                }
            ],
            "qc_inputs": [
                {
                    "name": "fastq.trim_reads.fastp.report_json",
                    "path": "/tmp/fastp/report.json",
                    "role": "report_json"
                },
                {
                    "name": "fastq.validate_reads.fastqvalidator.validation_report",
                    "path": "/tmp/validate/report.json",
                    "role": "validation_report"
                }
            ]
        })
        .to_string(),
    )
    .expect("write manifest");
    let plan = bijux_dna_stage_contract::StagePlanV1 {
        stage_id: StageId::from_static("fastq.report_qc"),
        tool_id: ToolId::from_static("multiqc"),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::new("qc_artifacts"),
                qc_input_path,
                ArtifactRole::Reads,
            )],
            outputs: vec![
                ArtifactRef::required(
                    ArtifactId::new("report_json"),
                    report_json_path,
                    ArtifactRole::ReportJson,
                ),
                ArtifactRef::required(
                    ArtifactId::new("multiqc_report"),
                    report_path,
                    ArtifactRole::ReportHtml,
                ),
                ArtifactRef::required(
                    ArtifactId::new("multiqc_data"),
                    data_dir,
                    ArtifactRole::Unknown,
                ),
                ArtifactRef::required(
                    ArtifactId::new("governed_qc_inputs_manifest"),
                    manifest_path.clone(),
                    ArtifactRole::SummaryJson,
                ),
            ],
        },
        ..plan("fastq.report_qc")
    };

    let output = plugin.parse_outputs(&plan, &plan.io.outputs).expect("parse outputs");

    assert!(output.warnings.is_empty());
    assert_eq!(
        output.report_parts[0].payload["semantic_metrics"]["contributor_artifact_count"],
        serde_json::json!(2)
    );
    assert_eq!(
        output.report_parts[0].payload["semantic_metrics"]["contributor_stage_ids"],
        serde_json::json!(["fastq.trim_reads", "fastq.validate_reads"])
    );
    assert_eq!(
        output.report_parts[0].payload["semantic_metrics"]["aggregation_engine"],
        serde_json::json!("multiqc")
    );
    assert_eq!(
        output.report_parts[0].payload["semantic_metrics"]["contributor_tool_ids"],
        serde_json::json!([id_catalog::TOOL_FASTP, "fastqvalidator"])
    );
    assert_eq!(
        output.report_parts[0].payload["semantic_metrics"]["multiqc_sample_count"],
        serde_json::json!(2)
    );
    assert_eq!(
        output.report_parts[0].payload["semantic_metrics"]["multiqc_module_count"],
        serde_json::json!(2)
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["lineage_hash"],
        serde_json::json!("fastq.trim_reads.fastp=report_json")
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["trimmed_fastqc_dir"],
        serde_json::json!("/tmp/trimmed_fastqc")
    );
}

#[test]
fn parse_outputs_surfaces_remove_duplicates_semantics() {
    let plugin = FastqStagePlugin;
    let temp = tempfile::tempdir().expect("tempdir");
    let reads_path = temp.path().join("reads.fastq");
    let dedup_reads_path = temp.path().join("dedup.fastq");
    let report_path = temp.path().join("deduplicate_report.json");
    let provenance_path = temp.path().join("duplicate_provenance.json");
    write_fastq(&reads_path, "r1", "ACGT");
    write_fastq(&dedup_reads_path, "r1", "ACGT");
    bijux_dna_infra::write_bytes(
        &report_path,
        serde_json::json!({
            "schema_version": "bijux.fastq.remove_duplicates.report.v2",
            "stage": "fastq.remove_duplicates",
            "stage_id": "fastq.remove_duplicates",
            "tool_id": "clumpify",
            "paired_mode": "single_end",
            "threads": 4,
            "dedup_mode": "optical_aware",
            "keep_order": false,
            "input_r1": "reads.fastq.gz",
            "input_r2": null,
            "output_r1": "dedup.fastq.gz",
            "output_r2": null,
            "reads_in": 100,
            "reads_out": 84,
            "reads_in_r2": null,
            "reads_out_r2": null,
            "pairs_in": null,
            "pairs_out": null,
            "pair_count_match": null,
            "duplicates_removed": 16,
            "dedup_rate": 0.16,
            "duplicate_classes_tsv": "duplicate_classes.tsv",
            "duplicate_provenance_json": provenance_path,
            "duplicate_classes": [
                {"class": "duplicate", "reads_removed": 12, "paired_mode": "single_end"},
                {"class": "optical_duplicate", "reads_removed": 4, "paired_mode": "single_end"}
            ],
            "raw_backend_report": "clumpify.log",
            "raw_backend_report_format": "clumpify_log",
            "runtime_s": 1.9,
            "memory_mb": 48.0
        })
        .to_string(),
    )
    .expect("write dedup report");
    bijux_dna_infra::write_bytes(
        &provenance_path,
        serde_json::json!({
            "schema_version": "bijux.fastq.remove_duplicates.provenance.v2",
            "stage_id": "fastq.remove_duplicates",
            "tool_id": "clumpify",
            "paired_mode": "single_end",
            "threads": 4,
            "dedup_mode": "optical_aware",
            "keep_order": false,
            "duplicates_removed": 16,
            "dedup_rate": 0.16,
            "backend_log": "clumpify.log",
            "input_r1": "reads.fastq.gz",
            "input_r2": null,
            "output_r1": "dedup.fastq.gz",
            "output_r2": null,
            "raw_backend_report": "clumpify.log",
            "raw_backend_report_format": "clumpify_log"
        })
        .to_string(),
    )
    .expect("write dedup provenance");

    let plan = bijux_dna_stage_contract::StagePlanV1 {
        stage_id: StageId::from_static("fastq.remove_duplicates"),
        tool_id: ToolId::from_static("clumpify"),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::new("reads_r1"),
                reads_path,
                ArtifactRole::Reads,
            )],
            outputs: vec![
                ArtifactRef::required(
                    ArtifactId::new("dedup_reads_r1"),
                    dedup_reads_path,
                    ArtifactRole::Reads,
                ),
                ArtifactRef::required(
                    ArtifactId::new("report_json"),
                    report_path,
                    ArtifactRole::ReportJson,
                ),
                ArtifactRef::required(
                    ArtifactId::new("duplicate_provenance_json"),
                    provenance_path,
                    ArtifactRole::SummaryJson,
                ),
            ],
        },
        ..plan("fastq.remove_duplicates")
    };

    let output = plugin.parse_outputs(&plan, &plan.io.outputs).expect("parse outputs");

    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["dedup_mode"],
        serde_json::json!("optical_aware")
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["duplicate_class_count"],
        serde_json::json!(2)
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["backend_log"],
        serde_json::json!("clumpify.log")
    );
}

#[test]
fn parse_outputs_surfaces_profile_read_semantics() {
    let plugin = FastqStagePlugin;
    let temp = tempfile::tempdir().expect("tempdir");
    let reads_r1_path = temp.path().join("reads_R1.fastq");
    let report_path = temp.path().join("qc.json");
    write_fastq(&reads_r1_path, "r1", "ACGT");
    bijux_dna_infra::write_bytes(
        &report_path,
        serde_json::json!({
            "schema_version": "bijux.fastq.profile_reads.report.v2",
            "stage": "fastq.profile_reads",
            "stage_id": "fastq.profile_reads",
            "tool_id": "seqkit_stats",
            "paired_mode": "paired_end",
            "threads": 6,
            "input_r1": "reads_R1.fastq.gz",
            "input_r2": "reads_R2.fastq.gz",
            "qc_json": "qc.json",
            "qc_tsv": "qc.tsv",
            "qc_plots_dir": "plots",
            "length_histogram_source": "seqkit_fx2tab",
            "reads_total": 200,
            "bases_total": 20000,
            "mean_q": 31.2,
            "gc_percent": 42.1,
            "length_histogram": [
                {"length": 100, "count": 180},
                {"length": 101, "count": 20}
            ],
            "mate_summaries": [
                {"label": "reads_r1", "reads": 100, "bases": 10000, "mean_q": 31.0, "gc_percent": 41.9},
                {"label": "reads_r2", "reads": 100, "bases": 10000, "mean_q": 31.4, "gc_percent": 42.3}
            ],
            "runtime_s": 1.5,
            "memory_mb": 20.0,
            "exit_code": 0,
            "raw_backend_report": "qc.tsv",
            "raw_backend_report_format": "seqkit_stats_tsv",
            "backend_metrics": [
                {"schema_version": "bijux.seqkit.metrics.v1", "reads": 100, "bases": 10000, "mean_q": 31.0, "gc_percent": 41.9},
                {"schema_version": "bijux.seqkit.metrics.v1", "reads": 100, "bases": 10000, "mean_q": 31.4, "gc_percent": 42.3}
            ]
        })
        .to_string(),
    )
    .expect("write profile report");

    let plan = bijux_dna_stage_contract::StagePlanV1 {
        stage_id: StageId::from_static("fastq.profile_reads"),
        tool_id: ToolId::from_static("seqkit_stats"),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::new("reads_r1"),
                reads_r1_path,
                ArtifactRole::Reads,
            )],
            outputs: vec![ArtifactRef::required(
                ArtifactId::new("qc_json"),
                report_path,
                ArtifactRole::MetricsJson,
            )],
        },
        ..plan("fastq.profile_reads")
    };

    let output = plugin.parse_outputs(&plan, &plan.io.outputs).expect("parse outputs");

    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["paired_mode"],
        serde_json::json!("paired_end")
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["length_histogram_bins"],
        serde_json::json!(2)
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["mate_summary_count"],
        serde_json::json!(2)
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["raw_backend_report_format"],
        serde_json::json!("seqkit_stats_tsv")
    );
}

#[test]
fn parse_outputs_surfaces_normalize_primer_semantics() {
    let plugin = FastqStagePlugin;
    let temp = tempfile::tempdir().expect("tempdir");
    let reads_path = temp.path().join("reads.fastq");
    let normalized_reads_path = temp.path().join("normalized.fastq");
    let report_path = temp.path().join("normalize_primers_report.json");
    write_fastq(&reads_path, "r1", "ACGT");
    write_fastq(&normalized_reads_path, "r1", "ACGT");
    bijux_dna_infra::write_bytes(
        &report_path,
        serde_json::json!({
            "schema_version": "bijux.fastq.normalize_primers.report.v2",
            "stage": "fastq.normalize_primers",
            "stage_id": "fastq.normalize_primers",
            "tool_id": id_catalog::TOOL_CUTADAPT,
            "paired_mode": "single_end",
            "primer_set_id": "16S_universal_v1",
            "marker_id": "16S",
            "primer_fasta": "assets/reference/primers/16S_universal_v1.fasta",
            "orientation_policy": "normalize_to_forward_primer",
            "max_mismatch_rate": 0.1,
            "min_overlap_bp": 10,
            "input_r1": "reads.fastq.gz",
            "input_r2": null,
            "output_r1": "normalized.fastq.gz",
            "output_r2": null,
            "reads_in": 100,
            "reads_out": 100,
            "bases_in": 1000,
            "bases_out": 980,
            "pairs_in": null,
            "pairs_out": null,
            "primer_trimmed_reads": 95,
            "primer_trimmed_fraction": 0.95,
            "orientation_forward_fraction": 0.93,
            "primer_orientation_report": "primer_orientation.tsv",
            "primer_stats_json": "primer_stats.json",
            "raw_backend_report": "primer_stats.json",
            "raw_backend_report_format": "cutadapt_json",
            "runtime_s": 2.4,
            "memory_mb": 80.0,
            "used_fallback": false,
            "backend_metrics": {}
        })
        .to_string(),
    )
    .expect("write normalize report");

    let plan = bijux_dna_stage_contract::StagePlanV1 {
        stage_id: StageId::from_static("fastq.normalize_primers"),
        tool_id: ToolId::from_static(id_catalog::TOOL_CUTADAPT),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::new("reads_r1"),
                reads_path,
                ArtifactRole::Reads,
            )],
            outputs: vec![
                ArtifactRef::required(
                    ArtifactId::new("normalized_reads_r1"),
                    normalized_reads_path,
                    ArtifactRole::Reads,
                ),
                ArtifactRef::required(
                    ArtifactId::new("report_json"),
                    report_path,
                    ArtifactRole::ReportJson,
                ),
            ],
        },
        ..plan("fastq.normalize_primers")
    };

    let output = plugin.parse_outputs(&plan, &plan.io.outputs).expect("parse outputs");

    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["primer_set_id"],
        serde_json::json!("16S_universal_v1")
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["primer_trimmed_fraction"],
        serde_json::json!(0.95)
    );
}

#[test]
fn parse_outputs_surfaces_normalize_abundance_semantics() {
    let plugin = FastqStagePlugin;
    let temp = tempfile::tempdir().expect("tempdir");
    let report_path = temp.path().join("normalize_abundance_report.json");
    bijux_dna_infra::write_bytes(
        &report_path,
        serde_json::json!({
            "schema_version": "bijux.fastq.normalize_abundance.report.v2",
            "stage": "fastq.normalize_abundance",
            "stage_id": "fastq.normalize_abundance",
            "tool_id": "seqkit",
            "method": "counts_per_million",
            "input_table": "otu_abundance.tsv",
            "normalized_abundance_tsv": "abundance_normalized.tsv",
            "expected_columns": ["sample_id", "feature_id", "abundance"],
            "input_value_column": "abundance",
            "normalized_value_column": "counts_per_million",
            "compositional_rule": "per_sample_sum_to_one_million",
            "scale_factor": 1_000_000.0,
            "table_rows": 12,
            "sample_count": 3,
            "feature_count": 4,
            "zero_fraction": 0.25,
            "per_sample_sums": [["sample_a", 1_000_000.0], ["sample_b", 1_000_000.0]],
            "runtime_s": 1.8,
            "memory_mb": 24.0,
            "raw_backend_report": null,
            "raw_backend_report_format": null,
            "used_fallback": false,
            "backend_metrics": {}
        })
        .to_string(),
    )
    .expect("write normalize abundance report");

    let plan = bijux_dna_stage_contract::StagePlanV1 {
        stage_id: StageId::from_static("fastq.normalize_abundance"),
        tool_id: ToolId::from_static("seqkit"),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::new("abundance_table"),
                temp.path().join("otu_abundance.tsv"),
                ArtifactRole::SummaryTsv,
            )],
            outputs: vec![ArtifactRef::required(
                ArtifactId::new("report_json"),
                report_path,
                ArtifactRole::ReportJson,
            )],
        },
        ..plan("fastq.normalize_abundance")
    };

    let output = plugin.parse_outputs(&plan, &plan.io.outputs).expect("parse outputs");

    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["method"],
        serde_json::json!("counts_per_million")
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["feature_count"],
        serde_json::json!(4)
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["zero_fraction"],
        serde_json::json!(0.25)
    );
}

#[test]
fn parse_outputs_surfaces_infer_asvs_semantics() {
    let plugin = FastqStagePlugin;
    let temp = tempfile::tempdir().expect("tempdir");
    let report_path = temp.path().join("infer_asvs_report.json");
    bijux_dna_infra::write_bytes(
        &report_path,
        serde_json::json!({
            "schema_version": "bijux.fastq.infer_asvs.report.v2",
            "stage": "fastq.infer_asvs",
            "stage_id": "fastq.infer_asvs",
            "tool_id": "dada2",
            "paired_mode": "paired_end",
            "denoising_method": "dada2",
            "pooling_mode": "pseudo_pool",
            "chimera_policy": "remove_bimera_denovo",
            "requires_r_runtime": true,
            "output_table_kind": "asv_abundance_table",
            "input_reads_r1": "reads_R1.fastq.gz",
            "input_reads_r2": "reads_R2.fastq.gz",
            "asv_table_tsv": "asv_abundance.tsv",
            "asv_sequences_fasta": "asv_sequences.fasta",
            "taxonomy_ready_fasta": "taxonomy_ready.fasta",
            "taxonomy_ready_fastq": "taxonomy_ready.fastq",
            "report_json": "infer_asvs_report.json",
            "asv_count": 11,
            "sample_count": 3,
            "representative_sequence_count": 11,
            "used_fallback": false,
            "raw_backend_report": "infer_asvs_report.json",
            "raw_backend_report_format": "infer_asvs_governed_report_json",
            "runtime_s": 3.2,
            "memory_mb": 192.0,
            "exit_code": 0,
            "backend_metrics": {}
        })
        .to_string(),
    )
    .expect("write infer_asvs report");

    let plan = bijux_dna_stage_contract::StagePlanV1 {
        stage_id: StageId::from_static("fastq.infer_asvs"),
        tool_id: ToolId::from_static("dada2"),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::new("reads_r1"),
                temp.path().join("reads.fastq.gz"),
                ArtifactRole::Reads,
            )],
            outputs: vec![ArtifactRef::required(
                ArtifactId::new("report_json"),
                report_path,
                ArtifactRole::ReportJson,
            )],
        },
        ..plan("fastq.infer_asvs")
    };

    let output = plugin.parse_outputs(&plan, &plan.io.outputs).expect("parse outputs");

    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["pooling_mode"],
        serde_json::json!("pseudo_pool")
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["asv_count"],
        serde_json::json!(11)
    );
}

#[test]
fn parse_outputs_surfaces_cluster_otus_semantics() {
    let plugin = FastqStagePlugin;
    let temp = tempfile::tempdir().expect("tempdir");
    let report_path = temp.path().join("cluster_otus_report.json");
    bijux_dna_infra::write_bytes(
        &report_path,
        serde_json::json!({
            "schema_version": "bijux.fastq.cluster_otus.report.v2",
            "stage": "fastq.cluster_otus",
            "stage_id": "fastq.cluster_otus",
            "tool_id": "vsearch",
            "otu_identity": 0.99,
            "threads": 8,
            "input_reads": "merged.fastq.gz",
            "otu_table": "otu_abundance.tsv",
            "otu_representatives": "otu_representatives.fasta",
            "taxonomy_ready_fasta": "taxonomy_ready.fasta",
            "taxonomy_ready_fastq": "taxonomy_ready.fastq",
            "report_json": "cluster_otus_report.json",
            "otu_count": 14,
            "sample_count": 3,
            "representative_sequence_count": 14,
            "output_table_kind": "otu_abundance_table",
            "used_fallback": false,
            "runtime_s": 2.4,
            "memory_mb": 96.0,
            "exit_code": 0,
            "raw_backend_report": "otu_clusters.uc",
            "raw_backend_report_format": "vsearch_uc",
            "backend_metrics": {}
        })
        .to_string(),
    )
    .expect("write cluster otus report");

    let plan = bijux_dna_stage_contract::StagePlanV1 {
        stage_id: StageId::from_static("fastq.cluster_otus"),
        tool_id: ToolId::from_static("vsearch"),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::new("reads"),
                temp.path().join("reads.fastq.gz"),
                ArtifactRole::Reads,
            )],
            outputs: vec![ArtifactRef::required(
                ArtifactId::new("report_json"),
                report_path,
                ArtifactRole::ReportJson,
            )],
        },
        ..plan("fastq.cluster_otus")
    };

    let output = plugin.parse_outputs(&plan, &plan.io.outputs).expect("parse outputs");

    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["otu_identity"],
        serde_json::json!(0.99)
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["otu_count"],
        serde_json::json!(14)
    );
}

#[test]
fn parse_outputs_surfaces_index_reference_semantics() {
    let plugin = FastqStagePlugin;
    let temp = tempfile::tempdir().expect("tempdir");
    let report_path = temp.path().join("index_reference_report.json");
    bijux_dna_infra::write_bytes(
        &report_path,
        serde_json::json!({
            "schema_version": "bijux.fastq.index_reference.report.v2",
            "stage": "fastq.index_reference",
            "stage_id": "fastq.index_reference",
            "tool_id": "bowtie2_build",
            "threads": 4,
            "index_format": "bowtie2_build",
            "reference_fasta": "reference.fa",
            "reference_bytes": 4096,
            "reference_index": "reference_index/bowtie2/reference",
            "report_json": "index_reference_report.json",
            "index_prefix": "reference",
            "emitted_files": [
                {"relative_path": "bowtie2/reference.1.bt2", "bytes": 1024},
                {"relative_path": "bowtie2/reference.2.bt2", "bytes": 2048}
            ],
            "index_file_count": 2,
            "index_bytes": 3072,
            "runtime_s": 1.5,
            "memory_mb": 96.0,
            "exit_code": 0,
            "backend_metrics": {}
        })
        .to_string(),
    )
    .expect("write index report");

    let plan = bijux_dna_stage_contract::StagePlanV1 {
        stage_id: StageId::from_static("fastq.index_reference"),
        tool_id: ToolId::from_static("bowtie2_build"),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::new("reference_fasta"),
                temp.path().join("reference.fa"),
                ArtifactRole::Reference,
            )],
            outputs: vec![ArtifactRef::required(
                ArtifactId::new("report_json"),
                report_path,
                ArtifactRole::ReportJson,
            )],
        },
        ..plan("fastq.index_reference")
    };

    let output = plugin.parse_outputs(&plan, &plan.io.outputs).expect("parse outputs");

    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["index_format"],
        serde_json::json!("bowtie2_build")
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["index_file_count"],
        serde_json::json!(2)
    );
}

#[test]
fn parse_outputs_surfaces_profile_read_length_semantics() {
    let plugin = FastqStagePlugin;
    let temp = tempfile::tempdir().expect("tempdir");
    let reads_r1_path = temp.path().join("reads_R1.fastq");
    let report_path = temp.path().join("profile_read_lengths_report.json");
    write_fastq(&reads_r1_path, "r1", "ACGT");
    bijux_dna_infra::write_bytes(
        &report_path,
        serde_json::json!({
            "schema_version": "bijux.fastq.profile_read_lengths.report.v2",
            "stage": "fastq.profile_read_lengths",
            "stage_id": "fastq.profile_read_lengths",
            "tool_id": "seqkit_stats",
            "paired_mode": "paired_end",
            "threads": 4,
            "histogram_bins": 64,
            "input_r1": "reads_R1.fastq.gz",
            "input_r2": "reads_R2.fastq.gz",
            "length_distribution_tsv": "length_distribution.tsv",
            "length_distribution_json": "length_distribution.json",
            "report_json": "profile_read_lengths_report.json",
            "read_count": 200,
            "mean_read_length": 101.5,
            "max_read_length": 150,
            "distinct_lengths": 12,
            "histogram": [
                {"read_length": 100, "count": 180},
                {"read_length": 101, "count": 20}
            ],
            "runtime_s": 1.1,
            "memory_mb": 16.0,
            "exit_code": 0,
            "raw_backend_report": "length_distribution.tsv",
            "raw_backend_report_format": "seqkit_fx2tab_tsv"
        })
        .to_string(),
    )
    .expect("write profile read lengths report");

    let plan = bijux_dna_stage_contract::StagePlanV1 {
        stage_id: StageId::from_static("fastq.profile_read_lengths"),
        tool_id: ToolId::from_static("seqkit_stats"),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::new("reads_r1"),
                reads_r1_path,
                ArtifactRole::Reads,
            )],
            outputs: vec![ArtifactRef::required(
                ArtifactId::new("report_json"),
                report_path,
                ArtifactRole::ReportJson,
            )],
        },
        ..plan("fastq.profile_read_lengths")
    };

    let output = plugin.parse_outputs(&plan, &plan.io.outputs).expect("parse outputs");

    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["histogram_bins"],
        serde_json::json!(64)
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["histogram_entry_count"],
        serde_json::json!(2)
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["read_count"],
        serde_json::json!(200)
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["raw_backend_report_format"],
        serde_json::json!("seqkit_fx2tab_tsv")
    );
}

#[test]
fn parse_outputs_surfaces_overrepresented_semantics() {
    let plugin = FastqStagePlugin;
    let temp = tempfile::tempdir().expect("tempdir");
    let report_path = temp.path().join("overrepresented_report.json");
    bijux_dna_infra::write_bytes(
        &report_path,
        serde_json::json!({
            "schema_version": "bijux.fastq.profile_overrepresented.report.v2",
            "stage": "fastq.profile_overrepresented_sequences",
            "stage_id": "fastq.profile_overrepresented_sequences",
            "tool_id": "fastqc",
            "paired_mode": "paired_end",
            "threads": 4,
            "top_k": 25,
            "input_r1": "reads_R1.fastq.gz",
            "input_r2": "reads_R2.fastq.gz",
            "overrepresented_sequences_tsv": "overrepresented_sequences.tsv",
            "overrepresented_sequences_json": "overrepresented_sequences.json",
            "report_json": "overrepresented_report.json",
            "sequence_count": 25,
            "flagged_sequences": 3,
            "top_fraction": 0.12,
            "rows": [
                {"sequence": "ACGT", "count": 12, "fraction": 0.12, "flag": "overrepresented"}
            ],
            "runtime_s": 1.4,
            "memory_mb": 48.0,
            "exit_code": 0,
            "raw_backend_report": null,
            "raw_backend_report_format": null
        })
        .to_string(),
    )
    .expect("write overrepresented report");

    let plan = bijux_dna_stage_contract::StagePlanV1 {
        stage_id: StageId::from_static("fastq.profile_overrepresented_sequences"),
        tool_id: ToolId::from_static("fastqc"),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::new("reads_r1"),
                temp.path().join("reads_R1.fastq.gz"),
                ArtifactRole::Reads,
            )],
            outputs: vec![ArtifactRef::required(
                ArtifactId::new("report_json"),
                report_path,
                ArtifactRole::ReportJson,
            )],
        },
        ..plan("fastq.profile_overrepresented_sequences")
    };

    let output = plugin.parse_outputs(&plan, &plan.io.outputs).expect("parse outputs");

    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["top_k"],
        serde_json::json!(25)
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["sequence_count"],
        serde_json::json!(25)
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["flagged_sequences"],
        serde_json::json!(3)
    );
}

#[test]
fn parse_outputs_surfaces_remove_chimeras_semantics() {
    let plugin = FastqStagePlugin;
    let temp = tempfile::tempdir().expect("tempdir");
    let report_path = temp.path().join("remove_chimeras_report.json");
    bijux_dna_infra::write_bytes(
        &report_path,
        serde_json::json!({
            "schema_version": "bijux.fastq.remove_chimeras.report.v2",
            "stage": "fastq.remove_chimeras",
            "stage_id": "fastq.remove_chimeras",
            "tool_id": "vsearch",
            "paired_mode": "single_end",
            "threads": 2,
            "method": "vsearch_uchime_denovo",
            "detection_scope": "denovo",
            "chimera_removed_definition": "reads flagged as de_novo chimeras are excluded from downstream abundance tables",
            "input_reads": "merged.fastq.gz",
            "output_reads": "nonchimeras.fastq.gz",
            "chimera_metrics_json": "chimera_metrics.json",
            "chimeras_fasta": "chimeras.fasta",
            "uchime_report_tsv": "uchime.tsv",
            "reads_in": 100,
            "reads_out": 92,
            "chimeras_removed": 8,
            "chimera_fraction": 0.08,
            "used_fallback": false,
            "raw_backend_report": "uchime.tsv",
            "raw_backend_report_format": "vsearch_uchime_tsv",
            "runtime_s": 1.4,
            "memory_mb": 24.0,
            "exit_code": 0,
            "backend_metrics": {
                "parsed_records": 100,
                "flagged_records": 8
            }
        })
        .to_string(),
    )
    .expect("write chimera report");

    let plan = bijux_dna_stage_contract::StagePlanV1 {
        stage_id: StageId::from_static("fastq.remove_chimeras"),
        tool_id: ToolId::from_static("vsearch"),
        io: StageIO {
            inputs: vec![ArtifactRef::required(
                ArtifactId::new("reads"),
                temp.path().join("merged.fastq.gz"),
                ArtifactRole::Reads,
            )],
            outputs: vec![ArtifactRef::required(
                ArtifactId::new("report_json"),
                report_path,
                ArtifactRole::ReportJson,
            )],
        },
        ..plan("fastq.remove_chimeras")
    };

    let output = plugin.parse_outputs(&plan, &plan.io.outputs).expect("parse outputs");

    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]["method"],
        serde_json::json!("vsearch_uchime_denovo")
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["chimera_fraction"],
        serde_json::json!(0.08)
    );
    assert_eq!(
        output.verdict.as_ref().expect("verdict").key_metrics["semantic_metrics"]
            ["raw_backend_report_format"],
        serde_json::json!("vsearch_uchime_tsv")
    );
}
